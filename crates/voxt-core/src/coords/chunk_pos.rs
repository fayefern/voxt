use std::{
    fmt::Display,
    hash::Hash,
    ops::{Add, RangeInclusive, Sub},
};

use crate::{
    constants::{CHUNK_SIZE, WORLD_HEIGHT_IN_CHUNKS, WORLD_SIZE_IN_CHUNKS},
    coords::{Pos, VoxelPos, WorldPos},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkPos {
    x: i16,
    y: i16,
    z: i16,
}

impl ChunkPos {
    #[must_use]
    pub fn from_world_pos(world_pos: &WorldPos) -> Self {
        Self {
            x: world_pos.x().div_euclid(CHUNK_SIZE as i32) as i16,
            y: world_pos.y().div_euclid(CHUNK_SIZE as i32) as i16,
            z: world_pos.z().div_euclid(CHUNK_SIZE as i32) as i16,
        }
    }

    #[must_use]
    pub fn into_world_pos(&self, voxel_pos: &VoxelPos) -> WorldPos {
        Pos::from_raw(
            voxel_pos.x() as i32 + (self.x as i32 * CHUNK_SIZE as i32),
            voxel_pos.y() as i32 + (self.y as i32 * CHUNK_SIZE as i32),
            voxel_pos.z() as i32 + (self.z as i32 * CHUNK_SIZE as i32),
        )
    }
}

impl Pos for ChunkPos {
    type T = i16;

    const MIN: Self::T = -((WORLD_SIZE_IN_CHUNKS >> 1) as i16);
    const MAX: Self::T = (WORLD_SIZE_IN_CHUNKS >> 1) as i16 - 1;
    const RANGE: RangeInclusive<Self::T> = Self::MIN..=Self::MAX;

    const MIN_HEIGHT: Self::T = -((WORLD_HEIGHT_IN_CHUNKS >> 1) as i16);
    const MAX_HEIGHT: Self::T = (WORLD_HEIGHT_IN_CHUNKS >> 1) as i16 - 1;
    const RANGE_HEIGHT: RangeInclusive<Self::T> = Self::MIN_HEIGHT..=Self::MAX_HEIGHT;

    fn from_raw(x: Self::T, y: Self::T, z: Self::T) -> Self {
        Self { x, y, z }
    }

    fn pos(&self) -> (Self::T, Self::T, Self::T) {
        (self.x, self.y, self.z)
    }
}

impl From<WorldPos> for ChunkPos {
    fn from(world_pos: WorldPos) -> Self {
        Self::from_world_pos(&world_pos)
    }
}

impl From<&WorldPos> for ChunkPos {
    fn from(world_pos: &WorldPos) -> Self {
        Self::from_world_pos(world_pos)
    }
}

impl From<(i16, i16, i16)> for ChunkPos {
    fn from((x, y, z): (i16, i16, i16)) -> Self {
        Self::from_raw(x, y, z)
    }
}

impl From<[i16; 3]> for ChunkPos {
    fn from([x, y, z]: [i16; 3]) -> Self {
        Self::from_raw(x, y, z)
    }
}

impl From<ChunkPos> for (i16, i16, i16) {
    fn from(chunk_pos: ChunkPos) -> (i16, i16, i16) {
        (chunk_pos.x, chunk_pos.y, chunk_pos.z)
    }
}

impl From<ChunkPos> for [i16; 3] {
    fn from(chunk_pos: ChunkPos) -> [i16; 3] {
        [chunk_pos.x, chunk_pos.y, chunk_pos.z]
    }
}

impl Add for ChunkPos {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::checked_add(&self, &rhs)
    }
}

impl Sub for ChunkPos {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::checked_sub(&self, &rhs)
    }
}

impl Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl Hash for ChunkPos {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let packed: u32 = ((self.x as u32 & 0x3FFF) << 18)
            | ((self.y as u32 & 0xF) << 14)
            | (self.z as u32 & 0x3FFF);

        packed.hash(state);
    }
}
