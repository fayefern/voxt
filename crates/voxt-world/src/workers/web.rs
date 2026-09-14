/*
use flume::{Receiver, Sender, unbounded};
use wasm_bindgen_futures::spawn_local;

pub fn spawn_worker_pool(_thread_count: usize) -> (Sender<ChunkTask>, Receiver<ChunkTaskResult>) {
    let (task_tx, task_rx) = unbounded::<ChunkTask>();
    let (result_tx, result_rx) = unbounded::<ChunkTaskResult>();

    spawn_local(async move {
        while let Ok(task) = task_rx.recv_async().await {
            let result = process_task_async(task).await;
            if result_tx.send_async(result).await.is_err() {
                break;
            }
        }
    });

    (task_tx, result_rx)
}
*/
