use std::sync::{Arc, mpsc};

use crate::{PoolError, Runtime, simple::SimplePool};

/// A thread pool for mapping functions over input lists.
pub struct MapPool<'a> {
    pool: SimplePool<'a>,
}

impl<'a> MapPool<'a> {
    /// Creates a new `MapPool` from a `Runtime`.
    pub(crate) fn new(runtime: &'a Runtime) -> Self {
        Self {
            pool: runtime.simple_pool(),
        }
    }

    /// Map a function over a list of inputs, returning results in order.
    pub fn map<T, R, F>(&self, inputs: Vec<T>, f: F) -> Vec<Result<R, PoolError>>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let len = inputs.len();

        let f = Arc::new(f);
        for (i, input) in inputs.into_iter().enumerate() {
            let f = Arc::clone(&f);
            let sender = sender.clone();

            self.pool.submit(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(input)));
                let _ = sender.send((i, result));
            });
        }

        // remove last sender as if cloned senders drop due to panic then recv will block forever
        drop(sender);

        let mut results: Vec<Option<Result<R, PoolError>>> = (0..len).map(|_| None).collect();

        receiver.iter().for_each(|(i, res)| match res {
            Ok(value) => results[i] = Some(Ok(value)),
            Err(panic) => results[i] = Some(Err(PoolError::JobPanic(panic))),
        });

        results
            .into_iter()
            .map(|r| r.unwrap_or(Err(PoolError::ChannelClosed)))
            .collect()
    }
}
