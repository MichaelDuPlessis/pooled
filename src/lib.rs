//! A lightweight thread pool library with specialized pool types.
//!
//! # Example
//!
//! ```
//! use pooled::Runtime;
//!
//! let runtime = Runtime::new(4);
//!
//! // Fire-and-forget
//! let pool = runtime.simple_pool();
//! pool.submit(|| println!("hello from the pool"));
//!
//! // Map over inputs in parallel
//! let map = runtime.map_pool();
//! let results = map.map(vec![1, 2, 3], |x| x * 2);
//!
//! // Submit and retrieve a result later
//! let future = runtime.future_pool();
//! let task = future.submit(|| 42);
//! assert_eq!(task.wait().unwrap(), 42);
//!
//! runtime.shutdown();
//! ```

pub mod future;
pub mod map;
pub mod sequential;
pub mod simple;

use crate::sequential::SeqPool;
use crate::simple::SimplePool;
use crate::{future::FuturePool, map::MapPool};
use lockout::channel::mpmc;
use std::{
    any::Any,
    fmt,
    sync::mpsc::RecvError,
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Errors that can occur when executing jobs on the pool.
pub enum PoolError {
    /// A job panicked. Contains the panic payload.
    JobPanic(Box<dyn Any + Send>),
    /// The channel was closed unexpectedly.
    ChannelClosed,
}

impl fmt::Debug for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JobPanic(_) => write!(f, "PoolError::JobPanic(..)"),
            Self::ChannelClosed => write!(f, "PoolError::ChannelClosed"),
        }
    }
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JobPanic(_) => write!(f, "job panicked"),
            Self::ChannelClosed => write!(f, "channel closed"),
        }
    }
}

impl std::error::Error for PoolError {}

impl From<RecvError> for PoolError {
    fn from(_: RecvError) -> Self {
        Self::ChannelClosed
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

/// The main thread pool used to create other pools.
pub struct Runtime {
    workers: Vec<Worker>,
    sender: mpmc::Sender<Message>,
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: mpmc::Receiver<Message>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.recv();
                match message {
                    Ok(Message::NewJob(job)) => {
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job));
                    }
                    Ok(Message::Terminate) => break,
                    Err(_) => break,
                }
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}

impl Runtime {
    /// Create a new Runtime with a specified number of threads.
    ///
    /// # Panics
    ///
    /// Panics if `num_threads` is zero.
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0, "thread pool must have at least one thread");

        let (sender, receiver) = mpmc::channel();

        let workers = (0..num_threads)
            .map(|_| Worker::new(receiver.clone()))
            .collect();

        Self { workers, sender }
    }

    fn send(&self, message: Message) {
        // Safety: Runtime controls the receivers which can only be dropped if the runtime is dropped
        unsafe { self.sender.send(message).unwrap_unchecked() };
    }

    fn send_job<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let message = Message::NewJob(Box::new(job));
        self.send(message);
    }

    fn send_shutdown(&self) {
        self.send(Message::Terminate);
    }

    /// Create a [`SimplePool`] from this runtime.
    pub fn simple_pool(&self) -> SimplePool<'_> {
        SimplePool::new(self)
    }

    /// Create a [`MapPool`] from this runtime.
    pub fn map_pool(&self) -> MapPool<'_> {
        MapPool::new(self)
    }

    /// Create a [`SeqPool`] from this runtime.
    pub fn seq_pool(&self) -> SeqPool<'_> {
        SeqPool::new()
    }

    /// Create a [`FuturePool`] from this runtime.
    pub fn future_pool(&self) -> FuturePool<'_> {
        FuturePool::new(self)
    }

    /// Shutdown the runtime and wait for all threads to finish.
    pub fn shutdown(self) {
        drop(self)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.send_shutdown();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}
