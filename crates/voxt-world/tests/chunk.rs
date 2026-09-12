use voxt_core::{
    bat,
    prelude::{BatVoxel, BlockId, ChunkPos, Pos, VoxelPos, constants::AIR},
};

use voxt_world::prelude::Chunk;

#[test]
fn public_chunk_api_supports_read_modify_read() {
    let mut chunk = Chunk::new(ChunkPos::from_raw(4, -2, 7));

    let pos = VoxelPos::from_raw(10, 20, 30);
    let stone = BlockId::new(1);

    assert_eq!(chunk.get(pos).id(), AIR);

    let voxels = bat!( Ck 10,20,30 => stone.id()).to_vec();

    let mut output = vec![BatVoxel::new(pos, AIR); voxels.len()];
    chunk.set_bulk(&voxels, &mut output);

    assert_eq!(output[0].id(), AIR);
    assert_eq!(chunk.get(pos).id(), stone);
    assert_eq!(chunk.chunk_pos().clone(), ChunkPos::from_raw(4, -2, 7));
}

#[test]
fn bulk_set_applies_duplicate_coordinates_in_order() {
    let mut chunk = Chunk::new(ChunkPos::from_raw(0, 0, 0));
    let pos = VoxelPos::from_raw(10, 20, 30);
    let requests = bat!(
        Ck 10,20,30 => 1,
        10,20,30 => 2,
    );
    let mut output = vec![BatVoxel::new(pos, AIR); requests.len()];

    chunk.set_bulk(requests, &mut output);

    assert_eq!(output[0].id(), AIR);
    assert_eq!(output[1].id(), BlockId::new(1));
    assert_eq!(chunk.get(pos).id(), BlockId::new(2));
}
