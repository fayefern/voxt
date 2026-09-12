use std::fmt::Display;

use crate::prelude::{BlockId, ChunkPos, Pos, VoxelPos, WorldPos};

#[macro_export]
macro_rules! bat {
    ( Ck $($x:expr, $y:expr, $z:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                {
                    const _: () = {
                        if $x >= $crate::prelude::VoxelPos::MAX || $x < $crate::prelude::VoxelPos::MIN
                        || $y >= $crate::prelude::VoxelPos::MAX_HEIGHT || $y < $crate::prelude::VoxelPos::MIN_HEIGHT
                        || $z >= $crate::prelude::VoxelPos::MAX || $z < $crate::prelude::VoxelPos::MIN {
                            panic!("VoxelPos must be strictly lesser than the Chunk's dimensions");
                        }
                    };

                    $crate::prelude::BatVoxel::new($crate::prelude::VoxelPos::from_raw($x, $y, $z), $crate::prelude::BlockId::new($id))
                },
            )+
        ] as &[$crate::prelude::BatVoxel]
    };

    ( Wd $($x:expr, $y:expr, $z:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                {
                    const _: () = {
                        if $x >= $crate::prelude::WorldPos::MAX || $x < $crate::prelude::WorldPos::MIN
                        || $y >= $crate::prelude::WorldPos::MAX_HEIGHT || $y < $crate::prelude::WorldPos::MIN_HEIGHT
                        || $z >= $crate::prelude::WorldPos::MAX || $z < $crate::prelude::WorldPos::MIN {
                            panic!("WorldPos must be strictly lesser than the World's dimensions")
                        }
                    };

                    $crate::prelude::BatWorld::new($crate::prelude::WorldPos::from_raw($x, $y, $z), $crate::prelude::BlockId::new($id))
                },
            )+
        ] as &[$crate::prelude::BatWorld]
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockAt<P: Pos> {
    pos: P,
    id: BlockId,
}

impl<P: Pos> BlockAt<P> {
    pub const fn new(pos: P, id: BlockId) -> Self {
        Self { pos, id }
    }

    pub fn new_bulk(poss: &[P], ids: &[BlockId]) -> Vec<Self> {
        assert!(
            poss.len() == ids.len(),
            "cannot create bulk BlockAt from slices of differing len"
        );
        assert!(
            !poss.is_empty(),
            "cannot create bulk BlockAt from empty slice"
        );

        poss.iter()
            .zip(ids.iter())
            .map(|(&pos, &id)| Self { pos, id })
            .collect()
    }

    pub const fn pos(&self) -> P {
        self.pos
    }

    pub const fn id(&self) -> BlockId {
        self.id
    }
}

impl BlockAt<ChunkPos> {
    pub fn into_world_bat(&self, voxel_pos: &VoxelPos) -> BlockAt<WorldPos> {
        BlockAt {
            pos: self.pos.into_world_pos(voxel_pos),
            id: self.id,
        }
    }
}

impl BlockAt<VoxelPos> {
    pub fn into_world_bat(&self, chunk_pos: &ChunkPos) -> BlockAt<WorldPos> {
        BlockAt {
            pos: self.pos.into_world_pos(chunk_pos),
            id: self.id,
        }
    }
}

impl From<BlockAt<WorldPos>> for BlockAt<ChunkPos> {
    fn from(world_block: BlockAt<WorldPos>) -> Self {
        Self {
            pos: world_block.pos.into_chunk_pos(),
            id: world_block.id,
        }
    }
}

impl From<&BlockAt<WorldPos>> for BlockAt<ChunkPos> {
    fn from(world_block: &BlockAt<WorldPos>) -> Self {
        Self {
            pos: world_block.pos.into_chunk_pos(),
            id: world_block.id,
        }
    }
}

impl From<BlockAt<WorldPos>> for BlockAt<VoxelPos> {
    fn from(world_block: BlockAt<WorldPos>) -> Self {
        Self {
            pos: world_block.pos.into_voxel_pos(),
            id: world_block.id,
        }
    }
}

impl From<&BlockAt<WorldPos>> for BlockAt<VoxelPos> {
    fn from(world_block: &BlockAt<WorldPos>) -> Self {
        Self {
            pos: world_block.pos.into_voxel_pos(),
            id: world_block.id,
        }
    }
}

impl<T: Pos> Display for BlockAt<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} => {}", self.pos, self.id)
    }
}

pub type BatVoxel = BlockAt<VoxelPos>;

pub type BatWorld = BlockAt<WorldPos>;
