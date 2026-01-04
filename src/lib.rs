use crate::map::MapPool;
use crate::simple::SimplePool;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};

pub mod map;
pub mod simple;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

/// The main thread pool used to create other pools.
pub struct Runtime {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(Message::NewJob(job)) => job(),
                    Ok(Message::Terminate) => break,
                    Err(_) => break,
                }
            }
        });

        Worker { thread }
    }
}

impl Runtime {
    /// Create a new Runtime with a specified number of threads.
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..num_threads)
            .map(|_| Worker::new(Arc::clone(&receiver)))
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

    /// Create a SimplePool from this runtime.
    pub fn simple_pool(&self) -> SimplePool<'_> {
        SimplePool::new(&self)
    }

    /// Create a MapPool from this runtime.
    pub fn map_pool(&self) -> MapPool<'_> {
        MapPool::new(&self)
    }

    /// Shutdown the runtime and wait for all threads to finish.
    pub fn shutdown(self) {
        for _ in &self.workers {
            self.send_shutdown();
        }

        for worker in self.workers {
            worker.thread.join().unwrap();
        }
    }
}
