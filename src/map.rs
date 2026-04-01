use std::sync::{Arc, mpsc};

use crate::{Runtime, simple::SimplePool};

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
    pub fn map<T, R, F>(&self, inputs: Vec<T>, f: F) -> Vec<R>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let len = inputs.len();

        let f = Arc::new(f);
        for (i, input) in inputs.into_iter().enumerate() {
            let f = Arc::clone(&f);

            let tx = tx.clone();
            let job = Box::new(move || {
                let result = f(input);
                tx.send((i, result)).unwrap();
            });

            self.pool.submit(job);
        }

        let mut results = Vec::with_capacity(len);
        unsafe { results.set_len(len) };

        for _ in 0..len {
            let (i, result) = rx.recv().unwrap();
            results[i] = Some(result);
        }

        results.into_iter().map(|r| r.unwrap()).collect()
    }
}
