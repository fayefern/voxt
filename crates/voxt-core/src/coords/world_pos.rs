use std::{
    fmt::Display,
    ops::{Add, RangeInclusive, Sub},
};

use crate::{
    constants::{CHUNK_SIZE, WORLD_HEIGHT, WORLD_SIZE},
    coords::{ChunkPos, Pos, VoxelPos},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldPos {
    x: i32,
    y: i32,
    z: i32,
}

impl WorldPos {
    #[must_use]
    pub fn into_voxel_pos(&self) -> VoxelPos {
        Pos::from_raw(
            self.x.rem_euclid(CHUNK_SIZE as i32) as u8,
            self.y.rem_euclid(CHUNK_SIZE as i32) as u8,
            self.z.rem_euclid(CHUNK_SIZE as i32) as u8,
        )
    }

    #[must_use]
    pub fn into_chunk_pos(&self) -> ChunkPos {
        Pos::from_raw(
            self.x.div_euclid(CHUNK_SIZE as i32) as i16,
            self.y.div_euclid(CHUNK_SIZE as i32) as i16,
            self.z.div_euclid(CHUNK_SIZE as i32) as i16,
        )
    }

    #[must_use]
    pub fn into_chunk_and_voxel_pos(&self) -> (ChunkPos, VoxelPos) {
        let voxel_pos = Pos::from_raw(
            self.x.rem_euclid(CHUNK_SIZE as i32) as u8,
            self.y.rem_euclid(CHUNK_SIZE as i32) as u8,
            self.z.rem_euclid(CHUNK_SIZE as i32) as u8,
        );

        let chunk_pos = Pos::from_raw(
            self.x.div_euclid(CHUNK_SIZE as i32) as i16,
            self.y.div_euclid(CHUNK_SIZE as i32) as i16,
            self.z.div_euclid(CHUNK_SIZE as i32) as i16,
        );

        (chunk_pos, voxel_pos)
    }

    #[must_use]
    pub fn from_chunk_and_voxel_pos(chunk_pos: &ChunkPos, voxel_pos: &VoxelPos) -> Self {
        Self {
            x: voxel_pos.x() as i32 + chunk_pos.x() as i32 * CHUNK_SIZE as i32,
            y: voxel_pos.y() as i32 + chunk_pos.y() as i32 * CHUNK_SIZE as i32,
            z: voxel_pos.z() as i32 + chunk_pos.z() as i32 * CHUNK_SIZE as i32,
        }
    }
}

impl Pos for WorldPos {
    type T = i32;

    const MIN: Self::T = -((WORLD_SIZE >> 1) as i32);
    const MAX: Self::T = (WORLD_SIZE >> 1) as i32 - 1;
    const RANGE: RangeInclusive<Self::T> = Self::MIN..=Self::MAX;

    const MIN_HEIGHT: Self::T = -((WORLD_HEIGHT >> 1) as i32);
    const MAX_HEIGHT: Self::T = (WORLD_HEIGHT >> 1) as i32 - 1;
    const RANGE_HEIGHT: RangeInclusive<Self::T> = Self::MIN_HEIGHT..=Self::MAX_HEIGHT;

    fn from_raw(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn pos(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

impl From<(ChunkPos, VoxelPos)> for WorldPos {
    fn from((chunk_pos, voxel_pos): (ChunkPos, VoxelPos)) -> Self {
        Self::from_chunk_and_voxel_pos(&chunk_pos, &voxel_pos)
    }
}

impl From<(&ChunkPos, &VoxelPos)> for WorldPos {
    fn from((chunk_pos, voxel_pos): (&ChunkPos, &VoxelPos)) -> Self {
        Self::from_chunk_and_voxel_pos(chunk_pos, voxel_pos)
    }
}

impl From<WorldPos> for (ChunkPos, VoxelPos) {
    fn from(world_pos: WorldPos) -> Self {
        world_pos.into_chunk_and_voxel_pos()
    }
}

impl From<&WorldPos> for (ChunkPos, VoxelPos) {
    fn from(world_pos: &WorldPos) -> Self {
        world_pos.into_chunk_and_voxel_pos()
    }
}

impl From<(i32, i32, i32)> for WorldPos {
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        Self::from_raw(x, y, z)
    }
}

impl From<[i32; 3]> for WorldPos {
    fn from([x, y, z]: [i32; 3]) -> Self {
        Self::from_raw(x, y, z)
    }
}

impl From<WorldPos> for (i32, i32, i32) {
    fn from(world_pos: WorldPos) -> (i32, i32, i32) {
        (world_pos.x, world_pos.y, world_pos.z)
    }
}

impl From<WorldPos> for [i32; 3] {
    fn from(world_pos: WorldPos) -> [i32; 3] {
        [world_pos.x, world_pos.y, world_pos.z]
    }
}

impl Add for WorldPos {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::checked_add(&self, &rhs)
    }
}

impl Sub for WorldPos {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::checked_sub(&self, &rhs)
    }
}

impl Display for WorldPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w[{}, {}, {}]", self.x, self.y, self.z)
    }
}
