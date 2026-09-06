use crate::CHUNK_SIZE;
use crate::ChunkPos;
use crate::VoxelPos;

/// Represents the position of a Voxel in the Global Grid.
/// Not to be confused with the `TotalPos`.
///
/// The `WorldPos` is a 3D coordinate system where each coordinate is an i32.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WorldPos {
    x: i32,
    y: i32,
    z: i32,
}

impl WorldPos {
    /// Creates a new `WorldPos` with the given coordinates.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns the coordinates of the `WorldPos` as a tuple of i32's.
    #[must_use]
    pub const fn coord(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    /// Returns the x-coordinate of the `WorldPos`.
    #[must_use]
    pub const fn x(&self) -> i32 {
        self.x
    }

    /// Returns the y-coordinate of the `WorldPos`.
    #[must_use]
    pub const fn y(&self) -> i32 {
        self.y
    }

    /// Returns the z-coordinate of the `WorldPos`.
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }

    /// Composes a `WorldPos` from a `ChunkPos` and a `VoxelPos`.
    #[must_use]
    pub const fn from_meso_micro(meso_pos: &ChunkPos, micro_pos: &VoxelPos) -> Self {
        let (micro_x, micro_y, micro_z) = micro_pos.coord();
        let (meso_x, meso_y, meso_z) = meso_pos.coord();

        Self {
            x: (micro_x as i32) + meso_x * CHUNK_SIZE,
            y: (micro_y as i32) + meso_y * CHUNK_SIZE,
            z: (micro_z as i32) + meso_z * CHUNK_SIZE,
        }
    }

    /// Creates a new `WorldPos` from a tuple of three i32's.
    #[must_use]
    pub const fn from_tuple(tuple: (i32, i32, i32)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }

    /// Creates a new `WorldPos` from an array of three i32's.
    #[must_use]
    pub const fn from_array(array: [i32; 3]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }
}

impl From<(i32, i32, i32)> for WorldPos {
    /// Creates a new `WorldPos` from a tuple of three i32's.
    fn from(tuple: (i32, i32, i32)) -> Self {
        Self::from_tuple(tuple)
    }
}

impl From<[i32; 3]> for WorldPos {
    /// Creates a new `WorldPos` from an array of three i32's.
    fn from(array: [i32; 3]) -> Self {
        Self::from_array(array)
    }
}

impl From<(&ChunkPos, &VoxelPos)> for WorldPos {
    /// Creates a new `WorldPos` from a tuple with `ChunkPos` and `VoxelPos`.
    fn from(tuple: (&ChunkPos, &VoxelPos)) -> Self {
        Self::from_meso_micro(tuple.0, tuple.1)
    }
}
