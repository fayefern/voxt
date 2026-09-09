use crate::ids::BlockId;

// Constants used throughout the library.

// Definitions. One should not change these, but if one were to change them, it would be here.

/// The log2 of the size of the World Grid.
pub const WORLD_SIZE_LOG: usize = 20;

/// The size of the World Grid.
pub const WORLD_SIZE: usize = 1 << WORLD_SIZE_LOG;

/// The log2 of the height of the World Grid.
pub const WORLD_HEIGHT_LOG: usize = 10;

/// The height of the World Grid.
pub const WORLD_HEIGHT: usize = 1 << WORLD_HEIGHT_LOG;

/// The log2 of the size of the Chunk Grid.
pub const CHUNK_SIZE_LOG: usize = 6;

/// The size of the Chunk Grid.
pub const CHUNK_SIZE: usize = 1 << CHUNK_SIZE_LOG;

/// The log2 of the size of the World Grid counted in Chunks.
pub const WORLD_SIZE_IN_CHUNKS_LOG: usize = WORLD_SIZE_LOG - CHUNK_SIZE_LOG;

/// The size of the World Grid counted in Chunks.
pub const WORLD_SIZE_IN_CHUNKS: usize = WORLD_SIZE / CHUNK_SIZE;

/// The log2 of the height of the World Grid counted in Chunks.
pub const WORLD_HEIGHT_IN_CHUNKS_LOG: usize = WORLD_HEIGHT_LOG - CHUNK_SIZE_LOG;

/// The height of the World Grid counted in Chunks.
pub const WORLD_HEIGHT_IN_CHUNKS: usize = WORLD_HEIGHT / CHUNK_SIZE;

/// BlockId of value 0 is reserved for "Air". One should never need to use BlockId::new(0) directly. Use this instead
pub const AIR: BlockId = BlockId::new(0);
