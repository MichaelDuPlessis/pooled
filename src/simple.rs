use crate::Runtime;

/// A simple fire-and-forget thread pool.
pub struct SimplePool<'a> {
    runtime: &'a Runtime,
}

impl<'a> SimplePool<'a> {
    /// Creates a new `SimplePool` from a `Runtime`.
    pub(crate) fn new(runtime: &'a Runtime) -> Self {
        Self { runtime }
    }

    /// Submit a job to be executed.
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.runtime.send_job(f);
    }
}
