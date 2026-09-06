// Constants used throughout the library.
// Definitions. One should not change these, but if one were to change them, it would be here.

/// The size of the World Grid.
pub const WORLD_SIZE_USZ: usize = 1 << 32;

/// The size of the Chunk Grid.
pub const CHUNK_SIZE_USZ: usize = 32;

// Convenience wrappers.

/// The size of the World Grid towards positive values.
pub const WORLD_POS_I32: i32 = ((WORLD_SIZE_USZ >> 1) - 1) as i32;

/// The size of the World Grid towards negative values.
pub const WORLD_NEG_I32: i32 = -1 - WORLD_POS_I32;

/// The size of the Chunk Grid.
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE_USZ as i32;

/// The size of the Chunk Grid.
pub const CHUNK_SIZE_U8: u8 = CHUNK_SIZE_USZ as u8;
