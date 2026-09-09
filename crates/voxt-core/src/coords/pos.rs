use std::{
    error::Error,
    fmt::Display,
    hash::Hash,
    ops::{Add, Range, Sub},
};

use num_traits::{AsPrimitive, CheckedAdd, CheckedSub, FromPrimitive};

use crate::{
    constants::{
        CHUNK_SIZE, WORLD_HEIGHT, WORLD_HEIGHT_IN_CHUNKS, WORLD_SIZE, WORLD_SIZE_IN_CHUNKS,
        WORLD_SIZE_IN_CHUNKS_LOG,
    },
    coords::dir::Dir,
};

mod coord {
    pub trait Coord:
        Clone
        + Copy
        + num_traits::PrimInt
        + num_traits::AsPrimitive<isize>
        + num_traits::FromPrimitive
    {
    }
    impl Coord for u8 {}
    impl Coord for i16 {}
    impl Coord for i32 {}
}

pub trait Pos {
    type T: coord::Coord;
    type E: Error;

    const MIN: Self::T;
    const MAX: Self::T;

    const MIN_HEIGHT: Self::T;
    const MAX_HEIGHT: Self::T;

    fn from_raw(x: Self::T, y: Self::T, z: Self::T) -> Self;

    fn try_new(x: Self::T, y: Self::T, z: Self::T) -> Result<Self, Self::E>
    where
        Self: Sized;

    fn pos(&self) -> (Self::T, Self::T, Self::T);

    fn x(&self) -> Self::T {
        self.pos().0
    }

    fn y(&self) -> Self::T {
        self.pos().1
    }

    fn z(&self) -> Self::T {
        self.pos().2
    }

    fn range() -> Range<Self::T> {
        Self::MIN..Self::MAX
    }

    fn range_height() -> Range<Self::T> {
        Self::MIN_HEIGHT..Self::MAX_HEIGHT
    }

    fn neighbor(&self, dir: Dir) -> Option<Self>
    where
        Self: Sized,
    {
        let (dx, dy, dz) = dir.as_off_tuple();

        let rx: isize = self.x().as_() + dx as isize;
        let ry: isize = self.y().as_() + dy as isize;
        let rz: isize = self.z().as_() + dz as isize;

        let nx: Self::T = FromPrimitive::from_isize(rx)?;
        let ny: Self::T = FromPrimitive::from_isize(ry)?;
        let nz: Self::T = FromPrimitive::from_isize(rz)?;

        if Self::range().contains(&nx)
            && Self::range_height().contains(&ny)
            && Self::range().contains(&nz)
        {
            Some(Self::from_raw(nx, ny, nz))
        } else {
            None
        }
    }

    fn neighborhood(&self) -> [Option<Self>; 6]
    where
        Self: Sized,
    {
        Dir::LIST.map(|d| self.neighbor(d))
    }

    fn checked_add(&self, rhs: &Self) -> Option<Self>
    where
        Self: Sized,
    {
        let nx: Self::T = self.x().checked_add(&rhs.x())?;
        let ny: Self::T = self.y().checked_add(&rhs.y())?;
        let nz: Self::T = self.z().checked_add(&rhs.z())?;

        if Self::range().contains(&nx)
            && Self::range_height().contains(&ny)
            && Self::range().contains(&nz)
        {
            Some(Self::from_raw(nx, ny, nz))
        } else {
            None
        }
    }

    fn checked_sub(&self, rhs: &Self) -> Option<Self>
    where
        Self: Sized,
    {
        let nx: Self::T = self.x().checked_sub(&rhs.x())?;
        let ny: Self::T = self.y().checked_sub(&rhs.y())?;
        let nz: Self::T = self.z().checked_sub(&rhs.z())?;

        if Self::range().contains(&nx)
            && Self::range_height().contains(&ny)
            && Self::range().contains(&nz)
        {
            Some(Self::from_raw(nx, ny, nz))
        } else {
            None
        }
    }

    fn dist_euclid_2(&self, rhs: &Self) -> usize {
        let dx: isize = self.x().as_() - rhs.x().as_();
        let dy: isize = self.y().as_() - rhs.y().as_();
        let dz: isize = self.z().as_() - rhs.z().as_();

        (dx * dx) as usize + (dy * dy) as usize + (dz * dz) as usize
    }

