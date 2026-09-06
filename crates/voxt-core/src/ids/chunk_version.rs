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

/// UNIT TESTS

#[test]
fn default_version_is_zero() {
    assert_eq!(ChunkVersion::default().as_u64(), 0);
}

#[test]
fn new_preserves_version() {
    assert_eq!(ChunkVersion::new(37).as_u64(), 37);
}

#[test]
fn next_does_not_mutate() {
    let version = ChunkVersion::new(10);

    assert_eq!(version.next().as_u64(), 11);
    assert_eq!(version.as_u64(), 10);
}

#[test]
fn increment_mutates() {
    let mut version = ChunkVersion::new(10);

    version.increment();

    assert_eq!(version.as_u64(), 11);
}

#[test]
fn versions_are_ordered() {
    assert!(ChunkVersion::new(10) < ChunkVersion::new(11));
}

#[test]
fn repeated_increment_advances_one_at_a_time() {
    let mut version = ChunkVersion::default();

    version.increment();
    assert_eq!(version.as_u64(), 1);

    version.increment();
    assert_eq!(version.as_u64(), 2);

    version.increment();
    assert_eq!(version.as_u64(), 3);
}
