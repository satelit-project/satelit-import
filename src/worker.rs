pub mod dump;

use futures::prelude::*;
use log::info;
use tokio::runtime;

use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{mpsc, Mutex};
use std::thread;

/// Represents a "worker" that may perform any task on background
pub trait Worker: Send + 'static {
    /// Returns a future (also known as tokio's task) that performs any kind of work
    ///
    /// This future will be spawned on a separate tokio runtime.
    fn task(self: Box<Self>) -> Box<dyn Future<Item = (), Error = ()> + Send + 'static>;
}

/// Starts a thread for running instances of `Worker` thread
///
/// This function must be called only once before running actix runtime. Once actix runtime is
/// shutdown, the `shutdown_worker_thread` must be called before exiting the process.
pub fn start_worker_thread() {
    assert_eq!(WORKER_DATA.load(Ordering::SeqCst), 0 as *mut WorkerData);

    let (tx, rx) = mpsc::channel::<Box<dyn Worker>>();
    let handle = thread::spawn(move || {
        let mut runtime = make_tokio_runtime();

        for worker in rx {
            let task = worker.task();
            runtime.spawn(task);
        }

        runtime
            .shutdown_on_idle()
            .wait()
            .expect("failed to shutdown tokio runtime");

        info!("Worker thread is going to shutdown");
    });

    let tx = Mutex::new(tx);
    let data = WorkerData { handle, tx };
    WORKER_DATA.store(Box::into_raw(Box::new(data)), Ordering::SeqCst);
}

/// Spawns a worker on a background thread thus starting the worker's task
///
/// An error is returned in case if it's not possible to run the worker right now. This function
/// must not be called before worker thread are started or after it has been shut down.
pub fn spawn_worker<W: Worker>(worker: W) -> Result<(), impl std::error::Error> {
    let ptr = WORKER_DATA.load(Ordering::SeqCst);
    assert_ne!(ptr, 0 as *mut WorkerData);

    let boxed = Box::new(worker);
    let worker_data = unsafe { &*ptr };
    let tx = worker_data.tx.lock().unwrap().clone();

    tx.send(boxed)
}

/// Shuts down worker thread and waits until all current workers are finished
///
/// This function must be called only once from the same thread where it has started. Moreover,
/// it's not allowed to call `spawn_worker` until the worker thread is started again.
pub fn shutdown_worker_thread() {
    let ptr = WORKER_DATA.load(Ordering::SeqCst);
    assert_ne!(ptr, 0 as *mut WorkerData);

    let worker_data = unsafe { Box::from_raw(ptr) };
    WORKER_DATA.store(0 as *mut WorkerData, Ordering::SeqCst);

    worker_data
        .handle
        .join()
        .expect("failed to shutdown worker thread");
}

/// Global storage for worker thread's specific data
static WORKER_DATA: AtomicPtr<WorkerData> = AtomicPtr::new(0 as *mut WorkerData);

/// Data that's related to worker thread
struct WorkerData {
    /// Thread join handle
    handle: thread::JoinHandle<()>,

    /// Sender part of mpsc channel to send worker instances to worker thread
    tx: Mutex<mpsc::Sender<Box<dyn Worker>>>,
}

/// Creates tokio runtime for spawning a worker's tasks
fn make_tokio_runtime() -> runtime::Runtime {
    runtime::Builder::new()
        .blocking_threads(2)
        .core_threads(2)
        .name_prefix("satelit-worker-")
        .build()
        .expect("failed to initialize tokio runtime for workers")
}