    fn dist_manhattan(&self, rhs: &Self) -> usize {
        let dx: isize = self.x().as_() - rhs.x().as_();
        let dy: isize = self.y().as_() - rhs.y().as_();
        let dz: isize = self.z().as_() - rhs.z().as_();

        dx.unsigned_abs() + dy.unsigned_abs() + dz.unsigned_abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelPos {
    x: u8,
    y: u8,
    z: u8,
}

impl VoxelPos {
    pub const MAX: u8 = CHUNK_SIZE as u8;

    #[inline]
    #[must_use]
    pub fn from_raw(x: u8, y: u8, z: u8) -> Self {
        debug_assert!(x < Self::MAX, "x greater than voxel max ({})", Self::MAX);
        debug_assert!(y < Self::MAX, "y greater than voxel max ({})", Self::MAX);
        debug_assert!(z < Self::MAX, "z greater than voxel max ({})", Self::MAX);

        Self { x, y, z }
    }

    #[must_use]
    pub fn try_new(x: u8, y: u8, z: u8) -> Result<Self, PosError> {
        if x < Self::MAX && y < Self::MAX && z < Self::MAX {
            Ok(Self { x, y, z })
        } else {
            Err(PosError::VoxelCoordOutofBounds(x, y, z))
        }
    }

    #[inline]
    #[must_use]
    pub const fn pos(&self) -> (u8, u8, u8) {
        (self.x, self.y, self.z)
    }

    #[must_use]
    pub const fn from_world_pos(world_pos: &WorldPos) -> Self {
        Self {
            x: world_pos.x.rem_euclid(Self::MAX as i32) as u8,
            y: world_pos.y.rem_euclid(Self::MAX as i32) as u8,
            z: world_pos.z.rem_euclid(Self::MAX as i32) as u8,
        }
    }

    #[must_use]
    pub const fn into_world_pos(&self, chunk_pos: &ChunkPos) -> WorldPos {
        WorldPos {
            x: self.x as i32 + (chunk_pos.x as i32 * Self::MAX as i32),
            y: self.y as i32 + (chunk_pos.y as i32 * Self::MAX as i32),
            z: self.z as i32 + (chunk_pos.z as i32 * Self::MAX as i32),
        }
    }
}

impl Pos for VoxelPos {
    type T = u8;
    type E = PosError;

    const MIN: Self::T = 0;
    const MAX: Self::T = CHUNK_SIZE as u8;

    const MIN_HEIGHT: Self::T = 0;
    const MAX_HEIGHT: Self::T = CHUNK_SIZE as u8;

    fn from_raw(x: Self::T, y: Self::T, z: Self::T) -> Self {
        debug_assert!(Self::range().contains(&x), "x not in range of voxel max");
        debug_assert!(
            Self::range_height().contains(&y),
            "y not in range of voxel max"
        );
        debug_assert!(Self::range().contains(&z), "z not in range of voxel max");

        Self { x, y, z }
    }

    fn try_new(x: Self::T, y: Self::T, z: Self::T) -> Result<Self, Self::E>
    where
        Self: Sized,
    {
        if Self::range().contains(&x)
            && Self::range_height().contains(&y)
            && Self::range().contains(&z)
        {
            Ok(Self { x, y, z })
        } else {
            Err(PosError::VoxelCoordOutofBounds(x, y, z))
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    x: i16,
    y: i16,
    z: i16,
}

impl ChunkPos {
    #[inline]
    #[must_use]
    pub const fn pos(&self) -> (i16, i16, i16) {
        (self.x, self.y, self.z)
    }

    #[must_use]
    pub const fn from_world_pos(world_pos: &WorldPos) -> Self {
        Self {
            x: world_pos.x.div_euclid(CHUNK_SIZE as i32) as i16,
            y: world_pos.y.div_euclid(CHUNK_SIZE as i32) as i16,
            z: world_pos.z.div_euclid(CHUNK_SIZE as i32) as i16,
        }
    }

    #[must_use]
    pub const fn into_world_pos(&self, voxel_pos: &VoxelPos) -> WorldPos {
        WorldPos {
            x: voxel_pos.x as i32 + (self.x as i32 * CHUNK_SIZE as i32),
            y: voxel_pos.y as i32 + (self.y as i32 * CHUNK_SIZE as i32),
            z: voxel_pos.z as i32 + (self.z as i32 * CHUNK_SIZE as i32),
        }
    }
}

impl Pos for ChunkPos {
    type T = i16;
    type E = PosError;

    const MIN: Self::T = -(WORLD_SIZE_IN_CHUNKS as i16);
    const MAX: Self::T = WORLD_SIZE_IN_CHUNKS as i16;

    const MIN_HEIGHT: Self::T = 0;
    const MAX_HEIGHT: Self::T = WORLD_HEIGHT_IN_CHUNKS as i16;

    fn from_raw(x: Self::T, y: Self::T, z: Self::T) -> Self {
        debug_assert!(Self::range().contains(&x), "x not in range of world max");
        debug_assert!(
            Self::range_height().contains(&y),
            "y not in range of world max"
        );
        debug_assert!(Self::range().contains(&z), "z not in range of world max");

        Self { x, y, z }
    }

    fn try_new(x: Self::T, y: Self::T, z: Self::T) -> Result<Self, Self::E>
    where
        Self: Sized,
    {
        if Self::range().contains(&x)
            && Self::range_height().contains(&y)
            && Self::range().contains(&z)
        {
            Ok(Self { x, y, z })
        } else {
            Err(PosError::ChunkCoordOutOfBounds(x, y, z))
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldPos {
    x: i32,
    y: i32,
    z: i32,
}

impl WorldPos {
    #[must_use]
    pub const fn into_voxel_pos(&self) -> VoxelPos {
        VoxelPos {
            x: self.x.rem_euclid(CHUNK_SIZE as i32) as u8,
            y: self.y.rem_euclid(CHUNK_SIZE as i32) as u8,
            z: self.z.rem_euclid(CHUNK_SIZE as i32) as u8,
        }
    }

    #[must_use]
    pub const fn into_chunk_pos(&self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(CHUNK_SIZE as i32) as i16,
            y: self.y.div_euclid(CHUNK_SIZE as i32) as i16,
            z: self.z.div_euclid(CHUNK_SIZE as i32) as i16,
        }
    }

    #[must_use]
    pub const fn from_chunk_voxel_pos(chunk_pos: &ChunkPos, voxel_pos: &VoxelPos) -> Self {
        Self {
            x: voxel_pos.x as i32 + chunk_pos.x as i32 * CHUNK_SIZE as i32,
            y: voxel_pos.y as i32 + chunk_pos.y as i32 * CHUNK_SIZE as i32,
            z: voxel_pos.z as i32 + chunk_pos.z as i32 * CHUNK_SIZE as i32,
        }
    }
}

impl Pos for WorldPos {
    type T = i32;
    type E = PosError;

    const MIN: Self::T = -(WORLD_SIZE as i32);
    const MAX: Self::T = WORLD_SIZE as i32;

    const MIN_HEIGHT: Self::T = 0;
    const MAX_HEIGHT: Self::T = WORLD_HEIGHT as i32;

    fn from_raw(x: i32, y: i32, z: i32) -> Self {
        debug_assert!(Self::range().contains(&x), "x not in range of world max");
        debug_assert!(
            Self::range_height().contains(&y),
            "y not in range of world max"
        );
        debug_assert!(Self::range().contains(&z), "z not in range of world max");

        Self { x, y, z }
    }

    fn try_new(x: i32, y: i32, z: i32) -> Result<Self, PosError> {
        if Self::range().contains(&x)
            && Self::range_height().contains(&y)
            && Self::range().contains(&z)
        {
            Ok(Self { x, y, z })
        } else {
            Err(PosError::WorldCoordOutOfBounds(x, y, z))
        }
    }

    fn pos(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

impl From<(ChunkPos, VoxelPos)> for WorldPos {
    fn from((chunk_pos, voxel_pos): (ChunkPos, VoxelPos)) -> Self {
        Self::from_chunk_voxel_pos(&chunk_pos, &voxel_pos)
    }
}

impl From<(&ChunkPos, &VoxelPos)> for WorldPos {
    fn from((chunk_pos, voxel_pos): (&ChunkPos, &VoxelPos)) -> Self {
        Self::from_chunk_voxel_pos(chunk_pos, voxel_pos)
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

#[derive(Debug, Clone, Copy)]
pub enum PosError {
    WorldCoordOutOfBounds(i32, i32, i32),
    ChunkCoordOutOfBounds(i16, i16, i16),
    VoxelCoordOutofBounds(u8, u8, u8),
}

impl Display for PosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorldCoordOutOfBounds(x, y, z) => {
                write!(
                    f,
                    "world coordinates ({}, {}, {}) are out of bounds",
                    x, y, z
                )
            }
            Self::ChunkCoordOutOfBounds(x, y, z) => {
                write!(
                    f,
                    "chunk coordinates ({}, {}, {}) are out of bounds",
                    x, y, z
                )
            }
            Self::VoxelCoordOutofBounds(x, y, z) => {
                write!(
                    f,
                    "voxel coordinates ({}, {}, {}) are out of bounds",
                    x, y, z
                )
            }
        }
    }
}

impl Error for PosError {}

#[cfg(test)]
mod tests {
    use crate::{
        constants::CHUNK_SIZE,
        coords::{ChunkPos, VoxelPos, WorldPos, pos::Pos},
    };

    #[test]
    fn from_raw_preserves_coordinates() {
        let pos = WorldPos::from_raw(1, -2, 3);

        assert_eq!(pos.pos(), (1, -2, 3));
        assert_eq!(pos.x(), 1);
        assert_eq!(pos.y(), -2);
        assert_eq!(pos.z(), 3);
    }

    #[test]
    fn tuple_conversion_preserves_coordinates() {
        let pos = WorldPos::from((1, 2, 3));

        assert_eq!(pos.pos(), (1, 2, 3));
    }

    #[test]
    fn array_conversion_preserves_coordinates() {
        let pos = WorldPos::from([1, 2, 3]);

        assert_eq!(pos.pos(), (1, 2, 3));
    }

    #[test]
    fn converts_to_tuple() {
        let pos = WorldPos::from_raw(1, 2, 3);

        let tuple: (i32, i32, i32) = pos.into();

        assert_eq!(tuple, (1, 2, 3));
    }

    #[test]
    fn converts_to_array() {
        let pos = WorldPos::from_raw(1, 2, 3);

        let array: [i32; 3] = pos.into();

        assert_eq!(array, [1, 2, 3]);
    }

    fn takes_chunk_pos(_: ChunkPos) {}

    fn takes_world_voxel_pos(_: WorldPos) {}

    #[test]
    fn position_aliases_are_usable_as_distinct_types() {
        let chunk = ChunkPos::from_raw(1, 2, 3);
        let world = WorldPos::from_raw(1, 2, 3);

        takes_chunk_pos(chunk);
        takes_world_voxel_pos(world);
    }

    #[test]
    fn try_new_accepts_valid_bounds() {
        assert!(VoxelPos::try_new(0, 0, 0).is_ok());
        assert!(VoxelPos::try_new(31, 31, 31).is_ok());
    }

    #[test]
    fn try_new_accepts_interior_coordinates() {
        let pos = VoxelPos::try_new(1, 2, 3).unwrap();

        assert_eq!(pos.pos(), (1, 2, 3));
    }

    #[test]
    fn try_new_rejects_invalid_x() {
        assert!(VoxelPos::try_new(32, 0, 0).is_err());
    }

    #[test]
    fn try_new_rejects_invalid_y() {
        assert!(VoxelPos::try_new(0, 32, 0).is_err());
    }

    #[test]
    fn try_new_rejects_invalid_z() {
        assert!(VoxelPos::try_new(0, 0, 32).is_err());
    }

    #[test]
    fn try_new_rejects_invalid_coordinates() {
        assert!(VoxelPos::try_new(32, 32, 32).is_err());
    }

    #[test]
    fn try_new_rejects_values_above_chunk_bounds() {
        assert!(VoxelPos::try_new(u8::MAX, 0, 0).is_err());
        assert!(VoxelPos::try_new(0, u8::MAX, 0).is_err());
        assert!(VoxelPos::try_new(0, 0, u8::MAX).is_err());
    }

    #[test]
    fn from_raw_does_not_modify_coordinates() {
        let pos = VoxelPos::from_raw(200, 201, 202);

        assert_eq!(pos.pos(), (200, 201, 202));
    }

    #[test]
    fn from_raw_accepts_valid_coordinates() {
        let pos = VoxelPos::from_raw(31, 10, 0);

        assert_eq!(pos.pos(), (31, 10, 0));
    }

    #[test]
    #[should_panic]
    fn from_raw_panics_on_invalid_x() {
        let _ = VoxelPos::from_raw(32, 0, 0);
    }

    #[test]
    #[should_panic]
    fn from_raw_panics_on_invalid_y() {
        let _ = VoxelPos::from_raw(0, 32, 0);
    }

    #[test]
    #[should_panic]
    fn from_raw_panics_on_invalid_z() {
        let _ = VoxelPos::from_raw(0, 0, 32);
    }

    #[test]
    fn world_origin_decomposes_correctly() {
        let world = WorldPos::from_raw(0, 0, 0);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw(0, 0, 0));
    }

    #[test]
    fn last_voxel_of_first_chunk_decomposes_correctly() {
        let world = WorldPos::from_raw(31, 31, 31);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw(31, 31, 31));
    }

    #[test]
    fn first_voxel_of_next_chunk_decomposes_correctly() {
        let world = WorldPos::from_raw(32, 32, 32);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(1, 1, 1));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw(0, 0, 0));
    }

    #[test]
    fn negative_one_decomposes_to_previous_chunk_last_voxel() {
        let world = WorldPos::from_raw(-1, -1, -1);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-1, -1, -1));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw(31, 31, 31));
    }

