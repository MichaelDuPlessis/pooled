use std::marker::PhantomData;

use crate::PoolError;

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

    /// Map `f` over `inputs` sequentially.
    pub fn map<T, R, F>(&self, inputs: &[T], f: F) -> Vec<Result<R, PoolError>>
    where
        F: Fn(&T) -> R,
    {
        inputs.iter().map(|x| Ok(f(x))).collect()
    }
}
