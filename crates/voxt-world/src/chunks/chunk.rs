use voxt_core::prelude::{
    BlockId, ChunkPos, ChunkVersion, VoxelPos,
    constants::{AIR, CHUNK_LOG_USZ, CHUNK_SIZE_USZ},
};

use crate::chunks::voxel::Voxel;

#[derive(Debug, Clone)]
pub struct Chunk {
    chunk_pos: ChunkPos,
    version: ChunkVersion,

    data: ChunkData,
}

impl Chunk {
    /// Length of a `Chunk` in voxels.
    pub const WIDTH: usize = CHUNK_SIZE_USZ;

    /// Area of a `Chunk` in voxels.
    pub const AREA: usize = CHUNK_SIZE_USZ * CHUNK_SIZE_USZ;

    /// Volume of a `Chunk` in voxels.
    pub const VOL: usize = CHUNK_SIZE_USZ * CHUNK_SIZE_USZ * CHUNK_SIZE_USZ;

    /// Convenience function. Initializes an empty `Chunk` at the provided position.
    #[must_use]
    pub fn new(chunk_pos: ChunkPos) -> Self {
        Self {
            chunk_pos,
            version: ChunkVersion::new(0),
            data: ChunkData::new(),
        }
    }

    /// Generic new function. Initializes a `Chunk` at the provided positions, with the provided blocks.
    ///
    /// Use `new` if intent on creating an empty `Chunk`
    /// If the slice is longer than the chunk volume, the extra elements are ignored.
    #[must_use]
    pub fn new_with(chunk_pos: ChunkPos, blocks: &[Voxel]) -> Self {
        Self {
            chunk_pos,
            version: ChunkVersion::new(0),
            data: ChunkData::new_with(blocks),
        }
    }

    pub const fn non_air(&self) -> u16 {
        self.data.non_air_helper()
    }

    /// Returns the position of the chunk in the Chunk Grid.
    #[must_use]
    pub const fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    /// Returns the version of the chunk. This is used to determine if the chunk has been modified.
    #[must_use]
    pub const fn version(&self) -> ChunkVersion {
        self.version
    }

    #[must_use]
    /// Returns the block at the given position.
    pub const fn get(&self, pos: VoxelPos) -> Voxel {
        self.data.get_helper(pos)
    }

    #[must_use]
    /// Returns the blocks at the given positions.
    pub fn get_bulk(&self, poss: &[VoxelPos]) -> Option<Box<[Voxel]>> {
        if poss.is_empty() {
            None
        } else {
            Some(self.data.get_bulk_helper(poss))
        }
    }

    #[must_use]
    /// Swaps the block at the given position with the provided block, returning the previous block
    pub fn set(&mut self, block: Voxel) -> Voxel {
        let (res, changed) = self.data.set_helper(block);

        if changed {
            self.version.increment();
        }

        res
    }

    #[must_use]
    /// Swaps the blocks at the given positions with the provided blocks, returning the previous blocks.
    ///
    /// May return None if the provided [Voxel] slice is empty.
    pub fn set_bulk(&mut self, blocks: &[Voxel]) -> Option<Box<[Voxel]>> {
        if blocks.is_empty() {
            None
        } else {
            let (res, changed) = self.data.set_bulk_helper(blocks);

            if changed {
                self.version.increment();
            }

            Some(res)
        }
    }
}

/// A `Chunk`'s blocks. This is a generic enum that can either be empty or contain a boxed slice of blocks.
///
/// More variants may be added, but the effect should not be felt upstream.
#[derive(Debug, Clone)]
enum ChunkData {
    Empty,
    General(ChunkNaive),
    //RLE?
    //Palette?
}

impl ChunkData {
    /// Creates a new `ChunkData`.
    /// This always returns `Empty`.
    #[must_use]
    const fn new() -> Self {
        Self::Empty
    }

    /// Creates a new `ChunkData` with the given blocks. For now what that means is that it will always be a `ChunkNaive`,
    /// unless the provided blocks are empty.
    #[must_use]
    fn new_with(blocks: &[Voxel]) -> Self {
        if blocks.is_empty() {
            Self::Empty
        } else {
            Self::General(ChunkNaive::new(blocks))
        }
    }

    /// Returns the number of blocks in the chunk.
    const fn non_air_helper(&self) -> u16 {
        match self {
            Self::Empty => 0,
            Self::General(c) => c.occupancy,
        }
    }