    #[test]
    fn negative_chunk_boundary_decomposes_correctly() {
        let world = WorldPos::from_raw(-32, -32, -32);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-1, -1, -1));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw(0, 0, 0));
    }

    #[test]
    fn negative_coordinate_just_beyond_chunk_boundary_decomposes_correctly() {
        let world = WorldPos::from_raw(-33, -33, -33);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-2, -2, -2));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw(31, 31, 31));
    }

    #[test]
    fn mixed_sign_coordinates_decompose_independently() {
        let world = WorldPos::from_raw(-1, 0, 32);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-1, 0, 1));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw(31, 0, 0));
    }

    #[test]
    fn coordinate_decomposition_round_trips() {
        let cases = [
            (0, 0, 0),
            (1, 2, 3),
            (31, 31, 31),
            (32, 32, 32),
            (33, 47, 65),
            (-1, -1, -1),
            (-32, -32, -32),
            (-33, -33, -33),
            (-65, 42, 97),
        ];

        for (x, y, z) in cases {
            let original = WorldPos::from_raw(x, y, z);

            let chunk = ChunkPos::from(original);
            let voxel = VoxelPos::from(original);

            let reconstructed = WorldPos::from((chunk, voxel));

            assert_eq!(reconstructed, original, "round-trip failed for {original}");
        }
    }

    #[test]
    fn world_conversion_always_produces_valid_voxel_positions() {
        for x in -100..=100 {
            for y in -100..=100 {
                for z in -100..=100 {
                    let world = WorldPos::from_raw(x, y, z);
                    let voxel = VoxelPos::from(world);

                    assert!(voxel.x() < CHUNK_SIZE as u8);
                    assert!(voxel.y() < CHUNK_SIZE as u8);
                    assert!(voxel.z() < CHUNK_SIZE as u8);
                }
            }
        }
    }

    #[test]
    fn chunk_pos_is_suitable_as_hash_map_key() {
        let mut chunks = std::collections::HashMap::new();

        chunks.insert(ChunkPos::from_raw(-4, 7, 12), "chunk");

        assert_eq!(chunks.get(&ChunkPos::from_raw(-4, 7, 12)), Some(&"chunk"));
    }
}
