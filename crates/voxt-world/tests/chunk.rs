use voxt_core::{
    prelude::{BlockId, ChunkPos, ChunkVoxel, VoxelPos},
    vox,
};

use voxt_world::prelude::Chunk;

#[test]
fn public_chunk_api_supports_read_modify_read() {
    let mut chunk = Chunk::new(ChunkPos::from_raw(4, -2, 7));

    let pos = VoxelPos::from_raw_checked(10, 20, 30);
    let stone = BlockId::new(1);

    assert_eq!(chunk.get(pos).id(), BlockId::new(0));

    let voxels = vox!( C 10,20,30 => stone.id());

    let old = chunk.set_bulk(voxels).unwrap();

    assert_eq!(old[0].id(), BlockId::new(0));
    assert_eq!(chunk.get(pos).id(), stone);
    assert_eq!(chunk.chunk_pos(), ChunkPos::from_raw(4, -2, 7));
}
