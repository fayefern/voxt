use voxt_core::prelude::{
    AtChunk, BlockId, Pos, VoxelPos,
    constants::{AIR, CHUNK_SIZE_LOG},
};

use crate::chunks::Chunk;

/// A `Chunk` is a 3D grid of voxels of width `Chunk::WIDTH`.
///
/// A `ChunkNaive` is a generalized version of a `Chunk` that holds `Chunk::WIDTH` voxels in the naive layout.
/// It also contains the opaque and transparent information for each block and the amount of non-air blocks of the chunk.
#[derive(Debug, Clone)]
pub struct ChunkNaive {
    data: Box<[BlockId; Chunk::VOL]>,
    occupancy: u16,
}

impl ChunkNaive {
    /// Creates a new `ChunkNaive` with the given blocks.
    ///
    /// If blocks is empty, this is equivalent to `new`
    #[must_use]
    pub fn new(blocks: &[AtChunk]) -> Self {
        let mut chunk = Self {
            data: Box::new([AIR; Chunk::VOL]),
            occupancy: 0,
        };

        let _ = chunk.set_bulk_naive(blocks);

        chunk
    }

    pub const fn occupancy(&self) -> u16 {
        self.occupancy
    }

    /// Gets the block at the given position. This is a helper function for the `get` method
    #[must_use]
    pub fn get_naive(&self, pos: VoxelPos) -> AtChunk {
        AtChunk::new(pos, self.data[naive(pos) as usize])
    }

    /// Gets the blocks at the given positions in bulk. This is a helper function for the `get_bulk` method
    #[must_use]
    pub fn get_bulk_naive(&self, poss: &[VoxelPos]) -> Box<[AtChunk]> {
        poss.iter()
            .map(|&pos| AtChunk::new(pos, self.data[naive(pos) as usize]))
            .collect()
    }

    /// Sets the block at the given position. This is a helper function for the `set` method
    #[must_use]
    pub fn set_naive(&mut self, block: AtChunk) -> AtChunk {
        let pos = block.pos();
        let new_id = block.id();

        let old_id = std::mem::replace(&mut self.data[naive(pos) as usize], new_id);

        if !old_id.is_equal(&new_id) {
            if new_id.is_air() {
                self.occupancy -= 1
            } else {
                self.occupancy += 1
            }
        }

        AtChunk::new(pos, old_id)
    }

    /// Sets the blocks at the given positions in bulk. This is a helper function for the `set_bulk` method
    #[must_use]
    pub fn set_bulk_naive(&mut self, blocks: &[AtChunk]) -> Box<[AtChunk]> {
        blocks
            .iter()
            .map(|vox| {
                let pos = vox.pos();
                let new_id = vox.id();

                let old_id = std::mem::replace(&mut self.data[naive(pos) as usize], new_id);

                if !old_id.is_equal(&new_id) {
                    if new_id.is_air() {
                        self.occupancy -= 1;
                    } else {
                        self.occupancy += 1;
                    }
                }

                AtChunk::new(pos, old_id)
            })
            .collect()
    }

    pub const fn is_empty(&self) -> bool {
        self.occupancy == 0
    }
}

/// A chunk is a 32x32x32 section of the world, this function turns `VoxelPos` into a 1D index. For now this is the only form of indexing into a `Chunk`'s contents.
#[must_use]
pub fn naive(pos: VoxelPos) -> u16 {
    INDEX[pos.x() as usize][pos.y() as usize][pos.z() as usize]
}

/// A lookup table for converting 3D positions to packed 1D indices. The coordinates are assumed to be within the bounds of the `Chunk`.
static INDEX: [[[u16; Chunk::WIDTH]; Chunk::WIDTH]; Chunk::WIDTH] = {
    let mut table = [[[0; Chunk::WIDTH]; Chunk::WIDTH]; Chunk::WIDTH];

    let mut x = 0;

    while x < Chunk::WIDTH {
        let mut y = 0;

        while y < Chunk::WIDTH {
            let mut z = 0;

            while z < Chunk::WIDTH {
                table[x][y][z] = (z as u16)
                    | ((x as u16) << CHUNK_SIZE_LOG)
                    | ((y as u16) << (2 * CHUNK_SIZE_LOG));
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }

    table
};
