use flume::{Receiver, Sender, unbounded};
use std::thread;

use crate::chunks::{Chunk, ChunkTask, ChunkTaskKind, ChunkTaskResult};

pub fn spawn_worker_pool(
    thread_count: Option<usize>,
) -> (Sender<ChunkTask>, Receiver<ChunkTaskResult>) {
    let (task_tx, task_rx) = unbounded::<ChunkTask>();
    let (result_tx, result_rx) = unbounded::<ChunkTaskResult>();

    let parallelism = thread_count.unwrap_or_else(|| {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    });

    let worker_count = if parallelism < 8 {
        parallelism.saturating_sub(1).max(1)
    } else {
        parallelism - 2
    };

    for thread_id in 0..worker_count {
        let rx = task_rx.clone();
        let tx = result_tx.clone();

        thread::Builder::new()
            .name(format!("voxt-worker-{}", thread_id))
            .spawn(move || {
                while let Ok(task) = rx.recv() {
                    let result = process_task_sync(task);

                    if tx.send(result).is_err() {
                        break;
                    }
                }
            })
            .expect("failed to spawn worker thread");
    }

    (task_tx, result_rx)
}

fn process_task_sync(task: ChunkTask) -> ChunkTaskResult {
    let key = task.key();

    match key.kind() {
        ChunkTaskKind::Load => {
            let chunk = Chunk::new(key.chunk_pos());
            ChunkTaskResult::Loaded { key, chunk }
        }
        ChunkTaskKind::Mesh => {
            let mesh = true;
            ChunkTaskResult::Meshed { key, mesh }
        }
        ChunkTaskKind::Save => ChunkTaskResult::Saved { key },
        ChunkTaskKind::Unload => ChunkTaskResult::Unloaded { key },
    }
}
