use crate::{
    coords::{Pos, VoxelPos},
    ids::BlockId,
};

pub struct BlockAt<T: Pos> {
    pos: T,
    id: BlockId,
}
