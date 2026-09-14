/// Versioning for Chunks. Encompasses changes to its state of residency, as well as mutations
/// to its data and metadata during its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Version {
    /// increased when the object comes into context (e.g. upon chunk loading).
    /// Implies a full sweep has been done (e.g. entire rebuild).
    major: u64,
    /// Increased when the object suffers data mutation (e.g. upon a block breaking).
    /// Implies a big sweep has been done (e.g. physics/collision recalculations).
    minor: u64,
    /// Increased when the object suffers metadata mutation (e.g. upon an animated block being interacted with, lighting update).
    /// Implies a small sweep has been done (e.g. lightweight visual pass, texture update).
    patch: u64,
}

impl Version {
    /// Creates a new version of the form: (major.minor.patch), to be used with the ChunkScheduler and other systems.
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Bumps the lifecycle version. Resets data mutation and metadata mutation versions.
    pub const fn inc_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    /// Checks if a version belongs to the same lifecycle version as another.
    ///
    /// May be used to reject Save or Unload tasks originating from a previous Load of a chunk, for example.
    pub const fn eq_major(&self, other: &Self) -> bool {
        self.major == other.major
    }

    /// Bumps the data mutation version. Resets the metadata mutation version.
    pub const fn inc_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    /// Checks if a version belongs to the same data mutation version as another.
    ///
    /// May be used to reject Mesh tasks originating from a previous state of a chunk, for example.
    pub const fn eq_minor(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }

    /// Bumps the metadata version.
    pub const fn inc_patch(&mut self) {
        self.patch += 1;
    }

    /// Checks if a version belongs to the same metadata mutation version as another.
    pub const fn eq_patch(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

#[cfg(test)]
mod tests {
    use crate::ids::Version;

    #[test]
    fn increment_mutates() {
        let mut version = Version::new(10, 10, 9);

        version.inc_patch();

        assert_eq!(version, Version::new(10, 10, 10));
    }

    #[test]
    fn versions_are_ordered() {
        assert!(Version::new(2, 1, 1) == Version::new(2, 1, 1));
        assert!(Version::new(3, 9, 9) != Version::new(2, 10, 10));
        assert!(Version::new(10, 1, 0) > Version::new(8, 11, 11));
        assert!(Version::new(10, 20, 10) > Version::new(8, 40, 50));
    }

    #[test]
    fn repeated_increment_advances_one_at_a_time_and_resets() {
        let mut version = Version::default();

        version.inc_patch();
        assert_eq!(version, Version::new(0, 0, 1));

        version.inc_patch();
        assert_eq!(version, Version::new(0, 0, 2));

        version.inc_minor();
        assert_eq!(version, Version::new(0, 1, 0));

        version.inc_major();
        assert_eq!(version, Version::new(1, 0, 0));
    }

    #[test]
    fn patch_change_does_not_invalidate_patch_independant() {
        let independant = Version::new(1, 4, 2);
        let current0 = Version::new(1, 4, 9);

        assert!(independant.eq_minor(&current0));

        let current1 = Version::new(1, 5, 0);

        assert!(!independant.eq_minor(&current1));
    }
}
