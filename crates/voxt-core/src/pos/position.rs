use std::{
    fmt::Display,
    marker::PhantomData,
    ops::{Add, Sub},
};

use thiserror::Error;

use crate::prelude::constants::{CHUNK_SIZE_I32, CHUNK_SIZE_U8, WORLD_NEG_I32, WORLD_POS_I32};

/// PhantomData, indicates a `Pos` in the World Grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldSpace {}
/// Phantom Data, indicates a `Pos` in the Chunk Grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkSpace {}
/// Phantom Data, indicates a `Pos` in the Voxel Grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoxelSpace {}

/// A position in a 3D space. The `Space` generic parameter indicates the space the `Pos` is in.
///
/// Not meant to be used on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pos<T, Space> {
    x: T,
    y: T,
    z: T,

    _space: PhantomData<Space>,
}

impl<T, Space> Pos<T, Space> {
    /// Creates a new `Pos` with the given x, y, z.
    ///
    /// # Safety
    ///
    /// This might introduce bugs if used incorrectly.
    /// Use `try_new` instead if you are not certain that the coordinates are valid.
    #[must_use]
    pub const fn from_raw(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }

    /// Returns a tuple containing all three coordinates of the `Pos`.
    #[must_use]
    pub const fn all(&self) -> (T, T, T)
    where
        T: Copy,
    {
        (self.x, self.y, self.z)
    }

    /// Returns the x-coordinate of the `Pos`.
    #[must_use]
    pub const fn x(&self) -> T
    where
        T: Copy,
    {
        self.x
    }

    /// Returns the y-coordinate of the `Pos`.
    #[must_use]
    pub const fn y(&self) -> T
    where
        T: Copy,
    {
        self.y
    }

    /// Returns the z-coordinate of the `Pos`.
    #[must_use]
    pub const fn z(&self) -> T
    where
        T: Copy,
    {
        self.z
    }
}

impl<T: Copy, Space> From<Pos<T, Space>> for (T, T, T) {
    /// Performs the conversion.
    fn from(pos: Pos<T, Space>) -> Self {
        (pos.x, pos.y, pos.z)
    }
}

impl<T: Copy, Space> From<&Pos<T, Space>> for (T, T, T) {
    /// Performs the conversion.
    fn from(pos: &Pos<T, Space>) -> Self {
        (pos.x, pos.y, pos.z)
    }
}

impl<T: Copy, Space> From<Pos<T, Space>> for [T; 3] {
    /// Performs the conversion.
    fn from(pos: Pos<T, Space>) -> Self {
        [pos.x, pos.y, pos.z]
    }
}

impl<T: Copy, Space> From<&Pos<T, Space>> for [T; 3] {
    /// Performs the conversion.
    fn from(pos: &Pos<T, Space>) -> Self {
        [pos.x, pos.y, pos.z]
    }
}

impl<T: Display, Space> Display for Pos<T, Space> {
    /// Formats the value using the given formatter.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

/// Represents the position of a Voxel in the Chunk Grid.
///
/// The `VoxelPos` is a 3D coordinate system where each coordinate is an u8, ranging from 0 to 31.
pub type VoxelPos = Pos<u8, VoxelSpace>;

impl VoxelPos {
    /// The origin of the Voxel Grid.
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        z: 0,
        _space: PhantomData,
    };

    /// The maximum valid value for a coordinate in a `VoxelPos`.
    pub const MAX_VALID: u8 = CHUNK_SIZE_U8 - 1;

    /// Total number of valid voxels (2^5) in a single dimension.
    const TOTAL_VALID: u8 = CHUNK_SIZE_U8;

    /// Creates a new `VoxelPos` with the given coordinates, if they are valid.
    ///
    /// # Panics
    ///
    /// Invalid values will cause a panic. Always verify the component is less than `CHUNK_SIZE_U8`
    #[must_use]
    pub const fn from_raw_checked(x: u8, y: u8, z: u8) -> Self {
        assert!(x < Self::TOTAL_VALID, "invalid x component");
        assert!(y < Self::TOTAL_VALID, "invalid y component");
        assert!(z < Self::TOTAL_VALID, "invalid z component");

        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }

