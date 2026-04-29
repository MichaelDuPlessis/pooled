use crate::{PoolError, Runtime};
use lockout::channel::oneshot;

/// A handle to a submitted job's result.
///
/// Call [`Task::wait`] to block until the job completes and retrieve its result.
pub struct Task<T>(oneshot::Receiver<Result<T, PoolError>>);

impl<T> Task<T> {
    /// Block until the job completes and return its result.
    ///
    /// Returns `Err(PoolError::JobPanic)` if the job panicked, or
    /// `Err(PoolError::ChannelClosed)` if the result channel was unexpectedly closed.
    pub fn wait(self) -> Result<T, PoolError> {
        self.0.recv().map_err(|_| PoolError::ChannelClosed)?
    }
}

/// A thread pool that returns a [`Task`] handle for each submitted job.
///
/// Unlike [`SimplePool`](crate::simple::SimplePool) which is fire-and-forget,
/// `FuturePool` allows retrieving the result of a job at a later point.
pub struct FuturePool<'a> {
    runtime: &'a Runtime,
}

impl<'a> FuturePool<'a> {
    /// Creates a new `FuturePool` from a `Runtime`.
    pub(crate) fn new(runtime: &'a Runtime) -> Self {
        Self { runtime }
    }

    /// Submit a job to be executed on the thread pool.
    ///
    /// Returns a [`Task`] that can be used to retrieve the result later.
    pub fn submit<F, T>(&self, f: F) -> Task<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let job = || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
                .map_err(PoolError::JobPanic);
            let _ = tx.send(result);
        };
        self.runtime.simple_pool().submit(job);

        Task(rx)
    }
}
