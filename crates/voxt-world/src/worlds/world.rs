use std::collections::VecDeque;

use voxt_core::prelude::{BatVoxel, BatWorld, ChunkPos, WorldPos, constants::AIR};

use crate::{
    chunks::{ChunkManager, ChunkScheduler},
    worlds::{
        WorldRequest, WorldResponse,
        command::{FailureReason, InvalidReason, WorldCommand, WorldQuery},
    },
};

pub struct World {
    chunks: ChunkManager,
    schedule: ChunkScheduler,

    tick: Tick,

    commands: VecDeque<WorldCommand>,
    responses: VecDeque<WorldResponse>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    #[must_use]
    pub fn new() -> Self {
        Self {
            chunks: ChunkManager::new(),
            schedule: ChunkScheduler::new(Tick::new(100)),

            tick: Tick::default(),
            commands: VecDeque::new(),
            responses: VecDeque::new(),
        }
    }

    #[must_use]
    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn advance(&mut self) {
        self.tick.increase();

        self.process_commands();

        self.simulate();

        self.schedule();
    }

    #[must_use]
    pub fn chunks(&self) -> &ChunkManager {
        &self.chunks
    }

    pub fn chunks_mut(&mut self) -> &mut ChunkManager {
        &mut self.chunks
    }

    pub fn submit(&mut self, command: impl Into<WorldCommand>) {
        self.commands.push_back(command.into());
    }

    pub fn process_commands(&mut self) {
        const MAX_POPS: usize = 16;

        let count = self.commands.len().min(MAX_POPS);

        for _ in 0..count {
            if let Some(c) = self.commands.pop_front() {
                let res = self.apply_commands(c);

                self.responses.push_back(res);
            }
        }
    }

    pub fn apply_commands(&mut self, command: impl Into<WorldCommand>) -> WorldResponse {
        match command.into() {
            WorldCommand::Query(q) => self.apply_query(q),
            WorldCommand::Request(r) => self.apply_request(r),
        }
    }

    fn apply_query(&self, query: WorldQuery) -> WorldResponse {
        match query {
            WorldQuery::GetBlock(pos) => match self.get_block(pos) {
                Some(bat) => WorldResponse::ReturnBlock(bat),
                None => WorldResponse::Failure {
                    reason: FailureReason::ChunkNotInMemory,
                },
            },
            WorldQuery::GetBlockBulk(poss) => {
                if poss.is_empty() {
                    WorldResponse::Invalid {
                        reason: InvalidReason::EmptyBlockBulkRequest,
                    }
                } else {
                    match self.get_block_bulk(&poss) {
                        Some(bats) => WorldResponse::ReturnBlockBulk(bats),
                        None => WorldResponse::Failure {
                            reason: FailureReason::ChunkNotInMemory,
                        },
                    }
                }
            }
        }
    }

    fn apply_request(&mut self, request: WorldRequest) -> WorldResponse {
        match request {
            WorldRequest::SetBlock(bat) => match self.set_block(bat) {
                Some(bat) => WorldResponse::ReturnBlock(bat),
                None => WorldResponse::Failure {
                    reason: FailureReason::ChunkNotInMemory,
                },
            },
            WorldRequest::SetBlockBulk(bats) => {
                if bats.is_empty() {
                    WorldResponse::Invalid {
                        reason: InvalidReason::EmptyBlockBulkRequest,
                    }
                } else {
                    match self.set_block_bulk(&bats) {
                        Some(bats) => WorldResponse::ReturnBlockBulk(bats),
                        None => WorldResponse::Failure {
                            reason: FailureReason::ChunkNotInMemory,
                        },
                    }
                }
            }
        }
    }

    fn get_block(&self, pos: WorldPos) -> Option<BatWorld> {
        let (chunk_pos, voxel_pos) = pos.into();

        self.chunks
            .get_chunk(&chunk_pos)
            .map(|chunk| chunk.get(voxel_pos).into_world_bat(&chunk_pos))
    }

