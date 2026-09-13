use std::collections::{BinaryHeap, HashMap, HashSet};

use voxt_core::prelude::{ChunkPos, ChunkVersion, Pos};

use crate::{
    chunks::{Chunk, ChunkManager},
    worlds::Tick,
};

pub struct ChunkScheduler {
    demands: HashMap<ChunkPos, ChunkDemand>,
    pending: BinaryHeap<Task>,
    queued: HashSet<TaskKey>,
    in_flight: HashSet<TaskKey>,
    hysteresis: Tick,
}

impl ChunkScheduler {
    pub fn new(hysteresis: Tick) -> Self {
        Self {
            demands: HashMap::new(),
            pending: BinaryHeap::new(),
            queued: HashSet::new(),
            in_flight: HashSet::new(),
            hysteresis,
        }
    }

    pub fn request_load(&mut self, chunk_pos: ChunkPos) {
        let demand = self.demands.entry(chunk_pos).or_default();
        demand.resident = true;
        demand.unload_since = None;
    }

    pub fn request_unload(&mut self, chunk_pos: ChunkPos, curr_tick: Tick) {
        let demand = self.demands.entry(chunk_pos).or_default();
        demand.resident = false;
        demand.unload_since.get_or_insert(curr_tick);
    }

    pub fn mark_mesh_dirty(&mut self, chunk_pos: ChunkPos) {
        self.demands.entry(chunk_pos).or_default().mesh_dirty = true;
    }

    pub fn mark_save_dirty(&mut self, chunk_pos: ChunkPos) {
        self.demands.entry(chunk_pos).or_default().save_dirty = true;
    }

    pub fn request_remesh(&mut self, chunk_pos: ChunkPos) {
        self.mark_mesh_dirty(chunk_pos);
    }

    pub fn update(&mut self, manager: &ChunkManager, player_pos: ChunkPos, curr_tick: Tick) {
        let mut tasks = Vec::new();

        for (&chunk_pos, demand) in &self.demands {
            let distance = chunk_pos.dist_euclid_2(&player_pos);

            if demand.resident && !manager.contains_chunk(&chunk_pos) {
                tasks.push(Task::load(chunk_pos, distance, demand.generation));
            }

            let Some(chunk) = manager.get_chunk(&chunk_pos) else {
                continue;
            };

            if demand.mesh_dirty {
                tasks.push(Task::mesh(
                    chunk_pos,
                    distance,
                    demand.generation,
                    chunk.version(),
                    chunk.clone(),
                ));
            }

            if demand.save_dirty {
                tasks.push(Task::save(
                    chunk_pos,
                    distance,
                    demand.generation,
                    chunk.version(),
                    chunk.clone(),
                ));
            }

            if !demand.resident
                && demand
                    .unload_since
                    .is_some_and(|start| curr_tick.ticks_since(&start) >= self.hysteresis)
            {
                tasks.push(Task::unload(chunk_pos, distance, demand.generation));
            }
        }

        for task in tasks {
            self.enqueue(task);
        }
    }

    #[must_use]
    pub fn next_task(&mut self) -> Option<ChunkTask> {
        self.pending.pop().map(|task| {
            self.queued.remove(&task.key());
            self.in_flight.insert(task.key());
            task.into_public()
        })
    }

    pub fn complete(&mut self, task: &ChunkTask) {
        let key = task.key();
        self.in_flight.remove(&key);

        let Some(demand) = self.demands.get_mut(&key.chunk_pos) else {
            return;
        };
        if demand.generation != key.generation {
            return;
        }

        match task {
            ChunkTask::Load { .. } | ChunkTask::Unload { .. } => {
                demand.unload_since = None;
            }
            ChunkTask::Mesh { .. } => demand.mesh_dirty = false,
            ChunkTask::Save { .. } => demand.save_dirty = false,
        }
    }

    fn enqueue(&mut self, task: Task) {
        if !self.queued.contains(&task.key()) && !self.in_flight.contains(&task.key()) {
            self.queued.insert(task.key());
            self.pending.push(task);
        }
    }
}

#[derive(Debug, Default)]
struct ChunkDemand {
    resident: bool,
    mesh_dirty: bool,
    save_dirty: bool,
    unload_since: Option<Tick>,
    generation: u64,
}

#[derive(Debug, Clone)]
pub enum ChunkTask {
    Load {
        chunk_pos: ChunkPos,
        generation: u64,
    },
    Mesh {
        chunk_pos: ChunkPos,
        generation: u64,
        version: ChunkVersion,
        chunk: Chunk,
    },
    Save {
        chunk_pos: ChunkPos,
        generation: u64,
        version: ChunkVersion,
        chunk: Chunk,
    },
    Unload {
        chunk_pos: ChunkPos,
        generation: u64,
    },
}

