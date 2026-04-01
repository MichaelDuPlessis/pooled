/// A pool that offers no parallelism, this is the same as just running the passed in function.
pub struct SeqPool;

impl SeqPool {
    /// Creates a new `SequentialPool`.
    pub(crate) fn new() -> Self {
        Self
    }

    /// Submit a job to be executed.
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        f()
    }
}