    /// Returns the block at the given position.
    const fn get_helper(&self, pos: VoxelPos) -> Voxel {
        match self {
            Self::Empty => Voxel::new(pos, AIR),
            Self::General(c) => c.get_naive(pos),
        }
    }

    fn get_bulk_helper(&self, poss: &[VoxelPos]) -> Box<[Voxel]> {
        match self {
            Self::Empty => poss.iter().map(|&p| Voxel::new(p, AIR)).collect(),
            Self::General(c) => c.get_bulk_naive(poss),
        }
    }

    fn set_helper(&mut self, block: Voxel) -> (Voxel, bool) {
        match self {
            Self::Empty => {
                let pos = block.pos();
                let new_id = block.id();

                let old_block = Voxel::new(pos, AIR);

                if !new_id.is_air() {
                    *self = Self::General(ChunkNaive::new(&[Voxel::new(pos, new_id)]));

                    (old_block, true)
                } else {
                    (old_block, false)
                }
            }
            Self::General(c) => {
                let old_block = c.set_naive(block);

                if c.is_empty() {
                    *self = Self::Empty;
                }

                if !block.id().is_equal(&old_block.id()) {
                    (old_block, true)
                } else {
                    (old_block, false)
                }
            }
        }
    }

    fn set_bulk_helper(&mut self, blocks: &[Voxel]) -> (Box<[Voxel]>, bool) {
        match self {
            Self::Empty => {
                let old_blocks = blocks
                    .iter()
                    .map(|vox| Voxel::new(vox.pos(), AIR))
                    .collect();

                let data = blocks
                    .iter()
                    .filter_map(|vox| {
                        if !vox.id().is_air() {
                            Some(Voxel::new(vox.pos(), vox.id()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                *self = Self::General(ChunkNaive::new(&data));

                if !data.is_empty() {
                    (old_blocks, true)
                } else {
                    (old_blocks, false)
                }
            }
            Self::General(data) => {
                let old_blocks = data.set_bulk_naive(blocks);

                if data.is_empty() {
                    *self = Self::Empty;
                }

                if old_blocks
                    .iter()
                    .zip(blocks.iter())
                    .any(|(vox_old, vox_new)| !vox_old.id().is_equal(&vox_new.id()))
                {
                    (old_blocks, true)
                } else {
                    (old_blocks, false)
                }
            }
        }
    }
}

/// A `Chunk` is a 3D grid of voxels of width `Chunk::WIDTH`.
///
/// A `ChunkNaive` is a generalized version of a `Chunk` that holds `Chunk::WIDTH` voxels in the naive layout.
/// It also contains the opaque and transparent information for each block and the amount of non-air blocks of the chunk.
#[derive(Debug, Clone)]
struct ChunkNaive {
    data: Box<[BlockId; Chunk::VOL]>,
    occupancy: u16,
}

impl ChunkNaive {
    /// Creates a new `ChunkNaive` with the given blocks.
    ///
    /// If blocks is empty, this is equivalent to `new`
    #[must_use]
    fn new(blocks: &[Voxel]) -> Self {
        let mut chunk = Self {
            data: Box::new([AIR; Chunk::VOL]),
            occupancy: 0,
        };

        let _ = chunk.set_bulk_naive(blocks);

        chunk
    }

    /// Gets the block at the given position. This is a helper function for the `get` method
    #[must_use]
    const fn get_naive(&self, pos: VoxelPos) -> Voxel {
        Voxel::new(pos, self.data[naive(pos) as usize])
    }

    /// Gets the blocks at the given positions in bulk. This is a helper function for the `get_bulk` method
    #[must_use]
    fn get_bulk_naive(&self, poss: &[VoxelPos]) -> Box<[Voxel]> {
        poss.iter()
            .map(|&pos| Voxel::new(pos, self.data[naive(pos) as usize]))
            .collect()
    }

    /// Sets the block at the given position. This is a helper function for the `set` method
    #[must_use]
    const fn set_naive(&mut self, block: Voxel) -> Voxel {
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

        Voxel::new(pos, old_id)
    }

    /// Sets the blocks at the given positions in bulk. This is a helper function for the `set_bulk` method
    #[must_use]
    fn set_bulk_naive(&mut self, blocks: &[Voxel]) -> Box<[Voxel]> {
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

                Voxel::new(pos, old_id)
            })
            .collect()
    }

    const fn is_empty(&self) -> bool {
        self.occupancy == 0
    }
}

/// A chunk is a 32x32x32 section of the world, this function turns `VoxelPos` into a 1D index. For now this is the only form of indexing into a `Chunk`'s contents.
#[must_use]
const fn naive(pos: VoxelPos) -> u16 {
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
                    | ((x as u16) << CHUNK_LOG_USZ)
                    | ((y as u16) << (2 * CHUNK_LOG_USZ));
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }

    table
};

// TESTS

#[cfg(test)]
mod tests {
    use crate::prelude::{Chunk, Voxel, vox};
    use voxt_core::prelude::{BlockId, ChunkPos, VoxelPos, constants::AIR};

    #[test]
    fn new_chunk_is_empty() {
        let chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        assert_eq!(chunk.chunk_pos(), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(chunk.version().as_u64(), 0);
    }

    #[test]
    fn new_chunk_contains_only_air() {
        let chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = [
            VoxelPos::from_raw_checked(0, 0, 0),
            VoxelPos::from_raw_checked(1, 2, 3),
            VoxelPos::from_raw_checked(31, 31, 31),
            VoxelPos::from_raw_checked(15, 20, 7),
        ];

        for pos in positions {
            assert_eq!(chunk.get(pos).id(), AIR);
            assert_eq!(chunk.get(pos).pos(), pos);
        }
    }

    #[test]
    fn new_with_places_blocks_at_requested_positions() {
        let chunk = Chunk::new_with(
            ChunkPos::from_raw(0, 0, 0),
            vox!(
                1,2,3 => 1,
                31,31,31 => 2,
            ),
        );

        assert_eq!(
            chunk.get(VoxelPos::from_raw_checked(1, 2, 3)).id(),
            BlockId::new(1)
        );

        assert_eq!(
            chunk.get(VoxelPos::from_raw_checked(31, 31, 31)).id(),
            BlockId::new(2)
        );

        assert_eq!(chunk.get(VoxelPos::from_raw_checked(0, 0, 0)).id(), AIR);
    }

    #[test]
    fn setting_one_voxel_does_not_change_neighbors() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let target = VoxelPos::from_raw_checked(5, 6, 7);
        let other = VoxelPos::from_raw_checked(5, 6, 8);

        let stone = BlockId::new(1);

        let _ = chunk.set(Voxel::new(target, stone));

        assert_eq!(chunk.get(target).id(), stone);
        assert_eq!(chunk.get(other).id(), AIR);
    }

    #[test]
    fn set_returns_previous_block() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw_checked(3, 4, 5);
        let stone = BlockId::new(1);
        let dirt = BlockId::new(2);

        let old = chunk.set(Voxel::new(pos, stone));

        assert_eq!(old.pos(), pos);
        assert_eq!(old.id(), AIR);

        let old = chunk.set(Voxel::new(pos, dirt));

        assert_eq!(old.pos(), pos);
        assert_eq!(old.id(), stone);

        assert_eq!(chunk.get(pos).id(), dirt);
    }

    #[test]
    fn air_to_block_and_back_to_air_works() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw_checked(10, 20, 30);
        let stone = BlockId::new(1);

        assert_eq!(chunk.get(pos).id(), AIR);

        let _ = chunk.set(Voxel::new(pos, stone));
        assert_eq!(chunk.get(pos).id(), stone);

        let _ = chunk.set(Voxel::new(pos, AIR));
        assert_eq!(chunk.get(pos).id(), AIR);
    }

