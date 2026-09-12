use std::fmt::{Debug, Display};

/// Direction in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    /// Positive Y-axis direction.
    Up,
    /// Negative Y-axis direction.
    Down,
    /// Negative X-axis direction.
    Left,
    /// Positive X-axis direction.
    Right,
    /// Positive Z-axis direction. +1.0 FAR.
    Front,
    /// Negative Z-axis direction. -1.0 NEAR.
    Back,
}

impl Dir {
    pub const LIST: [Self; 6] = [
        Dir::Up,
        Dir::Down,
        Dir::Left,
        Dir::Right,
        Dir::Front,
        Dir::Back,
    ];

    /// Returns the inverse direction.
    #[must_use]
    pub const fn inv(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Front => Self::Back,
            Self::Back => Self::Front,
        }
    }

    /// Returns true if the direction is along the X-axis.
    #[must_use]
    pub const fn is_x(&self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Returns true if the direction is along the Y-axis.
    #[must_use]
    pub const fn is_y(&self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    /// Returns true if the direction is along the Z-axis.
    #[must_use]
    pub const fn is_z(&self) -> bool {
        matches!(self, Self::Front | Self::Back)
    }

    /// Returns true if the direction is positive.
    #[must_use]
    pub const fn is_plus(&self) -> bool {
        matches!(self, Self::Up | Self::Right | Self::Front)
    }

    /// Returns true if the direction is negative.
    #[must_use]
    pub const fn is_minus(&self) -> bool {
        matches!(self, Self::Down | Self::Left | Self::Back)
    }

    /// Returns an iterator over all directions in the order that follows:
    ///
    /// Up, Down, Left, Right, Front, Back.
    pub fn iter() -> impl Iterator<Item = Self> {
        Self::LIST.into_iter()
    }

    /// Receives a tuple of the direction's x, y, and z unitary components.
    ///
    /// Returns the corresponding direction if the tuple is a valid unit.
    pub const fn from_off_tuple((x, y, z): (i8, i8, i8)) -> Option<Self> {
        match (x, y, z) {
            (0, 1, 0) => Some(Self::Up),
            (0, -1, 0) => Some(Self::Down),
            (-1, 0, 0) => Some(Self::Left),
            (1, 0, 0) => Some(Self::Right),
            (0, 0, 1) => Some(Self::Front),
            (0, 0, -1) => Some(Self::Back),
            _ => None,
        }
    }

    /// Returns the direction's x, y, and z unitary components as a tuple of i8
    #[must_use]
    pub const fn as_off_tuple(&self) -> (i8, i8, i8) {
        match self {
            Self::Up => (0, 1, 0),
            Self::Down => (0, -1, 0),
            Self::Left => (-1, 0, 0),
            Self::Right => (1, 0, 0),
            Self::Front => (0, 0, 1),
            Self::Back => (0, 0, -1),
        }
    }

    /// Receives an array of the direction's x, y, and z unitary components.
    ///
    /// Returns the corresponding direction if the tuple is a valid unit.
    pub const fn from_off_array([x, y, z]: [i8; 3]) -> Option<Self> {
        match (x, y, z) {
            (0, 1, 0) => Some(Self::Up),
            (0, -1, 0) => Some(Self::Down),
            (-1, 0, 0) => Some(Self::Left),
            (1, 0, 0) => Some(Self::Right),
            (0, 0, 1) => Some(Self::Front),
            (0, 0, -1) => Some(Self::Back),
            _ => None,
        }
    }

    /// Returns the direction's x, y, and z unitary components as an array of i8
    #[must_use]
    pub const fn as_off_array(&self) -> [i8; 3] {
        match self {
            Self::Up => [0, 1, 0],
            Self::Down => [0, -1, 0],
            Self::Left => [-1, 0, 0],
            Self::Right => [1, 0, 0],
            Self::Front => [0, 0, 1],
            Self::Back => [0, 0, -1],
        }
    }
}

impl From<Dir> for (i8, i8, i8) {
    fn from(d: Dir) -> Self {
        d.as_off_tuple()
    }
}

impl From<Dir> for [i8; 3] {
    fn from(d: Dir) -> Self {
        d.as_off_array()
    }
}

impl Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dir::Up => write!(f, "Up (+Y)"),
            Dir::Down => write!(f, "Down (-Y)"),
            Dir::Left => write!(f, "Left (-X)"),
            Dir::Right => write!(f, "Right (+X)"),
            Dir::Front => write!(f, "Front (+Z)"),
            Dir::Back => write!(f, "Back (-Z)"),
        }
    }
}
