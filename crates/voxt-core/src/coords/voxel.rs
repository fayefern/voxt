/*

/// Macro to create a [Voxel] from at least one position and at least one id.
/// This macro will not accept either of the (x, y, z) coordinates being out of bounds.
///
/// # Examples
///
/// ```
/// # use voxt_core::prelude::{WorldVoxel, ChunkVoxel, WorldPos, VoxelPos, BlockId};
/// # use voxt_core::vox;
/// let voxel_a: &[ChunkVoxel] = vox!( C 0, 0, 0 => 24 );
///
/// let voxel_b = vox!( W 64, 64, 64 => 24 );
///
/// assert_eq!(voxel_a[0].id(), voxel_b[0].id());
/// assert_eq!(voxel_a[0].pos(), voxel_b[0].pos().into());
/// ```
#[macro_export]
macro_rules! vox {
    ( C $($x:expr, $y:expr, $z:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                {
                    const _: () = {
                        if $x >= 32 || $y >= 32 || $z >= 32 {
                            panic!("VoxelPos within a Chunk must be strictly lesser than the Chunk's size (i.e. 32)");
                        }
                    };

                    ChunkVoxel::new(VoxelPos::from_raw($x, $y, $z), BlockId::new($id))
                },
            )+
        ] as &[ChunkVoxel]
    };

    ( W $($x:expr, $y:expr, $z:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                {
                    WorldVoxel::new(WorldPos::from_raw($x, $y, $z), BlockId::new($id))
                },
            )+
        ] as &[WorldVoxel]
    };
}

use std::fmt::Display;

use crate::{
    coords::ChunkPos,
    prelude::{BlockId, VoxelPos, WorldPos},
};

/// A [Voxel] couples a [VoxelPos], or a [WorldPos], with a [BlockId]. It is not meant to be used on it's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel<T: Copy> {
    pos: T,
    id: BlockId,
}

impl<T: Copy> Voxel<T> {
    /// Creates a new [Voxel] from a [VoxelPos], or a [WorldPos], and a [BlockId].
    ///
    /// Prefer this function over [new_bulk](Voxel::new_bulk) and [vox!] when creating a single Voxel.
    pub const fn new(pos: T, id: BlockId) -> Self {
        Self { pos, id }
    }

    /// Creates a boxed array of [Voxel] from a slice of [VoxelPos], or [WorldPos], and a slice of [BlockId].
    /// Avoid using this function for hand-writen [Voxel]s, use the [vox!] macro instead.
    ///
    /// # Panics
    ///
    /// This function will panic if the length of the two slices differs, or if either of them is empty.
    pub fn new_bulk(pos: &[T], ids: &[BlockId]) -> Box<[Self]> {
        assert!(!pos.is_empty(), "Voxel array cannot be empty");
        assert_eq!(
            pos.len(),
            ids.len(),
            "Position and id arrays must be of the same length"
        );

        pos.iter()
            .zip(ids.iter())
            .map(|(&pos, &id)| Voxel { pos, id })
            .collect()
    }

    /// Returns the position of the [Voxel].
    pub const fn pos(&self) -> T {
        self.pos
    }

    /// Returns the [BlockId] of the [Voxel].
    pub const fn id(&self) -> BlockId {
        self.id
    }
}

/// A [Voxel] with its position relative to it's Chunk).
pub type ChunkVoxel = Voxel<VoxelPos>;

impl Voxel<VoxelPos> {
    /// Converts the [ChunkVoxel] to a [WorldVoxel].
    pub const fn to_world(&self, chunk_pos: &ChunkPos) -> WorldVoxel {
        WorldVoxel {
            id: self.id,
            pos: WorldPos::from_chunk_and_voxel(chunk_pos, &self.pos),
        }
    }

    /// Converts the [WorldVoxel] to a [ChunkVoxel]
    pub const fn from_world(world_voxel: WorldVoxel) -> Self {
        world_voxel.to_chunk()
    }
}

impl From<WorldVoxel> for ChunkVoxel {
    /// Converts the [WorldVoxel] to a [ChunkVoxel].
    fn from(value: WorldVoxel) -> Self {
        value.to_chunk()
    }
}

impl Display for ChunkVoxel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y, z) = self.pos.all();

        write!(f, "ck({}, {}, {})", x, y, z)
    }
}

/// A [Voxel] with its position relative to it's World.
pub type WorldVoxel = Voxel<WorldPos>;

impl Voxel<WorldPos> {
    /// Converts the [WorldVoxel] to a [ChunkVoxel].
    pub const fn to_chunk(&self) -> ChunkVoxel {
        ChunkVoxel {
            id: self.id,
            pos: VoxelPos::from_world(&self.pos),
        }
    }

    /// Converts the [ChunkVoxel] to a [WorldVoxel].
    pub const fn from_chunk(chunk_voxel: ChunkVoxel, chunk_pos: &ChunkPos) -> Self {
        chunk_voxel.to_world(chunk_pos)
    }
}

impl Display for WorldVoxel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y, z) = self.pos.all();

        write!(f, "wd({}, {}, {})", x, y, z)
    }
}
 */