    #[test]
    fn setting_same_block_is_a_no_op() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw_checked(1, 1, 1);
        let stone = BlockId::new(1);

        let _ = chunk.set(Voxel::new(pos, stone));
        let version_after_change = chunk.version();

        let old = chunk.set(Voxel::new(pos, stone));

        assert_eq!(old.id(), stone);
        assert_eq!(chunk.version(), version_after_change);
        assert_eq!(chunk.get(pos).id(), stone);
    }

    #[test]
    fn mutation_increments_version_once() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw_checked(1, 2, 3);
        let stone = BlockId::new(1);

        assert_eq!(chunk.version().as_u64(), 0);

        let _ = chunk.set(Voxel::new(pos, stone));

        assert_eq!(chunk.version().as_u64(), 1);

        let _ = chunk.set(Voxel::new(pos, BlockId::new(2)));

        assert_eq!(chunk.version().as_u64(), 2);
    }

    #[test]
    fn changing_back_to_air_is_still_a_mutation() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw_checked(1, 2, 3);
        let stone = BlockId::new(1);

        let _ = chunk.set(Voxel::new(pos, stone));
        assert_eq!(chunk.version().as_u64(), 1);

        let _ = chunk.set(Voxel::new(pos, AIR));
        assert_eq!(chunk.version().as_u64(), 2);
    }

    #[test]
    fn all_chunk_corners_are_addressable() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let corners = [
            VoxelPos::from_raw_checked(0, 0, 0),
            VoxelPos::from_raw_checked(0, 0, 31),
            VoxelPos::from_raw_checked(0, 31, 0),
            VoxelPos::from_raw_checked(0, 31, 31),
            VoxelPos::from_raw_checked(31, 0, 0),
            VoxelPos::from_raw_checked(31, 0, 31),
            VoxelPos::from_raw_checked(31, 31, 0),
            VoxelPos::from_raw_checked(31, 31, 31),
        ];

        for (i, pos) in corners.into_iter().enumerate() {
            let block = BlockId::new((i + 1) as u16);

            let _ = chunk.set(Voxel::new(pos, block));
        }

        for (i, pos) in corners.into_iter().enumerate() {
            assert_eq!(chunk.get(pos).id(), BlockId::new((i + 1) as u16));
        }
    }

    #[test]
    fn distinct_positions_remain_distinct() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = [
            VoxelPos::from_raw_checked(0, 0, 0),
            VoxelPos::from_raw_checked(1, 0, 0),
            VoxelPos::from_raw_checked(0, 1, 0),
            VoxelPos::from_raw_checked(0, 0, 1),
            VoxelPos::from_raw_checked(31, 0, 0),
            VoxelPos::from_raw_checked(0, 31, 0),
            VoxelPos::from_raw_checked(0, 0, 31),
            VoxelPos::from_raw_checked(31, 31, 31),
        ];

        for (i, pos) in positions.into_iter().enumerate() {
            let _ = chunk.set(Voxel::new(pos, BlockId::new((i + 1) as u16)));
        }

        for (i, pos) in positions.into_iter().enumerate() {
            assert_eq!(chunk.get(pos).id(), BlockId::new((i + 1) as u16));
        }
    }

    #[test]
    fn get_bulk_preserves_requested_order() {
        let chunk = Chunk::new_with(
            ChunkPos::from_raw(0, 0, 0),
            vox!(
            1,2,3 => 1,
            4,5,6 => 2,
            7,8,9 => 3,
            ),
        );

        let requested = [
            VoxelPos::from_raw_checked(7, 8, 9),
            VoxelPos::from_raw_checked(1, 2, 3),
            VoxelPos::from_raw_checked(4, 5, 6),
        ];

        let blocks = chunk.get_bulk(&requested).unwrap();

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].pos(), requested[0]);
        assert_eq!(blocks[1].pos(), requested[1]);
        assert_eq!(blocks[2].pos(), requested[2]);

        assert_eq!(blocks[0].id(), BlockId::new(3));
        assert_eq!(blocks[1].id(), BlockId::new(1));
        assert_eq!(blocks[2].id(), BlockId::new(2));
    }

    #[test]
    fn get_bulk_on_empty_chunk_returns_air() {
        let chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = [
            VoxelPos::from_raw_checked(0, 0, 0),
            VoxelPos::from_raw_checked(10, 20, 30),
            VoxelPos::from_raw_checked(31, 31, 31),
        ];

        let blocks = chunk.get_bulk(&positions).unwrap();

        assert_eq!(blocks.len(), positions.len());

        for (block, &pos) in blocks.iter().zip(positions.iter()) {
            assert_eq!(block.pos(), pos);
            assert_eq!(block.id(), AIR);
        }
    }

    #[test]
    fn set_bulk_returns_previous_blocks() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = &[
            VoxelPos::from_raw_checked(1, 2, 3),
            VoxelPos::from_raw_checked(4, 5, 6),
            VoxelPos::from_raw_checked(7, 8, 9),
        ][..];

        let blocks = &[BlockId::new(1), BlockId::new(2), BlockId::new(3)][..];

        let old = chunk.set_bulk(&Voxel::new_bulk(positions, blocks)).unwrap();

        assert_eq!(old.len(), 3);

        for (old_block, &pos) in old.iter().zip(positions.iter()) {
            assert_eq!(old_block.pos(), pos);
            assert_eq!(old_block.id(), AIR);
        }

        for (&pos, &block) in positions.iter().zip(blocks.iter()) {
            assert_eq!(chunk.get(pos).id(), block);
        }
    }

    #[test]
    fn set_bulk_returns_current_values_on_replacement() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = &[
            VoxelPos::from_raw_checked(1, 2, 3),
            VoxelPos::from_raw_checked(4, 5, 6),
        ][..];

        let first = &[BlockId::new(1), BlockId::new(2)][..];

        let second = &[BlockId::new(3), BlockId::new(4)][..];

        let _ = chunk.set_bulk(&Voxel::new_bulk(positions, first));

        let old = chunk.set_bulk(&Voxel::new_bulk(positions, second)).unwrap();

        assert_eq!(old[0].id(), first[0]);
        assert_eq!(old[1].id(), first[1]);

        assert_eq!(chunk.get(positions[0]).id(), second[0]);
        assert_eq!(chunk.get(positions[1]).id(), second[1]);
    }

    #[test]
    fn set_bulk_respects_position_order() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let a = VoxelPos::from_raw_checked(1, 2, 3);
        let b = VoxelPos::from_raw_checked(10, 20, 30);
        let c = VoxelPos::from_raw_checked(31, 31, 31);

        let positions = &[c, a, b][..];
        let blocks = &[BlockId::new(3), BlockId::new(1), BlockId::new(2)][..];

        let _ = chunk.set_bulk(&Voxel::new_bulk(positions, blocks));

        assert_eq!(chunk.get(c).id(), BlockId::new(3));
        assert_eq!(chunk.get(a).id(), BlockId::new(1));
        assert_eq!(chunk.get(b).id(), BlockId::new(2));
    }

    #[test]
    fn bulk_setting_air_keeps_chunk_effectively_empty() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = &[
            VoxelPos::from_raw_checked(1, 2, 3),
            VoxelPos::from_raw_checked(4, 5, 6),
        ][..];

        let blocks = &[AIR, AIR][..];

        let old = chunk.set_bulk(&Voxel::new_bulk(positions, blocks)).unwrap();

        assert_eq!(old[0].id(), AIR);
        assert_eq!(old[1].id(), AIR);

        for &pos in positions {
            assert_eq!(chunk.get(pos).id(), AIR);
        }
    }

    #[test]
    fn bulk_setting_blocks_then_air_restores_air_state() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = &[
            VoxelPos::from_raw_checked(1, 2, 3),
            VoxelPos::from_raw_checked(4, 5, 6),
        ][..];

        let _ = chunk.set_bulk(&Voxel::new_bulk(
            positions,
            &[BlockId::new(1), BlockId::new(2)][..],
        ));

        let _ = chunk.set_bulk(&Voxel::new_bulk(positions, &[AIR, AIR][..]));

        for &pos in positions {
            assert_eq!(chunk.get(pos).id(), AIR);
        }
    }

    #[test]
    fn set_bulk_increments_version_once_for_multiple_changes() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let voxels = vox!(
            VoxelPos::from_raw_checked(1, 2, 3) => BlockId::new(1),
            VoxelPos::from_raw_checked(4, 5, 6) => BlockId::new(2),
            VoxelPos::from_raw_checked(7, 8, 9) => BlockId::new(3),
        );

        let _ = chunk.set_bulk(voxels);

        assert_eq!(chunk.version().as_u64(), 1);
    }

    #[test]
    fn repeated_identical_bulk_set_is_a_no_op() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let voxels = vox!(
            VoxelPos::from_raw_checked(1, 2, 3) => BlockId::new(1),
            VoxelPos::from_raw_checked(4, 5, 6) => BlockId::new(2),
        );

        let _ = chunk.set_bulk(voxels);
        assert_eq!(chunk.version().as_u64(), 1);

        let _ = chunk.set_bulk(voxels);
        assert_eq!(chunk.version().as_u64(), 1);
    }
}
