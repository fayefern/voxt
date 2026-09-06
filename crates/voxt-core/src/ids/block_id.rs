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
}
