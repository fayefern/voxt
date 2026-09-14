use voxt_core::prelude::{
    BatVoxel, BlockId, Pos, VoxelPos,
    constants::{AIR, CHUNK_SIZE, CHUNK_SIZE_LOG},
};

/// Paletted and bit packed Chunk.
pub struct SmartChunk {
    /// Variable palette of existing unique BlockIds within a Chunk.
    palette: Vec<PaletteEntry>,

    /// Bit packed palette indices.
    data: Vec<u64>,
    /// Number of bits that it takes to represent a single palette index (either 4-8-16).
    step: usize,
    /// Number of palette indices that fit inside a single u64 (either 16-8-4).
    density: usize,
    /// (1 << step) - 1.
    mask: u64,
}

impl SmartChunk {
    /// Maximum amount of Blocks a Chunk may hold.
    const VOL: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
    /// Safety gap. Otherwise a Chunk might cycle between growing and shrinking it's palette continuously.
    const HYSTERESIS_MARGIN: usize = 4;

    pub fn new() -> Self {
        let mut palette = Vec::with_capacity(16);
        palette.push(PaletteEntry {
            id: AIR,
            count: Self::VOL as u16,
        });

        let step = 4;
        let density = 64 / step;
        let mask = (1 << step) - 1;

        let data = vec![0u64; (Self::VOL * step).div_ceil(64)];

        Self {
            palette,
            data,
            step,
            density,
            mask,
        }
    }

    pub fn new_with(bats: &[BatVoxel]) -> Self {
        let mut chunk = Self::new();

        for &bat in bats {
            chunk.set(bat.pos(), bat.id());
        }

        chunk
    }

    pub fn set(&mut self, pos: VoxelPos, block_id: BlockId) -> BlockId {
        let idx = Self::index(pos);
        let old_palette_idx = self.read(idx) as usize;
        let old_block_id = self.palette[old_palette_idx].id;

        if old_block_id == block_id {
            return old_block_id;
        }

        let new_palette_idx = self.get_or_grow_palette(block_id) as usize;
        self.palette[old_palette_idx].count -= 1;
        self.palette[new_palette_idx].count += 1;
        self.write(idx, new_palette_idx as u16);
        self.try_shrink_palette();

        old_block_id
    }

    pub fn get(&self, pos: VoxelPos) -> BlockId {
        self.palette[self.read(Self::index(pos)) as usize].id
    }

    fn get_or_grow_palette(&mut self, block_id: BlockId) -> u16 {
        if let Some(index) = self.palette.iter().position(|entry| entry.id == block_id) {
            return index as u16;
        }

        self.palette.push(PaletteEntry {
            id: block_id,
            count: 0,
        });

        let new_len = self.palette.len();
        if new_len > (1usize << self.step) && self.step < 16 {
            self.rebuild(self.step * 2);
        }

        self.palette.len() as u16 - 1
    }

    fn try_shrink_palette(&mut self) {
        let active_len = self.palette.iter().filter(|entry| entry.count != 0).count();
        let new_step = match self.step {
            16 if active_len <= (1 << 8) - Self::HYSTERESIS_MARGIN => 8,
            8 if active_len <= (1 << 4) - Self::HYSTERESIS_MARGIN => 4,
            _ => return,
        };

        self.rebuild(new_step);
    }

    fn rebuild(&mut self, new_step: usize) {
        let mut remap = vec![u16::MAX; self.palette.len()];
        let mut new_palette = Vec::with_capacity(self.palette.len());

        for (old_index, entry) in self.palette.iter().enumerate() {
            if entry.count != 0 || new_step > self.step {
                remap[old_index] = new_palette.len() as u16;
                new_palette.push(PaletteEntry {
                    id: entry.id,
                    count: entry.count,
                });
            }
        }

        let new_density = 64 / new_step;
        let new_mask = (1u64 << new_step) - 1;
        let mut new_data = vec![0u64; (Self::VOL * new_step).div_ceil(64)];

        for idx in 0..Self::VOL {
            let old_word = self.data[idx / self.density];
            let old_shift = (idx % self.density) * self.step;
            let old_palette_idx = ((old_word >> old_shift) & self.mask) as usize;
            let new_palette_idx = remap[old_palette_idx] as u64;
            let new_word = &mut new_data[idx / new_density];
            let new_shift = (idx % new_density) * new_step;
            *new_word |= (new_palette_idx & new_mask) << new_shift;
        }

        self.palette = new_palette;
        self.data = new_data;
        self.step = new_step;
        self.density = new_density;
        self.mask = new_mask;
    }

    fn write(&mut self, idx: usize, val: u16) {
        let word = &mut self.data[idx / self.density];
        let shift = (idx % self.density) * self.step;
        let packed = (val as u64 & self.mask) << shift;
        *word = (*word & !(self.mask << shift)) | packed;
    }

    fn read(&self, idx: usize) -> u16 {
        let word = self.data[idx / self.density];
        let shift = (idx % self.density) * self.step;
        ((word >> shift) & self.mask) as u16
    }

    #[inline]
    fn index(pos: VoxelPos) -> usize {
        let (x, y, z) = pos.pos();
        usize::from(z)
            | (usize::from(x) << CHUNK_SIZE_LOG)
            | (usize::from(y) << (2 * CHUNK_SIZE_LOG))
    }
}

/// Entry for a SmartChunk's palette.
pub struct PaletteEntry {
    /// References the BlockRegistry's BlockIds.
    id: BlockId,
    /// The amount of times such BlockId appears in this Chunk.
    count: u16,
}

#[cfg(test)]
mod tests {
    use super::SmartChunk;
    use voxt_core::prelude::{BlockId, Pos, VoxelPos, constants::AIR};

    fn position(index: usize) -> VoxelPos {
        VoxelPos::from_raw(
            ((index / 32) % 32) as u8,
            (index / (32 * 32)) as u8,
            (index % 32) as u8,
        )
    }

    #[test]
    fn reads_and_writes_every_packed_position() {
        let mut chunk = SmartChunk::new();

        for index in 0..SmartChunk::VOL {
            let block_id = BlockId::new((index % 3 + 1) as u16);
            assert_eq!(chunk.set(position(index), block_id), AIR);
        }

        for index in 0..SmartChunk::VOL {
            assert_eq!(
                chunk.get(position(index)),
                BlockId::new((index % 3 + 1) as u16)
            );
        }
    }

    #[test]
    fn palette_grows_and_shrinks_without_losing_values() {
        let mut chunk = SmartChunk::new();

        for id in 1..=257 {
            assert_eq!(chunk.set(position(id as usize), BlockId::new(id)), AIR);
        }
        assert_eq!(chunk.step, 16);

        for id in 12..=257 {
            chunk.set(position(id as usize), AIR);
        }
        assert_eq!(chunk.step, 4);

        for id in 1..=11 {
            assert_eq!(chunk.get(position(id as usize)), BlockId::new(id));
        }
        assert_eq!(chunk.get(position(12)), AIR);
    }

    #[test]
    fn new_with_applies_duplicate_positions_in_order() {
        let pos = VoxelPos::from_raw(3, 4, 5);
        let chunk = SmartChunk::new_with(&[
            voxt_core::prelude::BatVoxel::new(pos, BlockId::new(1)),
            voxt_core::prelude::BatVoxel::new(pos, BlockId::new(2)),
        ]);

        assert_eq!(chunk.get(pos), BlockId::new(2));
    }
}