    /// Creates a new `VoxelPos` with the given coordinates, if they are valid, or returns an error
    ///
    /// # Errors
    ///
    /// If the coordinates are too large to be appropriate for the Chunk Grid, this function will return an error.
    pub const fn try_new(x: u8, y: u8, z: u8) -> Result<Self, PosOutOfBounds> {
        if x > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(x))
        } else if y > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(y))
        } else if z > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(z))
        } else {
            Ok(Self {
                x,
                y,
                z,
                _space: PhantomData,
            })
        }
    }

    /// Converts a `WorldPos` to a `VoxelPos` using `rem_euclid`.
    ///
    /// This ensures that the coordinates are within the bounds of an u8.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn from_world(world_pos: &WorldPos) -> Self {
        let (world_x, world_y, world_z) = world_pos.all();

        let x = world_x.rem_euclid(CHUNK_SIZE_I32) as u8;
        let y = world_y.rem_euclid(CHUNK_SIZE_I32) as u8;
        let z = world_z.rem_euclid(CHUNK_SIZE_I32) as u8;

        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }
}

impl From<WorldPos> for VoxelPos {
    /// Creates a new `VoxelPos` from a `WorldPos`.
    fn from(world_pos: WorldPos) -> Self {
        Self::from_world(&world_pos)
    }
}

impl TryFrom<(u8, u8, u8)> for VoxelPos {
    type Error = PosOutOfBounds;

    /// Attempts to perform the conversion.
    ///
    /// # Errors
    ///
    /// If the coordinates are too large to be appropriate for the Chunk Grid, this function will return an error.
    fn try_from((x, y, z): (u8, u8, u8)) -> Result<Self, Self::Error> {
        if x > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(x))
        } else if y > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(y))
        } else if z > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(z))
        } else {
            Ok(Self {
                x,
                y,
                z,
                _space: PhantomData,
            })
        }
    }
}

impl TryFrom<[u8; 3]> for VoxelPos {
    type Error = PosOutOfBounds;

    /// Attempts to perform the conversion.
    ///
    /// # Errors
    ///
    /// If the coordinates are too large to be appropriate for the Chunk Grid, this function will return an error.
    fn try_from([x, y, z]: [u8; 3]) -> Result<Self, Self::Error> {
        if x > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(x))
        } else if y > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(y))
        } else if z > Self::MAX_VALID {
            Err(PosOutOfBounds::VoxelPos(z))
        } else {
            Ok(Self {
                x,
                y,
                z,
                _space: PhantomData,
            })
        }
    }
}

impl Add for VoxelPos {
    type Output = Self;

    /// Performs the operation.
    ///
    /// # Panics
    ///
    /// If the resulting value would overflow/underflow, the operation will panic.
    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_add(rhs.x);
        let y = self.y.checked_add(rhs.y);
        let z = self.z.checked_add(rhs.z);

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Self {
                x,
                y,
                z,
                _space: PhantomData,
            },
            _ => panic!("invalid ChunkPos ({:?}, {:?}, {:?})", x, y, z),
        }
    }
}

impl Sub for VoxelPos {
    type Output = Self;

    /// Performs the operation.
    ///
    /// # Panics
    ///
    /// If the resulting value would overflow/underflow, the operation will panic.
    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_sub(rhs.x);
        let y = self.y.checked_sub(rhs.y);
        let z = self.z.checked_sub(rhs.z);

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Self {
                x,
                y,
                z,
                _space: PhantomData,
            },
            _ => panic!("invalid VoxelPos ({:?}, {:?}, {:?})", x, y, z),
        }
    }
}

/// Represents the position of a Chunk in the World Grid.
///
/// The `ChunkPos` is a 3D coordinate system where each coordinate is an i32.
pub type ChunkPos = Pos<i32, ChunkSpace>;

