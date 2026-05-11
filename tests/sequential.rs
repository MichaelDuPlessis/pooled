use pooled::Runtime;

#[test]
fn submit_runs_synchronously() {
    let runtime = Runtime::new(2);
    let pool = runtime.seq_pool();

    let mut value = 0;
    pool.submit(|| {
        value = 42;
    });
    assert_eq!(value, 42);

    runtime.shutdown();
}

#[test]
fn submit_can_borrow_local_data() {
    let runtime = Runtime::new(2);
    let pool = runtime.seq_pool();

    let data = vec![1, 2, 3];
    let mut sum = 0;
    pool.submit(|| {
        sum = data.iter().sum();
    });
    assert_eq!(sum, 6);

    runtime.shutdown();
}
