use voxt_core::prelude::{
    BatVoxel, ChunkPos, ChunkVersion, VoxelPos,
    constants::{AIR, CHUNK_SIZE},
};

use crate::chunks::naive_chunk::ChunkNaive;

#[derive(Debug)]
pub struct Chunk {
    chunk_pos: ChunkPos,
    version: ChunkVersion,

    data: ChunkData,
}

impl Chunk {
    /// Length of a `Chunk` in voxels.
    pub const WIDTH: usize = CHUNK_SIZE;

    /// Area of a `Chunk` in voxels.
    pub const AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

    /// Volume of a `Chunk` in voxels.
    pub const VOL: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

    /// Initializes an empty chunk at the provided chunk-grid position.
    #[must_use]
    pub fn new(chunk_pos: ChunkPos) -> Self {
        Self {
            chunk_pos,
            version: ChunkVersion::new(0),
            data: ChunkData::new(),
        }
    }

    /// Initializes a chunk with the provided voxel blocks.
    ///
    /// Use [`Self::new`] to create an empty chunk. Block positions must belong to
    /// this chunk and duplicate positions are applied in slice order.
    #[must_use]
    pub fn new_with(chunk_pos: ChunkPos, blocks: &[BatVoxel]) -> Self {
        Self {
            chunk_pos,
            version: ChunkVersion::new(0),
            data: ChunkData::new_with(blocks),
        }
    }

    /// Returns the number of non-air voxels in the chunk.
    pub const fn non_air(&self) -> u16 {
        self.data.non_air_helper()
    }

    /// Returns the chunk's position in the chunk grid.
    #[must_use]
    pub const fn chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    /// Returns the chunk version, which increments once for each mutating operation.
    #[must_use]
    pub const fn version(&self) -> ChunkVersion {
        self.version
    }

    /// Returns the block at the given position.
    #[must_use]
    pub fn get(&self, pos: VoxelPos) -> BatVoxel {
        self.data.get_helper(pos)
    }

    /// Reads blocks at the requested positions into `output` in the same order.
    ///
    /// The caller must provide an output slice with at least one element for each
    /// requested position. An empty request leaves `output` unchanged.
    pub fn get_bulk(&self, poss: &[VoxelPos], output: &mut [BatVoxel]) {
        if !poss.is_empty() {
            self.data.get_bulk_helper(poss, output);
        }
    }

    /// Replaces a block and returns its previous value.
    ///
    /// The chunk version increments once when the block ID changes.
    pub fn set(&mut self, block: BatVoxel) -> BatVoxel {
        let (res, changed) = self.data.set_helper(block);

        if changed {
            self.version.increment();
        }

        res
    }

