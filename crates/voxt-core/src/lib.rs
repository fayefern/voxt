pub(crate) mod config;
pub(crate) mod ids;
pub(crate) mod pos;

pub mod prelude {
    pub mod constants {
        pub use crate::config::{CHUNK_SIZE_I32, CHUNK_SIZE_U8, CHUNK_SIZE_USZ};
    }
    pub use crate::{
        ids::{block_id::BlockId, chunk_version::ChunkVersion},
        pos::{ChunkPos, VoxelPos, WorldPos},
    };
}
