use pooled::{PoolError, Runtime};

#[test]
fn map_returns_results_in_order() {
    let runtime = Runtime::new(4);
    let pool = runtime.map_pool();

    let results = pool.map(vec![1, 2, 3, 4, 5], |x| x * 2);
    let values: Vec<_> = results.into_iter().map(|r| r.unwrap()).collect();

    assert_eq!(values, vec![2, 4, 6, 8, 10]);

    runtime.shutdown();
}

#[test]
fn map_empty_input() {
    let runtime = Runtime::new(2);
    let pool = runtime.map_pool();

    let results = pool.map(Vec::<i32>::new(), |x| x * 2);
    assert!(results.is_empty());

    runtime.shutdown();
}

#[test]
fn map_captures_panics() {
    let runtime = Runtime::new(4);
    let pool = runtime.map_pool();

    let results = pool.map(vec![1, 2, 3], |x| {
        if x == 2 { panic!("bad input"); }
        x * 10
    });

    assert_eq!(results[0].as_ref().unwrap(), &10);
    assert!(matches!(results[1], Err(PoolError::JobPanic(_))));
    assert_eq!(results[2].as_ref().unwrap(), &30);

    runtime.shutdown();
}

#[test]
fn map_large_input() {
    let runtime = Runtime::new(4);
    let pool = runtime.map_pool();

    let inputs: Vec<_> = (0..1000).collect();
    let results = pool.map(inputs, |x| x * 2);
    let values: Vec<_> = results.into_iter().map(|r| r.unwrap()).collect();

    let expected: Vec<_> = (0..1000).map(|x| x * 2).collect();
    assert_eq!(values, expected);

    runtime.shutdown();
}
