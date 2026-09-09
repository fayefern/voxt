use std::fmt::Display;

use crate::{
    coords::{ChunkPos, Pos, VoxelPos, WorldPos},
    ids::BlockId,
};

#[macro_export]
macro_rules! bat {
    ( C $($x:expr, $y:expr, $z:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                {
                    const _: () = {
                        if $x >= VoxelPos::MAX || $y >= VoxelPos::MAX || $z >= VoxelPos::MAX {
                            panic!("VoxelPos must be strictly lesser than the Chunk's dimensions");
                        }
                    };

                    AtChunk::new(VoxelPos::from_raw($x, $y, $z), BlockId::new($id))
                },
            )+
        ] as &[AtChunk]
    };

    ( W $($x:expr, $y:expr, $z:expr => $id:expr ),+ $(,)? ) => {
        &[
            $(
                {
                    const _: () = {
                        if $x >= WorldPos::MAX || $x < WorldPos::MIN
                        || $y >= WorldPos::MAX_HEIGHT || $y < WorldPos::MIN_HEIGHT
                        || $z >= WorldPos::MAX || $z < WorldPos::MIN {
                            panic!("WorldPos must be strictly lesser than the World's dimensions")
                        }
                    };

                    AtWorld::new(WorldPos::from_raw($x, $y, $z), BlockId::new($id))
                },
            )+
        ] as &[AtWorld]
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockAt<T: Pos> {
    pos: T,
    id: BlockId,
}

impl<T: Pos> BlockAt<T> {
    pub const fn new(pos: T, id: BlockId) -> Self {
        Self { pos, id }
    }

    pub fn new_bulk(poss: &[T], ids: &[BlockId]) -> Box<[Self]> {
        debug_assert!(
            !poss.is_empty() && !ids.is_empty(),
            "empty slices used to create BlockAt"
        );
        debug_assert!(
            poss.len() == ids.len(),
            "slices of differing lengths used to create BlockAt"
        );

        poss.iter()
            .zip(ids.iter())
            .map(|(&p, &i)| Self { pos: p, id: i })
            .collect()
    }

    pub const fn pos(&self) -> T {
        self.pos
    }

    pub const fn id(&self) -> BlockId {
        self.id
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

pub type AtChunk = BlockAt<VoxelPos>;

pub type AtWorld = BlockAt<WorldPos>;
