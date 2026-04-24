use std::marker::PhantomData;

/// A pool that offers no parallelism, this is the same as just running the passed in function.
pub struct SeqPool<'a>(PhantomData<&'a ()>);

impl<'a> SeqPool<'a> {
    /// Creates a new `SequentialPool`.
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }

    /// Submit a job to be executed.
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        f()
    }
}
