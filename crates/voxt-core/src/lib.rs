mod constants;
mod coords;
mod ids;

pub mod prelude {
    pub mod constants {
        pub use super::super::constants::{
            AIR, CHUNK_SIZE, CHUNK_SIZE_LOG, WORLD_HEIGHT, WORLD_HEIGHT_IN_CHUNKS,
            WORLD_HEIGHT_IN_CHUNKS_LOG, WORLD_HEIGHT_LOG, WORLD_SIZE, WORLD_SIZE_IN_CHUNKS,
            WORLD_SIZE_IN_CHUNKS_LOG, WORLD_SIZE_LOG,
        };
    }
    pub use super::{
        coords::{ChunkPos, Pos, PosError, VoxelPos, WorldPos},
        ids::{BlockId, ChunkVersion},
    };
}
