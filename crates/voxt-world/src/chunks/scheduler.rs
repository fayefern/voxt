use std::collections::{BinaryHeap, HashMap, HashSet};

use voxt_core::prelude::{ChunkPos, Pos, Version};

use crate::{
    chunks::{Chunk, ChunkManager},
    worlds::Tick,
};

pub struct ChunkScheduler {
    config: ChunkSchedulerConfig,

    demanded: HashMap<ChunkPos, ChunkDemand>,
    pending: BinaryHeap<ChunkTask>,
    active: HashSet<ChunkTaskKey>,
}

impl ChunkScheduler {
    pub fn new(config: ChunkSchedulerConfig) -> Self {
        Self {
            config,
            demanded: HashMap::new(),
            pending: BinaryHeap::new(),
            active: HashSet::new(),
        }
    }

    pub fn add(
        &mut self,
        demand_kind: ChunkDemandKind,
        chunk_pos: ChunkPos,
        chunk_ver: Version,
        curr_tick: Tick,
    ) {
        let demand = self
            .demanded
            .entry(chunk_pos)
            .and_modify(|d| d.version = chunk_ver)
            .or_insert_with(|| ChunkDemand::new(chunk_ver));

        match demand_kind {
            ChunkDemandKind::Load => {
                demand.needs_residency = true;
                demand.unload_since = None
            }
            ChunkDemandKind::Mesh => demand.needs_meshing = true,
            ChunkDemandKind::Unload => {
                demand.needs_residency = false;
                demand.unload_since.get_or_insert(curr_tick);
            }
            ChunkDemandKind::Save => demand.needs_saving = true,
        }
    }

    pub fn commit(&mut self, manager: &ChunkManager, player_pos: &ChunkPos, curr_tick: Tick) {
        for (&chunk_pos, demand) in &self.demanded {
            let has_chunk = manager.contains_chunk(&chunk_pos);
            let distance = chunk_pos.dist_euclid_2(player_pos);

            let mut push = |priority: usize, kind: ChunkTaskKind| {
                let key = ChunkTaskKey {
                    chunk_pos,
                    version: demand.version,
                    kind,
                };

                if self.active.insert(key) {
                    self.pending.push(ChunkTask { priority, key });
                }
            };

            if !has_chunk && demand.needs_residency {
                push(distance, ChunkTaskKind::Load);
            }

            if let Some(_chunk) = manager.get_chunk(&chunk_pos) {
                if demand.needs_meshing {
                    push(distance, ChunkTaskKind::Mesh);
                }

                if demand.needs_saving {
                    push(distance, ChunkTaskKind::Save);
                }
            }

            if has_chunk
                && !demand.needs_residency
                && demand
                    .unload_since
                    .is_some_and(|start| curr_tick.ticks_since(&start) >= self.config.unload_delay)
            {
                push(usize::MAX - distance, ChunkTaskKind::Unload);
            }
        }
    }

    #[must_use]
    pub fn branch(&mut self) -> Option<ChunkTask> {
        self.pending.pop()
    }

    pub fn merge(&mut self, task_key: &ChunkTaskKey) -> bool {
        self.active.remove(task_key);

        let Some(demand) = self.demanded.get_mut(&task_key.chunk_pos) else {
            return false;
        };

        let same_patch = task_key.version.eq_patch(&demand.version);
        let same_minor = task_key.version.eq_minor(&demand.version);
        let same_major = task_key.version.eq_major(&demand.version);

        let success = match task_key.kind {
            ChunkTaskKind::Load => true,
            ChunkTaskKind::Mesh if same_patch => {
                demand.needs_meshing = false;
                true
            }
            ChunkTaskKind::Unload if same_major => {
                demand.unload_since = None;
                true
            }
            ChunkTaskKind::Save if same_minor => {
                demand.needs_saving = false;
                true
            }
            _ => false,
        };

        if demand.is_idle() {
            self.demanded.remove(&task_key.chunk_pos);
        }

        success
    }
}

pub struct ChunkSchedulerConfig {
    unload_delay: Tick,
}

impl ChunkSchedulerConfig {
    pub const fn new(unload_delay: Tick) -> Self {
        Self { unload_delay }
    }
}

#[derive(Debug)]
pub struct ChunkDemand {
    needs_residency: bool,
    needs_meshing: bool,
    needs_saving: bool,
    unload_since: Option<Tick>,

    version: Version,
}

