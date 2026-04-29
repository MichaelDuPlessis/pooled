use pooled::Runtime;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

#[test]
fn submit_executes_job() {
    let runtime = Runtime::new(2);
    let pool = runtime.simple_pool();

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    pool.submit(move || { c.fetch_add(1, Ordering::SeqCst); });

    runtime.shutdown();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn submit_multiple_jobs() {
    let runtime = Runtime::new(4);
    let pool = runtime.simple_pool();

    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..100 {
        let c = counter.clone();
        pool.submit(move || { c.fetch_add(1, Ordering::SeqCst); });
    }

    runtime.shutdown();
    assert_eq!(counter.load(Ordering::SeqCst), 100);
}

#[test]
fn survives_panicking_job() {
    let runtime = Runtime::new(2);
    let pool = runtime.simple_pool();

    pool.submit(|| panic!("oops"));

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();
    pool.submit(move || { c.fetch_add(1, Ordering::SeqCst); });

    runtime.shutdown();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}
