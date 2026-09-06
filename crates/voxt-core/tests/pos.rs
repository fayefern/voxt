use voxt_core::prelude::{ChunkPos, VoxelPos, WorldPos};

#[test]
fn public_position_api_supports_basic_world_coordinates() {
    let world = WorldPos::from_raw(-33, 12, 64);

    let chunk = ChunkPos::from(world);
    let voxel = VoxelPos::from(world);

    assert_eq!(chunk, ChunkPos::from_raw(-2, 0, 2));
    assert_eq!(voxel, VoxelPos::from_raw_checked(31, 12, 0));

    let reconstructed = WorldPos::from((chunk, voxel));

    assert_eq!(reconstructed, world);
}
