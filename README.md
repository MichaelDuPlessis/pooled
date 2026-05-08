# Pooled

A lightweight thread pool library with specialized pool types.

## Pool Types

- **SimplePool** — fire-and-forget job submission
- **MapPool** — map a function over inputs in parallel, results returned in order
- **FuturePool** — submit a job and retrieve the result later via a `Task` handle
- **SeqPool** — no parallelism, runs jobs synchronously (useful for testing/debugging)

## Usage

```rust
use pooled::Runtime;

let runtime = Runtime::new(4);

// Fire-and-forget
let pool = runtime.simple_pool();
pool.submit(|| println!("hello from the pool"));

// Map over inputs in parallel
let map = runtime.map_pool();
let results = map.map(vec![1, 2, 3], |x| x * 2);

// Submit and retrieve a result later
let future = runtime.future_pool();
let task = future.submit(|| 42);
assert_eq!(task.wait().unwrap(), 42);

runtime.shutdown();
```

## Error Handling

Jobs that panic are caught and returned as `Err(PoolError::JobPanic(..))` in `MapPool` and `FuturePool`. Worker threads stay alive after a panic.

## License

MIT
