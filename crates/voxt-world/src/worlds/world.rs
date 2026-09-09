use crate::{chunks::ChunkManager, worlds::WorldCommand};

pub struct World {
    chunks: ChunkManager,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: ChunkManager::new(),
        }
    }

    pub fn chunks(&self) -> &ChunkManager {
        &self.chunks
    }

    pub fn chunks_mut(&mut self) -> &mut ChunkManager {
        &mut self.chunks
    }

    pub fn apply(&mut self, command: WorldCommand) {
        match command {
            WorldCommand::GetBlock(pos) => {}
            WorldCommand::SetBlock(pos, block) => {}
            WorldCommand::RemoveBlock(pos) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::{Chunk, World};
    use voxt_core::prelude::{BlockId, ChunkPos, ChunkVoxel, VoxelPos};

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
        assert_eq!(world.chunks().get_chunk(&pos).unwrap().chunk_pos(), pos);
    }

    #[test]
    fn world_can_mutate_chunk_manager() {
        let mut world = World::new();
        let chunk_pos = ChunkPos::from_raw(1, 2, 3);
        let voxel_pos = VoxelPos::from_raw_checked(4, 5, 6);
        let block = BlockId::new(42);

        world.chunks_mut().insert_chunk(Chunk::new(chunk_pos));

        let _ = world
            .chunks_mut()
            .get_chunk_mut(&chunk_pos)
            .unwrap()
            .set(ChunkVoxel::new(voxel_pos, block));

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
}