impl ChunkTask {
    fn key(&self) -> TaskKey {
        match self {
            Self::Load {
                chunk_pos,
                generation,
            }
            | Self::Unload {
                chunk_pos,
                generation,
            } => TaskKey {
                chunk_pos: *chunk_pos,
                generation: *generation,
                kind: TaskKind::Lifecycle,
            },
            Self::Mesh {
                chunk_pos,
                generation,
                ..
            } => TaskKey {
                chunk_pos: *chunk_pos,
                generation: *generation,
                kind: TaskKind::Mesh,
            },
            Self::Save {
                chunk_pos,
                generation,
                ..
            } => TaskKey {
                chunk_pos: *chunk_pos,
                generation: *generation,
                kind: TaskKind::Save,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TaskKey {
    chunk_pos: ChunkPos,
    generation: u64,
    kind: TaskKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TaskKind {
    Lifecycle,
    Mesh,
    Save,
}

#[derive(Debug)]
struct Task {
    priority: usize,
    key: TaskKey,
    task: ChunkTask,
}

impl Task {
    fn load(chunk_pos: ChunkPos, priority: usize, generation: u64) -> Self {
        Self::new(
            priority,
            ChunkTask::Load {
                chunk_pos,
                generation,
            },
        )
    }

    fn mesh(
        chunk_pos: ChunkPos,
        priority: usize,
        generation: u64,
        version: ChunkVersion,
        chunk: Chunk,
    ) -> Self {
        Self::new(
            priority,
            ChunkTask::Mesh {
                chunk_pos,
                generation,
                version,
                chunk,
            },
        )
    }

    fn save(
        chunk_pos: ChunkPos,
        priority: usize,
        generation: u64,
        version: ChunkVersion,
        chunk: Chunk,
    ) -> Self {
        Self::new(
            priority,
            ChunkTask::Save {
                chunk_pos,
                generation,
                version,
                chunk,
            },
        )
    }

    fn unload(chunk_pos: ChunkPos, priority: usize, generation: u64) -> Self {
        Self::new(
            priority,
            ChunkTask::Unload {
                chunk_pos,
                generation,
            },
        )
    }

    fn new(priority: usize, task: ChunkTask) -> Self {
        let key = task.key();
        Self {
            priority,
            key,
            task,
        }
    }

    fn key(&self) -> TaskKey {
        self.key
    }

    fn into_public(self) -> ChunkTask {
        self.task
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.key == other.key
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.key.chunk_pos.cmp(&other.key.chunk_pos))
    }
}

#[cfg(test)]
mod tests {
    use super::{ChunkScheduler, ChunkTask};
    use crate::chunks::{Chunk, ChunkManager};
    use crate::worlds::Tick;
    use voxt_core::prelude::{ChunkPos, Pos};

    fn pos(x: i16) -> ChunkPos {
        ChunkPos::from_raw(x, 0, 0)
    }

    #[test]
    fn load_request_emits_one_task_and_deduplicates_updates() {
        let mut scheduler = ChunkScheduler::new(Tick::new(10));
        let manager = ChunkManager::new();

        scheduler.request_load(pos(2));
        scheduler.update(&manager, pos(0), Tick::new(1));
        scheduler.update(&manager, pos(0), Tick::new(2));

        assert!(matches!(
            scheduler.next_task(),
            Some(ChunkTask::Load {
                chunk_pos,
                generation: 0
            }) if chunk_pos == pos(2)
        ));
        assert!(scheduler.next_task().is_none());
    }

    #[test]
    fn dirty_demands_are_independent() {
        let mut scheduler = ChunkScheduler::new(Tick::new(10));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);
        manager.insert_chunk(Chunk::new(chunk_pos));

        scheduler.mark_mesh_dirty(chunk_pos);
        scheduler.mark_save_dirty(chunk_pos);
        scheduler.update(&manager, chunk_pos, Tick::new(1));

        let first = scheduler.next_task();
        let second = scheduler.next_task();
        assert!(matches!(first, Some(ChunkTask::Mesh { .. })));
        assert!(matches!(second, Some(ChunkTask::Save { .. })));
    }

    #[test]
    fn completing_a_task_clears_only_that_demand() {
        let mut scheduler = ChunkScheduler::new(Tick::new(10));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);
        manager.insert_chunk(Chunk::new(chunk_pos));

        scheduler.mark_mesh_dirty(chunk_pos);
        scheduler.mark_save_dirty(chunk_pos);
        scheduler.update(&manager, chunk_pos, Tick::new(1));

        let mesh = scheduler.next_task().unwrap();
        let save = scheduler.next_task().unwrap();
        scheduler.complete(&mesh);

        scheduler.update(&manager, chunk_pos, Tick::new(2));
        assert!(scheduler.next_task().is_none());
        scheduler.complete(&save);
        scheduler.update(&manager, chunk_pos, Tick::new(3));
        assert!(scheduler.next_task().is_none());
    }
}
