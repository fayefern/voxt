use crate::{CHUNK_SIZE, WorldPos};

/// Represents the position of a Voxel in the Chunk Grid.
///
/// The `VoxelPos` is a 3D coordinate system where each coordinate is an u8, ranging from 0 to 31.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct VoxelPos {
    x: u8,
    y: u8,
    z: u8,
}

impl VoxelPos {
    /// Creates a new `VoxelPos` with the given coordinates.
    #[must_use]
    pub const fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }

    /// Returns the coordinates of the `VoxelPos` as a tuple of u8's.
    #[must_use]
    pub const fn coord(&self) -> (u8, u8, u8) {
        (self.x, self.y, self.z)
    }

    /// Returns the x-coordinate of the `VoxelPos`.
    #[must_use]
    pub const fn x(&self) -> u8 {
        self.x
    }

    /// Returns the y-coordinate of the `VoxelPos`.
    #[must_use]
    pub const fn y(&self) -> u8 {
        self.y
    }

    /// Returns the z-coordinate of the `VoxelPos`.
    #[must_use]
    pub const fn z(&self) -> u8 {
        self.z
    }

    /// Converts a `WorldPos` to a `VoxelPos` using `rem_euclid`.
    ///
    /// This ensures that the coordinates are within the bounds of an u8.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn from_macro(macro_pos: &WorldPos) -> Self {
        let (macro_x, macro_y, macro_z) = macro_pos.coord();

        Self {
            x: (macro_x.rem_euclid(CHUNK_SIZE) as u8),
            y: (macro_y.rem_euclid(CHUNK_SIZE) as u8),
            z: (macro_z.rem_euclid(CHUNK_SIZE) as u8),
        }
    }

    /// Creates a new `VoxelPos` from a tuple of three u8's.
    #[must_use]
    pub const fn from_tuple(tuple: (u8, u8, u8)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }

    /// Creates a new `VoxelPos` from an array of three u8's.
    #[must_use]
    pub const fn from_array(array: [u8; 3]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }
}

impl From<(u8, u8, u8)> for VoxelPos {
    /// Creates a new `VoxelPos` from a tuple of three u8's.
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self::from_tuple(tuple)
    }
}

impl From<[u8; 3]> for VoxelPos {
    /// Creates a new `VoxelPos` from an array of three u8's.
    fn from(array: [u8; 3]) -> Self {
        Self::from_array(array)
    }
}

impl From<&WorldPos> for VoxelPos {
    /// Creates a new `VoxelPos` from a `VoxelPos`.
    fn from(macro_pos: &WorldPos) -> Self {
        Self::from_macro(macro_pos)
    }
}
