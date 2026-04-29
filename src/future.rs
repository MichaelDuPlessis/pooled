use crate::{PoolError, Runtime};
use lockout::channel::oneshot;

pub struct Task<T>(oneshot::Receiver<Result<T, PoolError>>);

impl<T> Task<T> {
    pub fn wait(self) -> Result<T, PoolError> {
        self.0.recv().map_err(|_| PoolError::ChannelClosed)?
    }
}

pub struct FuturePool<'a> {
    runtime: &'a Runtime,
}

impl<'a> FuturePool<'a> {
    pub(crate) fn new(runtime: &'a Runtime) -> Self {
        Self { runtime }
    }

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