    /// Replaces blocks in request order and writes their previous values to `output`.
    ///
    /// The caller must provide an output slice with at least one element for each
    /// requested block. Duplicate positions are applied sequentially, so each
    /// request observes the result of earlier requests in the same slice. The
    /// chunk version increments at most once for the whole operation.
    pub fn set_bulk(&mut self, blocks: &[BatVoxel], output: &mut [BatVoxel]) {
        if !blocks.is_empty() && self.data.set_bulk_helper(blocks, output) {
            self.version.increment();
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
    fn new_with(blocks: &[BatVoxel]) -> Self {
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
            Self::General(c) => c.occupancy(),
        }
    }

    /// Returns the block at the given position.
    fn get_helper(&self, pos: VoxelPos) -> BatVoxel {
        match self {
            Self::Empty => BatVoxel::new(pos, AIR),
            Self::General(c) => c.get_naive(pos),
        }
    }

    fn get_bulk_helper(&self, poss: &[VoxelPos], output: &mut [BatVoxel]) {
        match self {
            Self::Empty => {
                for (&pos, out) in poss.iter().zip(output.iter_mut()) {
                    *out = BatVoxel::new(pos, AIR);
                }
            }
            Self::General(c) => c.get_bulk_naive(poss, output),
        }
    }

    fn set_helper(&mut self, block: BatVoxel) -> (BatVoxel, bool) {
        match self {
            Self::Empty => {
                let at_chunk = block;

                let pos = at_chunk.pos();
                let new_id = at_chunk.id();

                let old_block = BatVoxel::new(pos, AIR);

                if !new_id.is_air() {
                    *self = Self::General(ChunkNaive::new(&[BatVoxel::new(pos, new_id)]));

                    (old_block, true)
                } else {
                    (old_block, false)
                }
            }
            Self::General(c) => {
                let new_id = block.id();
                let old_block = c.set_naive(block);

                if c.is_empty() {
                    *self = Self::Empty;
                }

                if !new_id.is_equal(&old_block.id()) {
                    (old_block, true)
                } else {
                    (old_block, false)
                }
            }
        }
    }

    fn set_bulk_helper(&mut self, blocks: &[BatVoxel], output: &mut [BatVoxel]) -> bool {
        match self {
            Self::Empty => {
                let mut data = ChunkNaive::new(&[]);
                let changed = data.set_bulk_naive(blocks, output);

                if !data.is_empty() {
                    *self = Self::General(data);
                }

                changed
            }
            Self::General(data) => {
                let changed = data.set_bulk_naive(blocks, output);

                if data.is_empty() {
                    *self = Self::Empty;
                }

                changed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::Chunk;
    use voxt_core::{
        bat,
        prelude::{BatVoxel, BlockId, ChunkPos, Pos, VoxelPos, constants::AIR},
    };

    fn output_for(blocks: &[BatVoxel]) -> Vec<BatVoxel> {
        blocks
            .iter()
            .map(|block| BatVoxel::new(block.pos(), AIR))
            .collect()
    }

    #[test]
    fn new_chunk_is_empty() {
        let chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        assert_eq!(chunk.chunk_pos().clone(), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(chunk.version().as_u64(), 0);
    }

    #[test]
    fn new_chunk_contains_only_air() {
        let chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = [
            VoxelPos::from_raw(0, 0, 0),
            VoxelPos::from_raw(1, 2, 3),
            VoxelPos::from_raw(30, 30, 30),
            VoxelPos::from_raw(15, 20, 7),
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
            bat!( Ck
                1,2,3 => 1,
                30,30,30 => 2,
            ),
        );

        assert_eq!(chunk.get(VoxelPos::from_raw(1, 2, 3)).id(), BlockId::new(1));

        assert_eq!(
            chunk.get(VoxelPos::from_raw(30, 30, 30)).id(),
            BlockId::new(2)
        );

        assert_eq!(chunk.get(VoxelPos::from_raw(0, 0, 0)).id(), AIR);
    }

    #[test]
    fn setting_one_voxel_does_not_change_neighbors() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let target = VoxelPos::from_raw(5, 6, 7);
        let other = VoxelPos::from_raw(5, 6, 8);

        let stone = BlockId::new(1);

        let _ = chunk.set(BatVoxel::new(target, stone));

        assert_eq!(chunk.get(target).id(), stone);
        assert_eq!(chunk.get(other).id(), AIR);
    }

    #[test]
    fn set_returns_previous_block() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw(3, 4, 5);
        let stone = BlockId::new(1);
        let dirt = BlockId::new(2);

        let old = chunk.set(BatVoxel::new(pos, stone));

        assert_eq!(old.pos(), pos);
        assert_eq!(old.id(), AIR);

        let old = chunk.set(BatVoxel::new(pos, dirt));

        assert_eq!(old.pos(), pos);
        assert_eq!(old.id(), stone);

        assert_eq!(chunk.get(pos).id(), dirt);
    }

    #[test]
    fn air_to_block_and_back_to_air_works() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw(10, 20, 30);
        let stone = BlockId::new(1);

        assert_eq!(chunk.get(pos).id(), AIR);

        let _ = chunk.set(BatVoxel::new(pos, stone));
        assert_eq!(chunk.get(pos).id(), stone);

        let _ = chunk.set(BatVoxel::new(pos, AIR));
        assert_eq!(chunk.get(pos).id(), AIR);
    }

    #[test]
    fn setting_same_block_is_a_no_op() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw(1, 1, 1);
        let stone = BlockId::new(1);

        let _ = chunk.set(BatVoxel::new(pos, stone));
        let version_after_change = chunk.version();

        let old = chunk.set(BatVoxel::new(pos, stone));

        assert_eq!(old.id(), stone);
        assert_eq!(chunk.version(), version_after_change);
        assert_eq!(chunk.get(pos).id(), stone);
    }

    #[test]
    fn mutation_increments_version_once() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw(1, 2, 3);
        let stone = BlockId::new(1);

        assert_eq!(chunk.version().as_u64(), 0);

        let _ = chunk.set(BatVoxel::new(pos, stone));

        assert_eq!(chunk.version().as_u64(), 1);

        let _ = chunk.set(BatVoxel::new(pos, BlockId::new(2)));

        assert_eq!(chunk.version().as_u64(), 2);
    }

    #[test]
    fn changing_back_to_air_is_still_a_mutation() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let pos = VoxelPos::from_raw(1, 2, 3);
        let stone = BlockId::new(1);

        let _ = chunk.set(BatVoxel::new(pos, stone));
        assert_eq!(chunk.version().as_u64(), 1);

