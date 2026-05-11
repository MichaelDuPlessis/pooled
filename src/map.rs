use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{Message, PoolError, Runtime};

/// A thread pool for mapping functions over input lists.
pub struct MapPool<'a> {
    runtime: &'a Runtime,
}

impl<'a> MapPool<'a> {
    /// Creates a new `MapPool` from a `Runtime`.
    pub(crate) fn new(runtime: &'a Runtime) -> Self {
        Self { runtime }
    }

    /// Map a function over a list of inputs, returning results in order.
    ///
    /// The closure may borrow non-`'static` data from the calling scope.
    /// This method blocks until all work is complete.
    pub fn map<T, R, F>(&self, inputs: &[T], f: F) -> Vec<Result<R, PoolError>>
    where
        T: Sync,
        R: Send,
        F: Fn(&T) -> R + Sync,
    {
        let len = inputs.len();
        if len == 0 {
            return Vec::new();
        }

        let num_chunks = self.runtime.workers.len().min(len);
        let mut buf: Vec<MaybeUninit<Result<R, PoolError>>> = Vec::with_capacity(len);
        unsafe { buf.set_len(len) };

        let buf_addr = buf.as_mut_ptr() as usize;
        let remaining = AtomicUsize::new(num_chunks);

        let f_ref: &F = &f;
        let inputs_ref: &[T] = inputs;
        let remaining_ref: &AtomicUsize = &remaining;

        let chunk_size = len / num_chunks;
        let extra = len % num_chunks;
        let mut offset = 0;

        for i in 0..num_chunks {
            let this_chunk = chunk_size + if i < extra { 1 } else { 0 };
            let chunk_offset = offset;
            offset += this_chunk;

            // Safety: we transmute to 'static. This function blocks below until all
            // jobs finish, guaranteeing all borrowed data outlives the jobs.
            let job: Box<dyn FnOnce() + Send + 'static> = unsafe {
                std::mem::transmute::<
                    Box<dyn FnOnce() + Send + '_>,
                    Box<dyn FnOnce() + Send + 'static>,
                >(Box::new(move || {
                    let out = buf_addr as *mut MaybeUninit<Result<R, PoolError>>;
                    for j in 0..this_chunk {
                        let idx = chunk_offset + j;
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            f_ref(&inputs_ref[idx])
                        }));
                        let val = match result {
                            Ok(v) => Ok(v),
                            Err(panic) => Err(PoolError::JobPanic(panic)),
                        };
                        out.add(idx).write(MaybeUninit::new(val));
                    }
                    remaining_ref.fetch_sub(1, Ordering::Release);
                }))
            };

            self.runtime.send(Message::NewJob(job));
        }

        while remaining.load(Ordering::Acquire) != 0 {
            if !self.runtime.steal() {
                std::thread::yield_now();
            }
        }

        // Safety: all slots initialized by workers above
        unsafe {
            let ptr = buf.as_ptr() as *mut Result<R, PoolError>;
            let cap = buf.capacity();
            std::mem::forget(buf);
            Vec::from_raw_parts(ptr, len, cap)
        }
    }
}