impl ChunkPos {
    /// The origin of the Chunk Grid.
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        z: 0,
        _space: PhantomData,
    };

    /// Maximum coordinate value for a valid `ChunkPos`.
    const MAX_VALID: i32 = WORLD_POS_I32 / CHUNK_SIZE_I32;

    /// Minimum coordinate value for a valid 'ChunkPos'
    const MIN_VALID: i32 = WORLD_NEG_I32 / CHUNK_SIZE_I32;

    /// Creates a new `ChunkPos` with the given coordinates, if they are valid.
    ///
    /// # Panics
    ///
    /// If the values are not valid (not in the range of `MIN_VALID` to `MAX_VALID`), this function will panic.
    #[must_use]
    pub const fn from_raw_checked(x: i32, y: i32, z: i32) -> Self {
        assert!(
            x >= Self::MIN_VALID || x <= Self::MAX_VALID,
            "invalid x component"
        );
        assert!(
            y >= Self::MIN_VALID || y <= Self::MAX_VALID,
            "invalid y component"
        );
        assert!(
            z >= Self::MIN_VALID || z <= Self::MAX_VALID,
            "invalid z component"
        );

        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }

    /// Creates a new `ChunkPos` with the given coordinates, if they are valid, or returns an error
    ///
    /// # Errors
    ///
    /// If the coordinates are too large or too little to be appropriate for the World Grid, this function will return an error.
    pub const fn try_new(x: i32, y: i32, z: i32) -> Result<Self, PosOutOfBounds> {
        if x < Self::MIN_VALID || x > Self::MAX_VALID {
            Err(PosOutOfBounds::ChunkPos(x))
        } else if y < Self::MIN_VALID || y > Self::MAX_VALID {
            Err(PosOutOfBounds::ChunkPos(y))
        } else if z < Self::MIN_VALID || z > Self::MAX_VALID {
            Err(PosOutOfBounds::ChunkPos(z))
        } else {
            Ok(Self {
                x,
                y,
                z,
                _space: PhantomData,
            })
        }
    }

    /// Converts a `WorldPos` to a `ChunkPos` using `div_euclid`.
    #[must_use]
    pub const fn from_world(world_pos: &WorldPos) -> Self {
        let (world_x, world_y, world_z) = world_pos.all();

        let x = world_x.div_euclid(CHUNK_SIZE_I32);
        let y = world_y.div_euclid(CHUNK_SIZE_I32);
        let z = world_z.div_euclid(CHUNK_SIZE_I32);

        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }
}

impl From<WorldPos> for ChunkPos {
    /// Creates a new `ChunkPos` from a `ChunkPos`.
    fn from(world_pos: WorldPos) -> Self {
        Self::from_world(&world_pos)
    }
}

impl TryFrom<(i32, i32, i32)> for ChunkPos {
    type Error = PosOutOfBounds;

    /// Attempts to perform the conversion.
    ///
    /// # Errors
    ///
    /// If the coordinates are too large or too little to be appropriate for the World Grid, this function will return an error.
    fn try_from((x, y, z): (i32, i32, i32)) -> Result<Self, Self::Error> {
        if !(Self::MIN_VALID..=Self::MAX_VALID).contains(&x) {
            Err(PosOutOfBounds::ChunkPos(x))
        } else if !(Self::MIN_VALID..=Self::MAX_VALID).contains(&y) {
            Err(PosOutOfBounds::ChunkPos(y))
        } else if !(Self::MIN_VALID..=Self::MAX_VALID).contains(&z) {
            Err(PosOutOfBounds::ChunkPos(z))
        } else {
            Ok(Self {
                x,
                y,
                z,
                _space: PhantomData,
            })
        }
    }
}

impl TryFrom<[i32; 3]> for ChunkPos {
    type Error = PosOutOfBounds;

    /// Attempts to perform the conversion.
    ///
    /// # Errors
    ///
    /// If the coordinates are too large or too little to be appropriate for the World Grid, this function will return an error.
    fn try_from([x, y, z]: [i32; 3]) -> Result<Self, Self::Error> {
        if !(Self::MIN_VALID..=Self::MAX_VALID).contains(&x) {
            Err(PosOutOfBounds::ChunkPos(x))
        } else if !(Self::MIN_VALID..=Self::MAX_VALID).contains(&y) {
            Err(PosOutOfBounds::ChunkPos(y))
        } else if !(Self::MIN_VALID..=Self::MAX_VALID).contains(&z) {
            Err(PosOutOfBounds::ChunkPos(z))
        } else {
            Ok(Self {
                x,
                y,
                z,
                _space: PhantomData,
            })
        }
    }
}

impl Add for ChunkPos {
    type Output = Self;