    fn set_block(&mut self, bat: BatWorld) -> Option<BatWorld> {
        let chunk_pos = bat.pos().into();
        let bat_voxel = bat.into();

        self.chunks
            .get_chunk_mut(&chunk_pos)
            .map(|chunk| chunk.set(bat_voxel).into_world_bat(&chunk_pos))
    }

    fn get_block_bulk(&self, poss: &[WorldPos]) -> Option<Box<[BatWorld]>> {
        if !poss.iter().all(|p| self.chunks.contains_chunk(&p.into())) {
            return None;
        }

        let mut order: Vec<usize> = (0..poss.len()).collect();
        order.sort_by_key(|&i| poss[i].into_chunk_pos());

        let mut answ = vec![None; poss.len()];

        for group in order.chunk_by(|&a, &b| ChunkPos::from(poss[a]) == ChunkPos::from(poss[b])) {
            let chunk_pos = poss[group[0]].into();
            let chunk = self.chunks.get_chunk(&chunk_pos).expect("chunk exists");

            let voxel_poss: Vec<_> = group.iter().map(|&i| (&poss[i]).into()).collect();
            let mut buffer = voxel_poss
                .iter()
                .map(|&pos| BatVoxel::new(pos, AIR))
                .collect::<Vec<_>>();

            chunk.get_bulk(&voxel_poss, &mut buffer);

            for (&idx, old) in group.iter().zip(buffer) {
                answ[idx] = Some(old.into_world_bat(&chunk_pos));
            }
        }

        Some(answ.into_iter().map(|b| b.expect("slots filled")).collect())
    }

    fn set_block_bulk(&mut self, bats: &[BatWorld]) -> Option<Box<[BatWorld]>> {
        if !bats
            .iter()
            .all(|b| self.chunks.contains_chunk(&b.pos().into()))
        {
            return None;
        }

        let mut order: Vec<usize> = (0..bats.len()).collect();
        order.sort_by_key(|&i| bats[i].pos().into_chunk_pos());

        let mut answ = vec![None; bats.len()];

        for group in
            order.chunk_by(|&a, &b| ChunkPos::from(bats[a].pos()) == ChunkPos::from(bats[b].pos()))
        {
            let chunk_pos = bats[group[0]].pos().into();
            let chunk = self.chunks.get_chunk_mut(&chunk_pos).expect("chunk exists");

            let voxel_bats: Vec<_> = group.iter().map(|&i| (&bats[i]).into()).collect();
            let mut buffer = voxel_bats.clone();

            chunk.set_bulk(&voxel_bats, &mut buffer);

            for (&idx, old) in group.iter().zip(buffer) {
                answ[idx] = Some(old.into_world_bat(&chunk_pos));
            }
        }

        Some(answ.into_iter().map(|b| b.expect("slots filled")).collect())
    }

    fn simulate(&mut self) {}