impl ChunkDemand {
    pub const fn new(version: Version) -> Self {
        Self {
            needs_residency: false,
            needs_meshing: false,
            needs_saving: false,
            unload_since: None,
            version,
        }
    }

    pub const fn is_idle(&self) -> bool {
        !self.needs_residency
            && !self.needs_meshing
            && !self.needs_saving
            && self.unload_since.is_none()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChunkDemandKind {
    Load,
    Mesh,
    Unload,
    Save,
}

#[derive(Debug)]
pub struct ChunkTask {
    priority: usize,

    key: ChunkTaskKey,
}

impl ChunkTask {
    pub const fn key(&self) -> ChunkTaskKey {
        self.key
    }
}

impl PartialEq for ChunkTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.key == other.key
    }
}

impl Eq for ChunkTask {}

impl PartialOrd for ChunkTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChunkTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.key.chunk_pos.cmp(&other.key.chunk_pos))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkTaskKey {
    chunk_pos: ChunkPos,
    version: Version,
    kind: ChunkTaskKind,
}

impl ChunkTaskKey {
    pub const fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    pub const fn version(&self) -> Version {
        self.version
    }

    pub const fn kind(&self) -> ChunkTaskKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkTaskKind {
    Load,
    Mesh,
    Unload,
    Save,
}

#[derive(Debug)]
pub enum ChunkTaskResult {
    Loaded { key: ChunkTaskKey, chunk: Chunk },
    Meshed { key: ChunkTaskKey, mesh: bool },
    Saved { key: ChunkTaskKey },
    Unloaded { key: ChunkTaskKey },
}

#[cfg(test)]
mod tests {
    use super::{ChunkScheduler, ChunkTask};
    use crate::chunks::scheduler::{
        ChunkDemandKind, ChunkSchedulerConfig, ChunkTaskKey, ChunkTaskKind,
    };
    use crate::chunks::{Chunk, ChunkManager};
    use crate::worlds::Tick;
    use voxt_core::prelude::{ChunkPos, Pos, Version};

    fn pos(x: i16) -> ChunkPos {
        ChunkPos::from_raw(x, 0, 0)
    }

    #[test]
    fn repeated_commit_does_not_duplicate_pending_work() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);

        manager.insert_chunk(Chunk::new(chunk_pos));

        let version = Version::new(1, 1, 0);

        scheduler.add(ChunkDemandKind::Mesh, chunk_pos, version, Tick::new(0));

        scheduler.commit(&manager, &chunk_pos, Tick::new(1));
        scheduler.commit(&manager, &chunk_pos, Tick::new(2));
        scheduler.commit(&manager, &chunk_pos, Tick::new(3));

