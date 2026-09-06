use crate::{MacroPos, MesoPos, MicroPos, NanoPos};

/// Represents the position of a Voxel in the Global Grid, including the `NanoPos`.
///
/// The `TotalPos` is a 3D coordinate system where each coordinate is an f32.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TotalPos {
    x: f32,
    y: f32,
    z: f32,
}

impl TotalPos {
    /// Creates a new `TotalPos` with the given coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns the coordinates of the `TotalPos` as a tuple of f32's.
    #[must_use]
    pub const fn coord(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    /// Returns the x-coordinate of the `TotalPos`.
    #[must_use]
    pub const fn x(&self) -> f32 {
        self.x
    }

    /// Returns the y-coordinate of the `TotalPos`.
    #[must_use]
    pub const fn y(&self) -> f32 {
        self.y
    }

    /// Returns the z-coordinate of the `TotalPos`.
    #[must_use]
    pub const fn z(&self) -> f32 {
        self.z
    }

    /// Creates a new `TotalPos` from a `MacroPos` and `NanoPos`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub const fn from_macro_nano(macro_pos: &MacroPos, nano_pos: &NanoPos) -> Self {
        let (macro_x, macro_y, macro_z) = macro_pos.coord();
        let (nano_x, nano_y, nano_z) = nano_pos.coord_as_f32();

        Self {
            x: (macro_x as f32) + nano_x,
            y: (macro_y as f32) + nano_y,
            z: (macro_z as f32) + nano_z,
        }
    }

    /// Creates a new `TotalPos` from a `MesoPos`, `MicroPos`, and `NanoPos`.
    #[must_use]
    pub const fn from_meso_micro_nano(
        meso_pos: &MesoPos,
        micro_pos: &MicroPos,
        nano_pos: &NanoPos,
    ) -> Self {
        Self::from_macro_nano(&MacroPos::from_meso_micro(meso_pos, micro_pos), nano_pos)
    }

    /// Creates a new `TotalPos` from a tuple of three f32's.
    #[must_use]
    pub const fn from_tuple(tuple: (f32, f32, f32)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
            z: tuple.2,
        }
    }

    /// Creates a new `TotalPos` from an array of three f32's.
    #[must_use]
    pub const fn from_array(array: [f32; 3]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }
}

impl From<(f32, f32, f32)> for TotalPos {
    /// Creates a new `TotalPos` from a tuple of three f32's.
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self::from_tuple(tuple)
    }
}

impl From<[f32; 3]> for TotalPos {
    /// Creates a new `TotalPos` from an array of three f32's.
    fn from(array: [f32; 3]) -> Self {
        Self::from_array(array)
    }
}

impl From<(&MacroPos, &NanoPos)> for TotalPos {
    /// Creates a new `TotalPos` from a tuple with `MacroPos` and `NanoPos`.
    fn from(tuple: (&MacroPos, &NanoPos)) -> Self {
        Self::from_macro_nano(tuple.0, tuple.1)
    }
}

impl From<(&MesoPos, &MicroPos, &NanoPos)> for TotalPos {
    /// Creates a new `TotalPos` from a tuple with `MesoPos`, `MicroPos` and `NanoPos`.
    fn from(tuple: (&MesoPos, &MicroPos, &NanoPos)) -> Self {
        Self::from_meso_micro_nano(tuple.0, tuple.1, tuple.2)
    }
}