    /// Performs the operation.
    ///
    /// # Panics
    ///
    /// If the resulting value would overflow/underflow, the operation will panic.
    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_add(rhs.x);
        let y = self.y.checked_add(rhs.y);
        let z = self.z.checked_add(rhs.z);

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Self {
                x,
                y,
                z,
                _space: PhantomData,
            },
            _ => panic!("invalid ChunkPos ({:?}, {:?}, {:?})", x, y, z),
        }
    }
}

impl Sub for ChunkPos {
    type Output = Self;

    /// Performs the operation.
    ///
    /// # Panics
    ///
    /// If the resulting value would overflow/underflow, the operation will panic.
    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_sub(rhs.x);
        let y = self.y.checked_sub(rhs.y);
        let z = self.z.checked_sub(rhs.z);

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Self {
                x,
                y,
                z,
                _space: PhantomData,
            },
            _ => panic!("invalid ChunkPos ({:?}, {:?}, {:?})", x, y, z),
        }
    }
}

/// Represents the position of a Voxel in the Global Grid.
///
/// The `WorldPos` is a 3D coordinate system where each coordinate is an i32.
pub type WorldPos = Pos<i32, WorldSpace>;

impl WorldPos {
    /// The origin of the Voxel Grid.
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        z: 0,
        _space: PhantomData,
    };

    /// Maximum coordinate value for a valid `WorldPos`.
    const _MAX_VALID: i32 = WORLD_POS_I32;

    /// Minimum coordinate value for a valid 'WorldPos'
    const _MIN_VALID: i32 = WORLD_NEG_I32;

    /// Creates a new `WorldPos` with the given coordinates.
    ///
    /// This is a convenience function, it behaves exactly like `new_unchecked`.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }

    /// Composes a `WorldPos` from a `ChunkPos` and a `VoxelPos`.
    #[must_use]
    pub const fn from_chunk_voxel(chunk_pos: &ChunkPos, voxel_pos: &VoxelPos) -> Self {
        let (voxel_x, voxel_y, voxel_z) = voxel_pos.all();
        let (chunk_x, chunk_y, chunk_z) = chunk_pos.all();

        let x = (voxel_x as i32) + chunk_x * CHUNK_SIZE_I32;
        let y = (voxel_y as i32) + chunk_y * CHUNK_SIZE_I32;
        let z = (voxel_z as i32) + chunk_z * CHUNK_SIZE_I32;

        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }
}

impl From<(ChunkPos, VoxelPos)> for WorldPos {
    /// Creates a new `WorldPos` from a tuple with `ChunkPos` and `VoxelPos`.
    fn from(tuple: (ChunkPos, VoxelPos)) -> Self {
        Self::from_chunk_voxel(&tuple.0, &tuple.1)
    }
}

impl From<(i32, i32, i32)> for WorldPos {
    /// Performs the conversion.
    fn from((x, y, z): (i32, i32, i32)) -> Self {
        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }
}

impl From<[i32; 3]> for WorldPos {
    /// Performs the conversion.
    fn from([x, y, z]: [i32; 3]) -> Self {
        Self {
            x,
            y,
            z,
            _space: PhantomData,
        }
    }
}

impl Add for WorldPos {
    type Output = Self;

    /// Performs the operation.
    ///
    /// # Panics
    ///
    /// If the resulting value would overflow/underflow, the operation will panic.
    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_add(rhs.x);
        let y = self.y.checked_add(rhs.y);
        let z = self.z.checked_add(rhs.z);

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Self {
                x,
                y,
                z,
                _space: PhantomData,
            },
            _ => panic!("invalid WorldPosition"),
        }
    }
}

impl Sub for WorldPos {
    type Output = Self;

    /// Performs the operation.
    ///
    /// # Panics
    ///
    /// If the resulting value would overflow/underflow, the operation will panic.
    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_sub(rhs.x);
        let y = self.y.checked_sub(rhs.y);
        let z = self.z.checked_sub(rhs.z);

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Self {
                x,
                y,
                z,
                _space: PhantomData,
            },
            _ => panic!("invalid WorldPosition"),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
/// Errors that can occur when working with positions. Mainly out of bounds.
pub enum PosOutOfBounds {
    #[error("VoxelPos out of bounds: {}", .0)]
    VoxelPos(u8),
    #[error("ChunkPos out of bounds: {}", .0)]
    ChunkPos(i32),
}

#[cfg(test)]
mod tests {
    use crate::{
        constants::CHUNK_SIZE_U8,
        pos::{
            ChunkPos, VoxelPos, WorldPos,
            position::{Pos, WorldSpace},
        },
    };

