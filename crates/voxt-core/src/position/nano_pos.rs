use crate::TotalPos;

/// Represents the position of any entity/object (e.g. a Player, a Particle) in the Voxel Grid.
///
/// The `NanoPos` is a 3D coordinate system where each coordinate is an u8.
/// It is used to represent the position of entities with high granularity.
/// The `NanoPos` value is interpreted as increasing in steps of 1/255th of a Voxel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NanoPos {
    x: u8,
    y: u8,
    z: u8,
}

impl NanoPos {
    /// The maximum value of the `NanoPos` coordinates as f32 is 255.0.
    const MAX: f32 = 255.0;

    /// The step size of the `NanoPos` coordinates as f32 is 1.0 / 255.0.
    const STEP: f32 = 1.0 / 255.0;

    /// Creates a new `NanoPos` with the given coordinates.
    #[must_use]
    pub const fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }

    /// Returns the coordinates of the `NanoPos` as a tuple of u8's.
    #[must_use]
    pub const fn coord(&self) -> (u8, u8, u8) {
        (self.x, self.y, self.z)
    }

    /// Returns the coordinates of the `NanoPos` as a tuple of f32's, ranging from 0.0 to 1.0.
    #[must_use]
    pub const fn coord_as_f32(&self) -> (f32, f32, f32) {
        (
            Self::STEP * (self.x as f32),
            Self::STEP * (self.y as f32),
            Self::STEP * (self.z as f32),
        )
    }

    /// Returns the x-coordinate of the `NanoPos`.
    #[must_use]
    pub const fn x(&self) -> u8 {
        self.x
    }

    /// Returns the x-coordinate of the `NanoPos` as a f32, ranging from 0.0 to 1.0.
    #[must_use]
    pub const fn x_as_f32(&self) -> f32 {
        Self::STEP * (self.x as f32)
    }

    /// Returns the y-coordinate of the `NanoPos`.
    #[must_use]
    pub const fn y(&self) -> u8 {
        self.y
    }

    /// Returns the y-coordinate of the `NanoPos` as a f32, ranging from 0.0 to 1.0.
    #[must_use]
    pub const fn y_as_f32(&self) -> f32 {
        Self::STEP * (self.y as f32)
    }

    /// Returns the z-coordinate of the `NanoPos`.
    #[must_use]
    pub const fn z(&self) -> u8 {
        self.z
    }

    /// Returns the z-coordinate of the `NanoPos` as a f32, ranging from 0.0 to 1.0.
    #[must_use]
    pub const fn z_as_f32(&self) -> f32 {
        Self::STEP * (self.z as f32)
    }

    /// Creates a new `NanoPos` from a `TotalPos`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn from_total(total_pos: &TotalPos) -> Self {
        let (total_x, total_y, total_z) = total_pos.coord();

        Self {
            x: (Self::MAX * total_x.fract()) as u8,
            y: (Self::MAX * total_y.fract()) as u8,
            z: (Self::MAX * total_z.fract()) as u8,
        }
    }

    /// Creates a new `NanoPos` from a tuple of three f32's.
    /// Values will be clamped to 0.0 -> 1.0.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn from_f32_tuple(tuple: (f32, f32, f32)) -> Self {
        Self {
            x: (Self::MAX * tuple.0.clamp(0.0, 1.0)) as u8,
            y: (Self::MAX * tuple.1.clamp(0.0, 1.0)) as u8,
            z: (Self::MAX * tuple.2.clamp(0.0, 1.0)) as u8,
        }
    }

    /// Creates a new `NanoPos` from an array of three f32's.
    /// Values will be clamped to 0.0 -> 1.0.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn from_f32_array(array: [f32; 3]) -> Self {
        Self {
            x: (Self::MAX * array[0].clamp(0.0, 1.0)) as u8,
            y: (Self::MAX * array[1].clamp(0.0, 1.0)) as u8,
            z: (Self::MAX * array[2].clamp(0.0, 1.0)) as u8,
        }
    }

    /// Creates a new `NanoPos` from a tuple of three u8's.
    #[must_use]
    pub const fn from_u8_tuple(tuple: (u8, u8, u8)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }

    /// Creates a new `NanoPos` from an array of three u8's.
    #[must_use]
    pub const fn from_u8_array(array: [u8; 3]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }
}

impl From<(f32, f32, f32)> for NanoPos {
    /// Creates a new `NanoPos` from a tuple of three f32's.
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self::from_f32_tuple(tuple)
    }
}

impl From<[f32; 3]> for NanoPos {
    /// Creates a new `NanoPos` from an array of three f32's.
    fn from(array: [f32; 3]) -> Self {
        Self::from_f32_array(array)
    }
}

impl From<(u8, u8, u8)> for NanoPos {
    /// Creates a new `NanoPos` from a tuple of three u8's.
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self::from_u8_tuple(tuple)
    }
}

impl From<[u8; 3]> for NanoPos {
    /// Creates a new `NanoPos` from an array of three u8's.
    fn from(array: [u8; 3]) -> Self {
        Self::from_u8_array(array)
    }
}

impl From<&TotalPos> for NanoPos {
    /// Creates a new `NanoPos` from a `TotalPos`.
    fn from(total_pos: &TotalPos) -> Self {
        Self::from_total(total_pos)
    }
}
