use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};

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
        SimplePool { runtime: &self }
    }

    /// Create a MapPool from this runtime.
    pub fn map_pool(&self) -> MapPool<'_> {
        MapPool { runtime: &self }
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

/// A simple fire-and-forget thread pool.
pub struct SimplePool<'a> {
    runtime: &'a Runtime,
}

impl<'a> SimplePool<'a> {
    /// Submit a job to be executed.
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.runtime.send_job(f);
    }
}

/// A thread pool for mapping functions over input lists.
pub struct MapPool<'a> {
    runtime: &'a Runtime,
}

impl<'a> MapPool<'a> {
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

            self.runtime.send_job(job);
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