        let _ = chunk.set(BatVoxel::new(pos, AIR));
        assert_eq!(chunk.version().as_u64(), 2);
    }

    #[test]
    fn all_chunk_corners_are_addressable() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let corners = [
            VoxelPos::from_raw(0, 0, 0),
            VoxelPos::from_raw(0, 0, 31),
            VoxelPos::from_raw(0, 31, 0),
            VoxelPos::from_raw(0, 31, 31),
            VoxelPos::from_raw(31, 0, 0),
            VoxelPos::from_raw(31, 0, 31),
            VoxelPos::from_raw(31, 31, 0),
            VoxelPos::from_raw(31, 31, 31),
        ];

        for (i, pos) in corners.into_iter().enumerate() {
            let block = BlockId::new((i + 1) as u16);

            let _ = chunk.set(BatVoxel::new(pos, block));
        }

        for (i, pos) in corners.into_iter().enumerate() {
            assert_eq!(chunk.get(pos).id(), BlockId::new((i + 1) as u16));
        }
    }

    #[test]
    fn distinct_positions_remain_distinct() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = [
            VoxelPos::from_raw(0, 0, 0),
            VoxelPos::from_raw(1, 0, 0),
            VoxelPos::from_raw(0, 1, 0),
            VoxelPos::from_raw(0, 0, 1),
            VoxelPos::from_raw(31, 0, 0),
            VoxelPos::from_raw(0, 31, 0),
            VoxelPos::from_raw(0, 0, 31),
            VoxelPos::from_raw(31, 31, 31),
        ];

        for (i, pos) in positions.into_iter().enumerate() {
            let _ = chunk.set(BatVoxel::new(pos, BlockId::new((i + 1) as u16)));
        }

        for (i, pos) in positions.into_iter().enumerate() {
            assert_eq!(chunk.get(pos).id(), BlockId::new((i + 1) as u16));
        }
    }

    #[test]
    fn get_bulk_preserves_requested_order() {
        let chunk = Chunk::new_with(
            ChunkPos::from_raw(0, 0, 0),
            bat!( Ck
            1,2,3 => 1,
            4,5,6 => 2,
            7,8,9 => 3,
            ),
        );

        let requested = [
            VoxelPos::from_raw(7, 8, 9),
            VoxelPos::from_raw(1, 2, 3),
            VoxelPos::from_raw(4, 5, 6),
        ];

        let mut blocks = requested
            .iter()
            .map(|&pos| BatVoxel::new(pos, AIR))
            .collect::<Vec<_>>();
        chunk.get_bulk(&requested, &mut blocks);

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
            VoxelPos::from_raw(0, 0, 0),
            VoxelPos::from_raw(10, 20, 30),
            VoxelPos::from_raw(31, 31, 31),
        ];

        let mut blocks = positions
            .iter()
            .map(|&pos| BatVoxel::new(pos, AIR))
            .collect::<Vec<_>>();
        chunk.get_bulk(&positions, &mut blocks);

        assert_eq!(blocks.len(), positions.len());

        for (block, &pos) in blocks.iter().zip(positions.iter()) {
            assert_eq!(block.pos(), pos);
            assert_eq!(block.id(), AIR);
        }
    }

    #[test]
    fn empty_get_bulk_request_leaves_output_unchanged() {
        let chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));
        let pos = VoxelPos::from_raw(1, 2, 3);
        let mut output = vec![BatVoxel::new(pos, BlockId::new(7))];

        chunk.get_bulk(&[], &mut output);

        assert_eq!(output, [BatVoxel::new(pos, BlockId::new(7))]);
    }

    #[test]
    fn set_bulk_returns_previous_blocks() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = &[
            VoxelPos::from_raw(1, 2, 3),
            VoxelPos::from_raw(4, 5, 6),
            VoxelPos::from_raw(7, 8, 9),
        ];

        let blocks = &[BlockId::new(1), BlockId::new(2), BlockId::new(3)];

        let requests = BatVoxel::new_bulk(positions, blocks);
        let mut output = output_for(&requests);
        chunk.set_bulk(&requests, &mut output);

        assert_eq!(output.len(), 3);

        for (old_block, &pos) in output.iter().zip(positions.iter()) {
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

        let positions = &[VoxelPos::from_raw(1, 2, 3), VoxelPos::from_raw(4, 5, 6)];

        let first = &[BlockId::new(1), BlockId::new(2)];

        let second = &[BlockId::new(3), BlockId::new(4)];

        let first_blocks = BatVoxel::new_bulk(positions, first);
        let mut first_output = output_for(&first_blocks);
        chunk.set_bulk(&first_blocks, &mut first_output);

        let second_blocks = BatVoxel::new_bulk(positions, second);
        let mut second_output = output_for(&second_blocks);
        chunk.set_bulk(&second_blocks, &mut second_output);

        assert_eq!(second_output[0].id(), first[0]);
        assert_eq!(second_output[1].id(), first[1]);

        assert_eq!(chunk.get(positions[0]).id(), second[0]);
        assert_eq!(chunk.get(positions[1]).id(), second[1]);
    }

    #[test]
    fn set_bulk_respects_position_order() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let a = VoxelPos::from_raw(1, 2, 3);
        let b = VoxelPos::from_raw(10, 20, 30);
        let c = VoxelPos::from_raw(31, 31, 31);

        let positions = &[c, a, b];
        let blocks = &[BlockId::new(3), BlockId::new(1), BlockId::new(2)];

        let blocks = BatVoxel::new_bulk(positions, blocks);
        let mut output = output_for(&blocks);
        chunk.set_bulk(&blocks, &mut output);

        assert_eq!(chunk.get(c).id(), BlockId::new(3));
        assert_eq!(chunk.get(a).id(), BlockId::new(1));
        assert_eq!(chunk.get(b).id(), BlockId::new(2));
    }

    #[test]
    fn bulk_setting_air_keeps_chunk_effectively_empty() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = &[VoxelPos::from_raw(1, 2, 3), VoxelPos::from_raw(4, 5, 6)];

        let blocks = &[AIR, AIR];

        let requests = BatVoxel::new_bulk(positions, blocks);
        let mut output = output_for(&requests);
        chunk.set_bulk(&requests, &mut output);

        assert_eq!(output[0].id(), AIR);
        assert_eq!(output[1].id(), AIR);

        for &pos in positions {
            assert_eq!(chunk.get(pos).id(), AIR);
        }
    }

    #[test]
    fn bulk_setting_blocks_then_air_restores_air_state() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let positions = &[VoxelPos::from_raw(1, 2, 3), VoxelPos::from_raw(4, 5, 6)];

        let blocks = BatVoxel::new_bulk(positions, &[BlockId::new(1), BlockId::new(2)]);
        let mut output = output_for(&blocks);
        chunk.set_bulk(&blocks, &mut output);

        let blocks = BatVoxel::new_bulk(positions, &[AIR, AIR]);
        let mut output = output_for(&blocks);
        chunk.set_bulk(&blocks, &mut output);

        for &pos in positions {
            assert_eq!(chunk.get(pos).id(), AIR);
        }
    }

    #[test]
    fn set_bulk_applies_duplicate_positions_in_order() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));
        let pos = VoxelPos::from_raw(10, 20, 30);
        let requests = bat!(
            Ck 10,20,30 => 1,
            10,20,30 => 2,
        );
        let mut output = output_for(requests);

        chunk.set_bulk(requests, &mut output);

        assert_eq!(output[0].id(), AIR);
        assert_eq!(output[1].id(), BlockId::new(1));
        assert_eq!(chunk.get(pos).id(), BlockId::new(2));
        assert_eq!(chunk.version().as_u64(), 1);
    }

    #[test]
    fn empty_set_bulk_request_is_a_no_op() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));
        let pos = VoxelPos::from_raw(1, 2, 3);
        let mut output = vec![BatVoxel::new(pos, BlockId::new(7))];

        chunk.set_bulk(&[], &mut output);

        assert_eq!(output, [BatVoxel::new(pos, BlockId::new(7))]);
        assert_eq!(chunk.version().as_u64(), 0);
        assert_eq!(chunk.get(pos).id(), AIR);
    }

    #[test]
    fn set_bulk_increments_version_once_for_multiple_changes() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let voxels = bat!( Ck
            1, 2, 3 => 1,
            4, 5, 6 => 2,
            7, 8, 9 => 3,
        );

        let voxels = voxels.to_vec();
        let mut output = output_for(&voxels);
        chunk.set_bulk(&voxels, &mut output);

        assert_eq!(chunk.version().as_u64(), 1);
    }

    #[test]
    fn repeated_identical_bulk_set_is_a_no_op() {
        let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));

        let voxels = bat!( Ck
            1, 2, 3 => 1,
            4, 5, 6 => 2,
        );

        let first = voxels.to_vec();
        let mut first_output = output_for(&first);
        chunk.set_bulk(&first, &mut first_output);
        assert_eq!(chunk.version().as_u64(), 1);

        let second = voxels.to_vec();
        let mut second_output = output_for(&second);
        chunk.set_bulk(&second, &mut second_output);
        assert_eq!(chunk.version().as_u64(), 1);
    }
}
