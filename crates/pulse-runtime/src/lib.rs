use std::future::Future;

use gpui::{App, AppContext, Global, Task};

pub use tokio::task::JoinError;

struct GlobalTokio {
    owned_runtime: Option<tokio::runtime::Runtime>,
    handle: tokio::runtime::Handle,
}

impl Global for GlobalTokio {}

impl Drop for GlobalTokio {
    fn drop(&mut self) {
        if let Some(runtime) = self.owned_runtime.take() {
            runtime.shutdown_background();
        }
    }
}

struct AbortOnDrop(tokio::task::AbortHandle);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

fn default_worker_threads() -> usize {
    std::thread::available_parallelism().map_or(4, |count| count.get().max(2))
}

/// Initializes a tokio runtime and registers it as a gpui global
///
/// # Errors
///
/// Returns an error if the tokio runtime could not be created
pub fn init(cx: &mut App) -> Result<(), std::io::Error> {
    init_with_workers(cx, default_worker_threads())
}

/// Initializes a multi-threaded tokio runtime with the given worker thread count
///
/// # Errors
///
/// Returns an error if the tokio runtime could not be created
pub fn init_with_workers(cx: &mut App, worker_threads: usize) -> Result<(), std::io::Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()?;

    let handle = runtime.handle().clone();
    cx.set_global(GlobalTokio {
        owned_runtime: Some(runtime),
        handle,
    });

    Ok(())
}

/// Initializes the tokio wrapper using an existing runtime handle
pub fn init_from_handle(cx: &mut App, handle: tokio::runtime::Handle) {
    cx.set_global(GlobalTokio {
        owned_runtime: None,
        handle,
    });
}

pub struct Tokio;

impl Tokio {
    /// Spawns a future on tokio's thread pool and returns a gpui task
    ///
    /// The tokio task is cancelled if the returned gpui task is dropped
    pub fn spawn<C, Fut, R>(cx: &C, future: Fut) -> Task<Result<R, JoinError>>
    where
        C: AppContext,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.handle.spawn(future);
            let abort_handle = join_handle.abort_handle();
            let cancel = AbortOnDrop(abort_handle);
            cx.background_spawn(async move {
                let result = join_handle.await;
                drop(cancel);
                result
            })
        })
    }

    /// Spawns a fallible future on tokio's thread pool and returns a gpui task
    pub fn spawn_result<C, Fut, R>(cx: &C, future: Fut) -> Task<anyhow::Result<R>>
    where
        C: AppContext,
        Fut: Future<Output = anyhow::Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.handle.spawn(future);
            let abort_handle = join_handle.abort_handle();
            let cancel = AbortOnDrop(abort_handle);
            cx.background_spawn(async move {
                let result = join_handle.await;
                drop(cancel);
                result?
            })
        })
    }

    /// Spawns blocking work on tokio's blocking thread pool and returns a gpui task
    pub fn spawn_blocking<C, F, R>(cx: &C, function: F) -> Task<Result<R, JoinError>>
    where
        C: AppContext,
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        cx.read_global(|tokio: &GlobalTokio, cx| {
            let join_handle = tokio.handle.spawn_blocking(function);
            let abort_handle = join_handle.abort_handle();
            let cancel = AbortOnDrop(abort_handle);
            cx.background_spawn(async move {
                let result = join_handle.await;
                drop(cancel);
                result
            })
        })
    }

    pub fn handle<C: AppContext>(cx: &C) -> tokio::runtime::Handle {
        cx.read_global(|tokio: &GlobalTokio, _| tokio.handle.clone())
    }
}
