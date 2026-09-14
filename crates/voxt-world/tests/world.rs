use voxt_core::prelude::{ChunkPos, Pos, Version};
use voxt_world::prelude::{
    ChunkDemandKind, ChunkSchedulerConfig, Tick, World, workers::spawn_worker_pool,
};

fn load_event(world: &mut World, pos: ChunkPos) {
    let tick = world.tick();
    world
        .schedule_mut()
        .add(ChunkDemandKind::Load, pos, Version::new(0, 0, 0), tick);
}

fn unload_event(world: &mut World, pos: ChunkPos) {
    let tick = world.tick();
    let ver = world
        .chunks()
        .get_chunk(&pos)
        .map_or_else(|| Version::new(0, 0, 0), |c| c.version());
    world
        .schedule_mut()
        .add(ChunkDemandKind::Unload, pos, ver, tick);
}

fn advance_until_loaded(world: &mut World, positions: &[ChunkPos]) {
    for _ in 0..100 {
        if positions
            .iter()
            .all(|pos| world.chunks().contains_chunk(pos))
        {
            return;
        }

        world.advance();
    }

    panic!("worker pool did not load the requested chunks");
}

fn advance_until_unloaded(world: &mut World, positions: &[ChunkPos]) {
    for _ in 0..100 {
        if positions
            .iter()
            .all(|pos| !world.chunks().contains_chunk(pos))
        {
            return;
        }

        world.advance();
    }

    panic!("worker pool did not unload the requested chunks");
}

#[test]
fn world_demands_and_receives_load_chunk() {
    let (task_tx, result_rx) = spawn_worker_pool(None);

    let mut world = World::new(ChunkSchedulerConfig::new(Tick::new(10)), task_tx, result_rx);

    let pos0 = ChunkPos::from_raw(0, 0, 0);
    let pos1 = ChunkPos::from_raw(10, 10, 10);
    let pos2 = ChunkPos::from_raw(20, 20, 20);

    load_event(&mut world, pos0);
    load_event(&mut world, pos1);
    load_event(&mut world, pos2);

    world.advance();

    unload_event(&mut world, pos0);
    unload_event(&mut world, pos1);

    advance_until_loaded(&mut world, &[pos0, pos1, pos2]);

    assert!(world.chunks().contains_chunk(&pos0));
    assert!(world.chunks().contains_chunk(&pos1));
    assert!(world.chunks().contains_chunk(&pos2));

    advance_until_unloaded(&mut world, &[pos0, pos1]);

    assert!(!world.chunks().contains_chunk(&pos0));
    assert!(!world.chunks().contains_chunk(&pos1));
    assert!(world.chunks().contains_chunk(&pos2));
}
