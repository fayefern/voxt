use std::collections::HashMap;

use voxt_core::prelude::ChunkPos;

use crate::chunks::Chunk;

pub struct ChunkManager {
    map: HashMap<ChunkPos, Chunk>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_chunk(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.map.get(pos)
    }

    pub fn get_chunk_mut(&mut self, pos: &ChunkPos) -> Option<&mut Chunk> {
        self.map.get_mut(pos)
    }

    pub fn insert_chunk(&mut self, chunk: Chunk) -> Option<Chunk> {
        self.map.insert(chunk.chunk_pos(), chunk)
    }

    pub fn remove_chunk(&mut self, pos: &ChunkPos) -> Option<Chunk> {
        self.map.remove(pos)
    }

    pub fn contains_chunk(&self, pos: &ChunkPos) -> bool {
        self.map.contains_key(pos)
    }

    pub fn num_chunks(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use voxt_core::prelude::{BlockId, ChunkPos, ChunkVoxel, VoxelPos};

    use crate::chunks::{Chunk, ChunkManager};

    fn chunk(x: i32, y: i32, z: i32) -> Chunk {
        Chunk::new(ChunkPos::from_raw(x, y, z))
    }

    #[test]
    fn new_manager_is_empty() {
        let manager = ChunkManager::new();

        assert!(manager.is_empty());
        assert_eq!(manager.num_chunks(), 0);
    }

    #[test]
    fn insert_and_get_chunk() {
        let mut manager = ChunkManager::new();
        let chunk = chunk(1, 2, 3);
        let pos = chunk.chunk_pos();

        assert!(manager.insert_chunk(chunk).is_none());

        let stored = manager.get_chunk(&pos).unwrap();
        assert_eq!(stored.chunk_pos(), pos);
    }

    #[test]
    fn get_missing_chunk_returns_none() {
        let manager = ChunkManager::new();
        let pos = ChunkPos::from_raw(1, 2, 3);

        assert!(manager.get_chunk(&pos).is_none());
    }

    #[test]
    fn get_chunk_mut_allows_mutation() {
        let mut manager = ChunkManager::new();
        let pos = ChunkPos::from_raw(1, 2, 3);

        manager.insert_chunk(Chunk::new(pos));

        let chunk = manager.get_chunk_mut(&pos).unwrap();
        let _ = chunk.set(ChunkVoxel::new(
            VoxelPos::from_raw_checked(0, 0, 0),
            BlockId::new(1),
        ));

        assert_eq!(
            manager
                .get_chunk(&pos)
                .unwrap()
                .get(VoxelPos::from_raw_checked(0, 0, 0))
                .id(),
            BlockId::new(1)
        );
    }

    #[test]
    fn contains_chunk() {
        let mut manager = ChunkManager::new();
        let present = ChunkPos::from_raw(1, 2, 3);
        let absent = ChunkPos::from_raw(4, 5, 6);

        manager.insert_chunk(Chunk::new(present));

        assert!(manager.contains_chunk(&present));
        assert!(!manager.contains_chunk(&absent));
    }

    #[test]
    fn remove_chunk() {
        let mut manager = ChunkManager::new();
        let pos = ChunkPos::from_raw(1, 2, 3);

        manager.insert_chunk(Chunk::new(pos));

        let removed = manager.remove_chunk(&pos).unwrap();

        assert_eq!(removed.chunk_pos(), pos);
        assert!(!manager.contains_chunk(&pos));
        assert!(manager.is_empty());
    }

    #[test]
    fn remove_missing_chunk_returns_none() {
        let mut manager = ChunkManager::new();
        let pos = ChunkPos::from_raw(1, 2, 3);

        assert!(manager.remove_chunk(&pos).is_none());
    }

    #[test]
    fn inserting_same_position_replaces_chunk() {
        let mut manager = ChunkManager::new();
        let pos = ChunkPos::from_raw(1, 2, 3);

        let first = Chunk::new(pos);

        let mut second = Chunk::new(pos);
        let _ = second.set(ChunkVoxel::new(
            VoxelPos::from_raw_checked(0, 0, 0),
            BlockId::new(42),
        ));

        assert!(manager.insert_chunk(first).is_none());

        let replaced = manager.insert_chunk(second).unwrap();

        assert_eq!(replaced.chunk_pos(), pos);
        assert_eq!(replaced.version().as_u64(), 0);

        assert_eq!(manager.num_chunks(), 1);
        assert_eq!(
            manager
                .get_chunk(&pos)
                .unwrap()
                .get(VoxelPos::from_raw_checked(0, 0, 0))
                .id(),
            BlockId::new(42)
        );
    }

    #[test]
    fn num_chunks_tracks_resident_chunks() {
        let mut manager = ChunkManager::new();

        let a = ChunkPos::from_raw(0, 0, 0);
        let b = ChunkPos::from_raw(1, 0, 0);
        let c = ChunkPos::from_raw(2, 0, 0);

        assert_eq!(manager.num_chunks(), 0);

        manager.insert_chunk(Chunk::new(a));
        assert_eq!(manager.num_chunks(), 1);

        manager.insert_chunk(Chunk::new(b));
        assert_eq!(manager.num_chunks(), 2);

        manager.insert_chunk(Chunk::new(c));
        assert_eq!(manager.num_chunks(), 3);

        manager.remove_chunk(&b);
        assert_eq!(manager.num_chunks(), 2);

        manager.remove_chunk(&a);
        manager.remove_chunk(&c);

        assert_eq!(manager.num_chunks(), 0);
    }
}
