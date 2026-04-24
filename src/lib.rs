pub mod map;
pub mod sequential;
pub mod simple;

use crate::map::MapPool;
use crate::sequential::SeqPool;
use crate::simple::SimplePool;
use lockout::channel::mpmc;
use std::{
    any::Any,
    sync::mpsc::RecvError,
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

// TODO: Libraries name is probably going to change so this may need to also
pub enum PoolError {
    JobPanic(Box<dyn Any + Send>),
    ChannelClosed,
}

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
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver) = mpmc::channel();

        let workers = (0..num_threads)
            .map(|_| Worker::new(receiver.clone()))
            .collect();

        Self { workers, sender }
    }

    fn send(&self, message: Message) {
        self.sender.send(message).unwrap();
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

    /// Create a `SimplePool` from this runtime.
    pub fn simple_pool(&self) -> SimplePool<'_> {
        SimplePool::new(&self)
    }

    /// Create a `MapPool` from this runtime.
    pub fn map_pool(&self) -> MapPool<'_> {
        MapPool::new(&self)
    }

    /// Create a `SeqPool` from this runtime.
    pub fn seq_pool(&self) -> SeqPool {
        SeqPool::new()
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