    #[test]
    fn from_raw_preserves_coordinates() {
        let pos = Pos::<i32, WorldSpace>::from_raw(1, -2, 3);

        assert_eq!(pos.all(), (1, -2, 3));
        assert_eq!(pos.x(), 1);
        assert_eq!(pos.y(), -2);
        assert_eq!(pos.z(), 3);
    }

    #[test]
    fn tuple_conversion_preserves_coordinates() {
        let pos = Pos::<i32, WorldSpace>::from((1, 2, 3));

        assert_eq!(pos.all(), (1, 2, 3));
    }

    #[test]
    fn array_conversion_preserves_coordinates() {
        let pos = Pos::<i32, WorldSpace>::from([1, 2, 3]);

        assert_eq!(pos.all(), (1, 2, 3));
    }

    #[test]
    fn converts_to_tuple() {
        let pos = Pos::<i32, WorldSpace>::from_raw(1, 2, 3);

        let tuple: (i32, i32, i32) = pos.into();

        assert_eq!(tuple, (1, 2, 3));
    }

    #[test]
    fn converts_to_array() {
        let pos = Pos::<i32, WorldSpace>::from_raw(1, 2, 3);

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

        assert_eq!(pos.all(), (1, 2, 3));
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

        assert_eq!(pos.all(), (200, 201, 202));
    }

    #[test]
    fn from_raw_checked_accepts_valid_coordinates() {
        let pos = VoxelPos::from_raw_checked(31, 10, 0);

        assert_eq!(pos.all(), (31, 10, 0));
    }

    #[test]
    #[should_panic]
    fn from_raw_checked_panics_on_invalid_x() {
        let _ = VoxelPos::from_raw_checked(32, 0, 0);
    }

    #[test]
    #[should_panic]
    fn from_raw_checked_panics_on_invalid_y() {
        let _ = VoxelPos::from_raw_checked(0, 32, 0);
    }

    #[test]
    #[should_panic]
    fn from_raw_checked_panics_on_invalid_z() {
        let _ = VoxelPos::from_raw_checked(0, 0, 32);
    }

    #[test]
    fn world_origin_decomposes_correctly() {
        let world = WorldPos::from_raw(0, 0, 0);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw_checked(0, 0, 0));
    }

    #[test]
    fn last_voxel_of_first_chunk_decomposes_correctly() {
        let world = WorldPos::from_raw(31, 31, 31);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(0, 0, 0));
        assert_eq!(
            VoxelPos::from(world),
            VoxelPos::from_raw_checked(31, 31, 31)
        );
    }

    #[test]
    fn first_voxel_of_next_chunk_decomposes_correctly() {
        let world = WorldPos::from_raw(32, 32, 32);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(1, 1, 1));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw_checked(0, 0, 0));
    }

    #[test]
    fn negative_one_decomposes_to_previous_chunk_last_voxel() {
        let world = WorldPos::from_raw(-1, -1, -1);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-1, -1, -1));
        assert_eq!(
            VoxelPos::from(world),
            VoxelPos::from_raw_checked(31, 31, 31)
        );
    }

    #[test]
    fn negative_chunk_boundary_decomposes_correctly() {
        let world = WorldPos::from_raw(-32, -32, -32);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-1, -1, -1));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw_checked(0, 0, 0));
    }

    #[test]
    fn negative_coordinate_just_beyond_chunk_boundary_decomposes_correctly() {
        let world = WorldPos::from_raw(-33, -33, -33);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-2, -2, -2));
        assert_eq!(
            VoxelPos::from(world),
            VoxelPos::from_raw_checked(31, 31, 31)
        );
    }

    #[test]
    fn mixed_sign_coordinates_decompose_independently() {
        let world = WorldPos::from_raw(-1, 0, 32);

        assert_eq!(ChunkPos::from(world), ChunkPos::from_raw(-1, 0, 1));
        assert_eq!(VoxelPos::from(world), VoxelPos::from_raw_checked(31, 0, 0));
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

                    assert!(voxel.x() < CHUNK_SIZE_U8);
                    assert!(voxel.y() < CHUNK_SIZE_U8);
                    assert!(voxel.z() < CHUNK_SIZE_U8);
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
