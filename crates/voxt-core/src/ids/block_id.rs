use std::fmt::Display;

/// Represents the unique identifier of a Block. This is used to identify a Block's type in the `BlockRegistry`.
///
/// Internally stores an u16. This makes the maximum number of Block types 65,536. `BlockId` 0 is reserved for "Air".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BlockId(u16);

impl BlockId {
    /// Creates a new `BlockId` with the given id.
    #[must_use]
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    /// Returns the id of the `BlockId`.
    #[must_use]
    pub const fn id(&self) -> u16 {
        self.0
    }

    /// Returns true if the `BlockId` is "Air"
    #[must_use]
    pub const fn is_air(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_equal(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::ids::BlockId;

    #[test]
    fn new_preserves_id() {
        let id = BlockId::new(42);
        assert_eq!(id.id(), 42);
    }

    #[test]
    fn zero_is_air() {
        assert!(BlockId::new(0).is_air());
        assert!(!BlockId::new(1).is_air())
    }

    #[test]
    fn maximum_id_is_valid() {
        let id = BlockId::new(u16::MAX);
        assert_eq!(id.id(), u16::MAX);
        assert!(!id.is_air());
    }
}
