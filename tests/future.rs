use pooled::{PoolError, Runtime};

#[test]
fn submit_returns_result() {
    let runtime = Runtime::new(2);
    let pool = runtime.future_pool();

    let task = pool.submit(|| 42);
    assert_eq!(task.wait().unwrap(), 42);

    runtime.shutdown();
}

#[test]
fn submit_multiple_tasks() {
    let runtime = Runtime::new(4);
    let pool = runtime.future_pool();

    let tasks: Vec<_> = (0..10).map(|i| pool.submit(move || i * 2)).collect();
    let results: Vec<_> = tasks.into_iter().map(|t| t.wait().unwrap()).collect();

    assert_eq!(results, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);

    runtime.shutdown();
}

#[test]
fn submit_panicking_job_returns_error() {
    let runtime = Runtime::new(2);
    let pool = runtime.future_pool();

    let task = pool.submit(|| -> i32 { panic!("oops") });
    let result = task.wait();

    assert!(matches!(result, Err(PoolError::JobPanic(_))));

    runtime.shutdown();
}

#[test]
fn pool_still_works_after_panic() {
    let runtime = Runtime::new(2);
    let pool = runtime.future_pool();

    let _panic_task = pool.submit(|| -> i32 { panic!("oops") });
    let _ = _panic_task.wait();

    let task = pool.submit(|| 123);
    assert_eq!(task.wait().unwrap(), 123);

    runtime.shutdown();
}

#[test]
fn submit_with_moved_data() {
    let runtime = Runtime::new(2);
    let pool = runtime.future_pool();

    let data = vec![1, 2, 3, 4, 5];
    let task = pool.submit(move || data.iter().sum::<i32>());

    assert_eq!(task.wait().unwrap(), 15);

    runtime.shutdown();
}
