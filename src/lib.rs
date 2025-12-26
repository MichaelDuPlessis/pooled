use std::thread::{self, JoinHandle};

/// The main thread pool used to create other pools.
pub struct CorePool {
    // the threads
    threads: Vec<JoinHandle<()>>,
}

impl CorePool {
    /// Create a new CorePool with a specified number of threads.
    pub fn new(num_threads: usize) -> Self {
        // creating threads
        let threads = (0..num_threads).map(|_| thread::spawn(|| {})).collect();

        Self { threads }
    }
}
