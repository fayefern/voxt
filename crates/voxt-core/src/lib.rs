mod constants;
mod ids;
mod pos;

pub mod prelude {
    pub mod constants {
        pub use super::super::constants::{
            AIR, CHUNK_LOG_USZ, CHUNK_SIZE_I32, CHUNK_SIZE_U8, CHUNK_SIZE_USZ, WORLD_NEG_I32,
            WORLD_POS_I32, WORLD_SIZE_USZ,
        };
    }
    pub use super::{
        ids::{BlockId, ChunkVersion},
        pos::{ChunkPos, VoxelPos, WorldPos},
    };
}