    fn schedule(&mut self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Tick(u64);

impl Tick {
    pub const fn new(tick: u64) -> Self {
        Self(tick)
    }

    pub const fn tick(&self) -> u64 {
        self.0
    }

    pub const fn increase(&mut self) {
        self.0 += 1;
    }

    pub const fn ticks_since(&self, other: &Self) -> Self {
        assert!(self.0 >= other.0, "negative tick");

        Self(self.0 - other.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::{Chunk, World},
        worlds::{
            WorldRequest, WorldResponse,
            command::{FailureReason, WorldQuery},
            world::Tick,
        },
    };
    use voxt_core::{
        bat,
        prelude::{BatVoxel, BatWorld, BlockId, ChunkPos, Pos, VoxelPos, WorldPos},
    };

    #[test]
    fn new_world_has_no_chunks() {
        let world = World::new();

        assert!(world.chunks().is_empty());
        assert_eq!(world.chunks().num_chunks(), 0);
    }

    #[test]
    fn world_can_access_chunk_manager() {
        let mut world = World::new();
        let pos = ChunkPos::from_raw(1, 2, 3);

        world.chunks_mut().insert_chunk(Chunk::new(pos));

        assert!(world.chunks().contains_chunk(&pos));
        assert_eq!(
            world.chunks().get_chunk(&pos).unwrap().chunk_pos().clone(),
            pos
        );
    }

    #[test]
    fn world_can_mutate_chunk_manager() {
        let mut world = World::new();
        let chunk_pos = ChunkPos::from_raw(1, 2, 3);
        let voxel_pos = VoxelPos::from_raw(4, 5, 6);
        let block = BlockId::new(42);

        world.chunks_mut().insert_chunk(Chunk::new(chunk_pos));

        let _ = world
            .chunks_mut()
            .get_chunk_mut(&chunk_pos)
            .unwrap()
            .set(BatVoxel::new(voxel_pos, block));

        assert_eq!(
            world
                .chunks()
                .get_chunk(&chunk_pos)
                .unwrap()
                .get(voxel_pos)
                .id(),
            block
        );
    }

    #[test]
    fn get_block_on_resident_and_on_missing() {
        let input = [
            WorldPos::from_raw(0, 0, 0),
            WorldPos::from_raw(35, 0, 0),
            WorldPos::from_raw(111, 111, 111),
        ];

        let expected = bat!(
            Wd 0,0,0 => 0,
            35,0,0 => 0,
        );

        let mut world = World::new();
        let chunk_pos0 = ChunkPos::from_raw(0, 0, 0);
        let chunk_pos1 = ChunkPos::from_raw(1, 0, 0);

        world.chunks_mut().insert_chunk(Chunk::new(chunk_pos0));
        world.chunks_mut().insert_chunk(Chunk::new(chunk_pos1));

        let res0 = world.apply_commands(WorldQuery::GetBlock(input[0]));
        let res1 = world.apply_commands(WorldQuery::GetBlock(input[1]));
        let res2 = world.apply_commands(WorldQuery::GetBlock(input[2]));

        assert_eq!(WorldResponse::ReturnBlock(expected[0]), res0);
        assert_eq!(WorldResponse::ReturnBlock(expected[1]), res1);
        assert_eq!(
            WorldResponse::Failure {
                reason: FailureReason::ChunkNotInMemory
            },
            res2
        );
    }

    #[test]
    fn set_block_on_resident_and_on_missing() {
        let input = bat!(
            Wd 0,0,0 => 1,
            35,60,37 => 2,
            283,202,1490 => 3
        );

        let expected = bat!(
            Wd 0,0,0 => 0,
            35,60,37 => 0,
        );

        let mut world = World::new();
        let chunk_pos0 = ChunkPos::from_raw(0, 0, 0);
        let chunk_pos1 = ChunkPos::from_raw(1, 1, 1);

        world.chunks_mut().insert_chunk(Chunk::new(chunk_pos0));
        world.chunks_mut().insert_chunk(Chunk::new(chunk_pos1));

        let res0 = world.apply_commands(WorldRequest::SetBlock(input[0]));
        let res1 = world.apply_commands(WorldRequest::SetBlock(input[1]));
        let res2 = world.apply_commands(WorldRequest::SetBlock(input[2]));

        assert_eq!(WorldResponse::ReturnBlock(expected[0]), res0);
        assert_eq!(WorldResponse::ReturnBlock(expected[1]), res1);
        assert_eq!(
            WorldResponse::Failure {
                reason: FailureReason::ChunkNotInMemory
            },
            res2
        );
    }

    #[test]
    fn get_block_bulk_on_resident_and_on_missing() {
        let start0 = bat!(
            Ck 0,0,0 => 10,
            1,1,1 => 11,
            2,2,2 => 12,
            3,3,3 => 13,
            4,4,4 => 14,
            5,5,5 => 15,
            6,6,6 => 16,
            7,7,7 => 17,
            8,8,8 => 18
        );
        let input0 = vec![
            WorldPos::from_raw(0, 0, 0),
            WorldPos::from_raw(1, 1, 1),
            WorldPos::from_raw(2, 2, 2),
        ]
        .into_boxed_slice();
        let input1 = vec![
            WorldPos::from_raw(3, 3, 3),
            WorldPos::from_raw(4, 4, 4),
            WorldPos::from_raw(5, 5, 5),
        ]
        .into_boxed_slice();
        let input2 = vec![
            WorldPos::from_raw(32, 32, 32),
            WorldPos::from_raw(33, 33, 33),
            WorldPos::from_raw(34, 34, 34),
        ]
        .into_boxed_slice();
        let expected0 = bat!(
            Wd 0,0,0 => 10,
            1,1,1 => 11,
            2,2,2 => 12,
        )
        .to_vec()
        .into_boxed_slice();
        let expected1 = bat!(
            Wd 3,3,3 => 13,
            4,4,4 => 14,
            5,5,5 => 15,
        )
        .to_vec()
        .into_boxed_slice();

        let mut world = World::new();
        let chunk_pos0 = ChunkPos::from_raw(0, 0, 0);

        world
            .chunks_mut()
            .insert_chunk(Chunk::new_with(chunk_pos0, start0));

        let res0 = world.apply_commands(WorldQuery::GetBlockBulk(input0));
        let res1 = world.apply_commands(WorldQuery::GetBlockBulk(input1));
        let res2 = world.apply_commands(WorldQuery::GetBlockBulk(input2));

        assert_eq!(WorldResponse::ReturnBlockBulk(expected0), res0);
        assert_eq!(WorldResponse::ReturnBlockBulk(expected1), res1);
        assert_eq!(
            WorldResponse::Failure {
                reason: FailureReason::ChunkNotInMemory
            },
            res2
        );
    }

    #[test]
    fn set_block_bulk_on_resident_and_on_missing() {
        let start = bat!(
            Ck 0,0,0 => 10,
            1,1,1 => 11,
            2,2,2 => 12,
            3,3,3 => 13,
            4,4,4 => 14,
            5,5,5 => 15,
            6,6,6 => 16,
            7,7,7 => 17,
            8,8,8 => 18
        );
        let input0 = bat!(
            Wd 0,0,0 => 0,
            1,1,1 => 1,
            2,2,2 => 2,
            3,3,3 => 3
        )
        .to_vec()
        .into_boxed_slice();
        let input1 = bat!(
            Wd 4,4,4 => 4,
            5,5,5 => 5,
            6,6,6 => 6
        )
        .to_vec()
        .into_boxed_slice();
        let input2 = bat!(
            Wd 7,7,7 => 7,
            8,8,8 => 8,
            99,99,99 => 9
        )
        .to_vec()
        .into_boxed_slice();
        let input3 =
            vec![WorldPos::from_raw(7, 7, 7), WorldPos::from_raw(8, 8, 8)].into_boxed_slice();
        let expected0 = bat!(
            Wd 0,0,0 => 10,
            1,1,1 => 11,
            2,2,2 => 12,
            3,3,3 => 13
        )
        .to_vec()
        .into_boxed_slice();
        let expected1 = bat!(
            Wd 4,4,4 => 14,
            5,5,5 => 15,
            6,6,6 => 16
        )
        .to_vec()
        .into_boxed_slice();
        let expected2 = bat!(
            Wd 7,7,7 => 17,
            8,8,8 => 18
        )
        .to_vec()
        .into_boxed_slice();

        let mut world = World::new();
        let chunk_pos = ChunkPos::from_raw(0, 0, 0);

        world
            .chunks_mut()
            .insert_chunk(Chunk::new_with(chunk_pos, start));

        let res0 = world.apply_commands(WorldRequest::SetBlockBulk(input0));
        let res1 = world.apply_commands(WorldRequest::SetBlockBulk(input1));
        let res2 = world.apply_commands(WorldRequest::SetBlockBulk(input2));

        assert_eq!(WorldResponse::ReturnBlockBulk(expected0), res0);
        assert_eq!(WorldResponse::ReturnBlockBulk(expected1), res1);
        assert_eq!(
            WorldResponse::Failure {
                reason: FailureReason::ChunkNotInMemory
            },
            res2
        );

        let res3 = world.apply_commands(WorldQuery::GetBlockBulk(input3));
        assert_eq!(WorldResponse::ReturnBlockBulk(expected2), res3);
    }

    #[test]
    fn request_order_is_preserved() {
        let input = bat!(
            Wd 0,0,0 => 0,
            32,1,1 => 1,
            63,2,2 => 2,
            3,3,3 => 3,
            4,4,4 => 4,
            39,5,5 => 5,
            50,6,6 => 6,
            70,7,7 => 7,
            70,7,7 => 8
        )
        .to_vec()
        .into_boxed_slice();
        let expected = bat!(
            Wd 0,0,0 => 0,
            32,1,1 => 0,
            63,2,2 => 0,
            3,3,3 => 0,
            4,4,4 => 0,
            39,5,5 => 0,
            50,6,6 => 0,
            70,7,7 => 0,
            70,7,7 => 7
        )
        .to_vec()
        .into_boxed_slice();

        let mut world = World::new();
        let chunk_pos0 = ChunkPos::from_raw(0, 0, 0);
        let chunk_pos1 = ChunkPos::from_raw(1, 0, 0);
        let chunk_pos2 = ChunkPos::from_raw(2, 0, 0);

        {
            let manager = world.chunks_mut();

            manager.insert_chunk(Chunk::new(chunk_pos0));
            manager.insert_chunk(Chunk::new(chunk_pos1));
            manager.insert_chunk(Chunk::new(chunk_pos2));
        }

        let res = world.apply_commands(WorldRequest::SetBlockBulk(input));

        assert_eq!(WorldResponse::ReturnBlockBulk(expected), res);
    }

    #[test]
    fn tick_monotonic_and_independent() {
        let mut world = World::new();

        assert_eq!(Tick::default(), world.tick());

        world.advance();
        world.advance();
        world.advance();

        assert_eq!(Tick::new(3), world.tick());

        world
            .chunks_mut()
            .insert_chunk(Chunk::new(ChunkPos::from_raw(0, 0, 0)));
        let input = BatWorld::new(WorldPos::from_raw(0, 0, 0), BlockId::new(1));
        let _ = world.apply_commands(WorldRequest::SetBlock(input));

        assert_eq!(Tick::new(3), world.tick());
    }

    #[test]
    fn responses_arrive_at_sent_order() {
        let mut world = World::new();

        world
            .chunks_mut()
            .insert_chunk(Chunk::new(ChunkPos::from_raw(0, 0, 0)));

        let input = bat!(
            Wd 0,0,0 => 1,
            1,1,1 => 2,
            2,2,2 => 3,
            4,4,4 => 4,
            4,4,4 => 5,
            5,5,5 => 6,
            6,6,6 => 7,
            7,7,7 => 8
        );
        let expected: Vec<_> = bat!(
            Wd 0,0,0 => 0,
            1,1,1 => 0,
            2,2,2 => 0,
            4,4,4 => 0,
            4,4,4 => 4,
            5,5,5 => 0,
            6,6,6 => 0,
            7,7,7 => 0
        )
        .iter()
        .map(|&b| WorldResponse::ReturnBlock(b))
        .collect();

        input
            .iter()
            .for_each(|&b| world.submit(WorldRequest::SetBlock(b)));

        world.advance();

        expected
            .into_iter()
            .zip(world.responses)
            .for_each(|(e, r)| assert_eq!(e, r));
    }
}
