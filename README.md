# waitgroup

A WaitGroup waits for a collection of task to finish. 

## Examples

```rust
use waitgroup::WaitGroup;
use async_std::task;
async {
	let (wg, done) = WaitGroup::new();
	for _ in 0..100 {
		let d = done.clone();
		task::spawn(async move {
			// do work
			drop(d); // drop d means task finished
		};
	}
	drop(done);

	wg.await;
}
```
# License

This project is licensed under Apache License, Version 2.0 ([LICENSE](LICENSE) ).

