use voxt_core::prelude::{BlockId, ChunkPos, VoxelPos};

use voxt_world::{
    prelude::{Chunk, Voxel},
    vox,
};

#[test]
fn public_chunk_api_supports_read_modify_read() {
    let mut chunk = Chunk::new(ChunkPos::from_raw(4, -2, 7));

    let pos = VoxelPos::from_raw_checked(10, 20, 30);
    let stone = BlockId::new(1);

    assert_eq!(chunk.get(pos).id(), BlockId::new(0));

    let old = chunk.set_bulk(vox!(pos => stone)).unwrap();

    assert_eq!(old[0].id(), BlockId::new(0));
    assert_eq!(chunk.get(pos).id(), stone);
    assert_eq!(chunk.chunk_pos(), ChunkPos::from_raw(4, -2, 7));
}
