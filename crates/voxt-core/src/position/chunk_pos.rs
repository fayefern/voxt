use crate::{CHUNK_SIZE, WorldPos};

/// Represents the position of a Chunk in the World Grid.
///
/// The `ChunkPos` is a 3D coordinate system where each coordinate is an i32.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    x: i32,
    y: i32,
    z: i32,
}

impl ChunkPos {
    /// Creates a new `ChunkPos` with the given coordinates.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns the coordinates of the `ChunkPos` as a tuple of i32's.
    #[must_use]
    pub const fn coord(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    /// Returns the x-coordinate of the `ChunkPos`.
    #[must_use]
    pub const fn x(&self) -> i32 {
        self.x
    }

    /// Returns the y-coordinate of the `ChunkPos`.
    #[must_use]
    pub const fn y(&self) -> i32 {
        self.y
    }

    /// Returns the z-coordinate of the `ChunkPos`.
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }

    /// Converts a `WorldPos` to a `ChunkPos` using `div_euclid`.
    #[must_use]
    pub const fn from_macro(macro_pos: &WorldPos) -> Self {
        let (macro_x, macro_y, macro_z) = macro_pos.coord();

        Self {
            x: macro_x.div_euclid(CHUNK_SIZE),
            y: macro_y.div_euclid(CHUNK_SIZE),
            z: macro_z.div_euclid(CHUNK_SIZE),
        }
    }

    /// Creates a new `ChunkPos` from a tuple of three i32's.
    #[must_use]
    pub const fn from_tuple(tuple: (i32, i32, i32)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }

    /// Creates a new `ChunkPos` from an array of three i32's.
    #[must_use]
    pub const fn from_array(array: [i32; 3]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }
}

impl From<(i32, i32, i32)> for ChunkPos {
    /// Creates a new `ChunkPos` from a tuple of three i32's.
    fn from(tuple: (i32, i32, i32)) -> Self {
        Self::from_tuple(tuple)
    }
}

impl From<[i32; 3]> for ChunkPos {
    /// Creates a new `ChunkPos` from an array of three i32's.
    fn from(array: [i32; 3]) -> Self {
        Self::from_array(array)
    }
}

impl From<&WorldPos> for ChunkPos {
    /// Creates a new `ChunkPos` from a `ChunkPos`.
    fn from(macro_pos: &WorldPos) -> Self {
        Self::from_macro(macro_pos)
    }
}
