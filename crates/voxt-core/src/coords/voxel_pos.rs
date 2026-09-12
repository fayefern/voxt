use std::{
    fmt::Display,
    ops::{Add, RangeInclusive, Sub},
};

use crate::{
    constants::CHUNK_SIZE,
    coords::{ChunkPos, Pos, WorldPos},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VoxelPos {
    x: u8,
    y: u8,
    z: u8,
}

impl VoxelPos {
    #[must_use]
    pub fn from_world_pos(world_pos: &WorldPos) -> Self {
        Self {
            x: world_pos.x().rem_euclid(CHUNK_SIZE as i32) as u8,
            y: world_pos.y().rem_euclid(CHUNK_SIZE as i32) as u8,
            z: world_pos.z().rem_euclid(CHUNK_SIZE as i32) as u8,
        }
    }

    #[must_use]
    pub fn into_world_pos(&self, chunk_pos: &ChunkPos) -> WorldPos {
        Pos::from_raw(
            self.x as i32 + (chunk_pos.x() as i32 * CHUNK_SIZE as i32),
            self.y as i32 + (chunk_pos.y() as i32 * CHUNK_SIZE as i32),
            self.z as i32 + (chunk_pos.z() as i32 * CHUNK_SIZE as i32),
        )
    }
}

impl Pos for VoxelPos {
    type T = u8;

    const MIN: Self::T = 0;
    const MAX: Self::T = CHUNK_SIZE as u8 - 1;
    const RANGE: RangeInclusive<Self::T> = Self::MIN..=Self::MAX;

    const MIN_HEIGHT: Self::T = 0;
    const MAX_HEIGHT: Self::T = CHUNK_SIZE as u8 - 1;
    const RANGE_HEIGHT: RangeInclusive<Self::T> = Self::MIN_HEIGHT..=Self::MAX_HEIGHT;

    fn from_raw(x: Self::T, y: Self::T, z: Self::T) -> Self {
        Self { x, y, z }
    }

    fn pos(&self) -> (Self::T, Self::T, Self::T) {
        (self.x, self.y, self.z)
    }
}

impl From<WorldPos> for VoxelPos {
    fn from(world_pos: WorldPos) -> Self {
        Self::from_world_pos(&world_pos)
    }
}

impl From<&WorldPos> for VoxelPos {
    fn from(world_pos: &WorldPos) -> Self {
        Self::from_world_pos(world_pos)
    }
}

impl From<(u8, u8, u8)> for VoxelPos {
    fn from((x, y, z): (u8, u8, u8)) -> Self {
        Self::from_raw(x, y, z)
    }
}

impl From<[u8; 3]> for VoxelPos {
    fn from([x, y, z]: [u8; 3]) -> Self {
        Self::from_raw(x, y, z)
    }
}

impl From<VoxelPos> for (u8, u8, u8) {
    fn from(voxel_pos: VoxelPos) -> Self {
        (voxel_pos.x, voxel_pos.y, voxel_pos.z)
    }
}

impl From<VoxelPos> for [u8; 3] {
    fn from(voxel_pos: VoxelPos) -> Self {
        [voxel_pos.x, voxel_pos.y, voxel_pos.z]
    }
}

impl Add for VoxelPos {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::checked_add(&self, &rhs)
    }
}

impl Sub for VoxelPos {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::checked_sub(&self, &rhs)
    }
}

impl Display for VoxelPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v[{}, {}, {}]", self.x, self.y, self.z)
    }
}
