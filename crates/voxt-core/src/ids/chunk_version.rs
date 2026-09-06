/// Versioning for chunks. Used in resolving conflicts and ensuring consistency.
///
/// When a Chunk receives an update, the `ChunkVersion` number increments by 1.
///
/// A `ChunkReference` may be given version 3 upon being put on queue for processing (meshing, for example).
/// Upon completion of the mesh job, the World will check if the Chunk still has the same `ChunkVersion` number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ChunkVersion(u64);

impl ChunkVersion {
    /// Creates a new `ChunkVersion` with the given version number.
    #[must_use]
    pub const fn new(version: u64) -> Self {
        Self(version)
    }

    /// Returns the `ChunkVersion` number as an u64.
    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Returns the next `ChunkVersion` number.
    #[must_use]
    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    /// Increments the `ChunkVersion` number by 1.
    pub const fn increment(&mut self) {
        self.0 += 1;
    }
}
