/// Macro to create a [Voxel] from at least one position and at least one id.
/// This macro will not accept either of the (x, y, z) coordinates being out of bounds.
///
/// # Examples
///
/// ```
/// # use voxt_core::prelude::{VoxelPos, BlockId};
/// # use voxt_world::prelude::{vox, Voxel};
/// // voxels can be build from raw coordinates and ids...
/// let voxel_a: &[Voxel] = vox!(0, 0, 0 => 24);
///
/// // ...or from `VoxelPos` and `BlockId` types.
/// let voxel_b = vox!(VoxelPos::from_raw_checked(1, 2, 3) => BlockId::new(24));
///
/// assert_eq!(voxel_a[0].id(), voxel_b[0].id());
/// ```
#[macro_export]
macro_rules! vox {
    ( $( $x:expr, $y:expr, $z:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                {
                    const _: () = {
                        if $x >= 32 || $y >= 32 || $z >= 32 {
                            panic!("VoxelPos within a Chunk must be strictly lesser than the Chunk's size (i.e. 32)");
                        }
                    };

                    Voxel::new(VoxelPos::from_raw($x, $y, $z), BlockId::new($id))
                },
            )+
        ] as &[Voxel]
    };
    ( $( $pos:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                    Voxel::new($pos, $id),
            )+
        ] as &[Voxel]
    };
}

use voxt_core::prelude::{BlockId, VoxelPos};

/// A [Voxel] couples a [VoxelPos] with a [BlockId]. It is the basic unit of a [Chunk](crate::prelude::Chunk).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel {
    pos: VoxelPos,
    id: BlockId,
}

impl Voxel {
    /// Creates a new [Voxel] from a [VoxelPos] and a [BlockId].
    ///
    /// Prefer this function over [new_bulk](Voxel::new_bulk) and [vox!] when creating a single Voxel.
    pub const fn new(pos: VoxelPos, id: BlockId) -> Self {
        Self { pos, id }
    }

    /// Creates a boxed array of [Voxel] from a slice of [VoxelPos]a and a slice of [BlockId].
    /// Avoid using this function, use the [vox!] macro instead.
    ///
    /// # Panics
    ///
    /// This function will panic if the length of the two slices differs, or if either of them is empty.
    pub fn new_bulk(pos: &[VoxelPos], ids: &[BlockId]) -> Box<[Self]> {
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

    /// Returns the [VoxelPos] of the [Voxel].
    pub const fn pos(&self) -> VoxelPos {
        self.pos
    }

    /// Returns the [BlockId] of the [Voxel].
    pub const fn id(&self) -> BlockId {
        self.id
    }
}