        assert!(scheduler.branch().is_some());
        assert!(scheduler.branch().is_none());
    }

    #[test]
    fn multiple_demand_kinds_can_coexist() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);

        manager.insert_chunk(Chunk::new(chunk_pos));

        let version = Version::new(2, 3, 4);

        scheduler.add(ChunkDemandKind::Mesh, chunk_pos, version, Tick::new(0));
        scheduler.add(ChunkDemandKind::Save, chunk_pos, version, Tick::new(0));

        scheduler.commit(&manager, &chunk_pos, Tick::new(0));

        let first = scheduler.branch().unwrap();
        let second = scheduler.branch().unwrap();

        assert_ne!(first.key.kind, second.key.kind);
    }

    #[test]
    fn completing_a_task_clears_only_that_demand() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);
        manager.insert_chunk(Chunk::new(chunk_pos));

        let ver = Version::new(0, 0, 0);

        scheduler.add(ChunkDemandKind::Mesh, chunk_pos, ver, Tick::new(0));
        scheduler.add(ChunkDemandKind::Save, chunk_pos, ver, Tick::new(0));
        scheduler.commit(&manager, &chunk_pos, Tick::new(0));

        let mesh = scheduler.branch().unwrap();
        let save = scheduler.branch().unwrap();
        scheduler.merge(&mesh.key);

        scheduler.commit(&manager, &chunk_pos, Tick::new(2));
        assert!(scheduler.branch().is_none());
        scheduler.merge(&save.key);
        scheduler.commit(&manager, &chunk_pos, Tick::new(3));
        assert!(scheduler.branch().is_none());
    }

    #[test]
    fn later_demand_updates_chunk_version() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let chunk_pos = pos(0);

        scheduler.add(
            ChunkDemandKind::Load,
            chunk_pos,
            Version::new(1, 0, 0),
            Tick::new(0),
        );

        scheduler.add(
            ChunkDemandKind::Mesh,
            chunk_pos,
            Version::new(1, 4, 2),
            Tick::new(1),
        );

        let demand = scheduler.demanded.get(&chunk_pos).unwrap();

        assert_eq!(demand.version, Version::new(1, 4, 2));
    }

    #[test]
    fn newer_version_can_schedule_new_mesh_while_old_mesh_is_active() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);

        manager.insert_chunk(Chunk::new(chunk_pos));

        let old_version = Version::new(1, 1, 0);
        let new_version = Version::new(1, 2, 0);

        scheduler.add(ChunkDemandKind::Mesh, chunk_pos, old_version, Tick::new(0));
        scheduler.commit(&manager, &chunk_pos, Tick::new(0));

        let old_task = scheduler.branch().unwrap();

        scheduler.add(ChunkDemandKind::Mesh, chunk_pos, new_version, Tick::new(1));
        scheduler.commit(&manager, &chunk_pos, Tick::new(1));

        let new_task = scheduler.branch().unwrap();

        assert_eq!(old_task.key.version, old_version);
        assert_eq!(new_task.key.version, new_version);
        assert_ne!(old_task.key, new_task.key);
    }

    #[test]
    fn stale_mesh_completion_does_not_clear_newer_demand() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);

        manager.insert_chunk(Chunk::new(chunk_pos));

        let old_version = Version::new(1, 1, 0);
        let new_version = Version::new(1, 2, 0);

        scheduler.add(ChunkDemandKind::Mesh, chunk_pos, old_version, Tick::new(0));
        scheduler.commit(&manager, &chunk_pos, Tick::new(0));

        let old_task = scheduler.branch().unwrap();

        scheduler.add(ChunkDemandKind::Mesh, chunk_pos, new_version, Tick::new(1));

        scheduler.merge(&old_task.key);

        assert!(scheduler.demanded.get(&chunk_pos).unwrap().needs_meshing);
    }

    #[test]
    fn unload_is_not_scheduled_for_nonresident_chunk() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let manager = ChunkManager::new();
        let chunk_pos = pos(0);

        scheduler.add(
            ChunkDemandKind::Unload,
            chunk_pos,
            Version::new(1, 0, 0),
            Tick::new(0),
        );

        scheduler.commit(&manager, &chunk_pos, Tick::new(10));

        assert!(scheduler.branch().is_none());
    }

    #[test]
    fn unload_waits_for_hysteresis() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);

        manager.insert_chunk(Chunk::new(chunk_pos));

        let version = Version::new(1, 0, 0);

        scheduler.add(ChunkDemandKind::Load, chunk_pos, version, Tick::new(0));

        scheduler.add(ChunkDemandKind::Unload, chunk_pos, version, Tick::new(5));

        scheduler.commit(&manager, &chunk_pos, Tick::new(14));
        assert!(scheduler.branch().is_none());

        scheduler.commit(&manager, &chunk_pos, Tick::new(15));

        assert!(matches!(
            scheduler.branch(),
            Some(ChunkTask {
                key: ChunkTaskKey {
                    kind: ChunkTaskKind::Unload,
                    ..
                },
                ..
            })
        ));
    }

    #[test]
    fn load_completion_does_not_cancel_pending_unload() {
        let mut scheduler = ChunkScheduler::new(ChunkSchedulerConfig::new(Tick::new(10)));
        let mut manager = ChunkManager::new();
        let chunk_pos = pos(0);
        let version = Version::new(0, 0, 0);

        scheduler.add(ChunkDemandKind::Load, chunk_pos, version, Tick::new(0));
        scheduler.commit(&manager, &chunk_pos, Tick::new(0));
        let load = scheduler.branch().unwrap();

        scheduler.add(ChunkDemandKind::Unload, chunk_pos, version, Tick::new(1));
        scheduler.merge(&load.key());
        manager.insert_chunk(Chunk::new(chunk_pos));

        scheduler.commit(&manager, &chunk_pos, Tick::new(10));
        assert!(scheduler.branch().is_none());

        scheduler.commit(&manager, &chunk_pos, Tick::new(11));
        assert!(matches!(
            scheduler.branch(),
            Some(ChunkTask {
                key: ChunkTaskKey {
                    kind: ChunkTaskKind::Unload,
                    ..
                },
                ..
            })
        ));
    }
}
