# waitgroup

A WaitGroup waits for a collection of task to finish. 

## Examples

```rust 
use waitgroup::WaitGroup;
use async_std::task;
async {
    let wg = WaitGroup::new();
    for _ in 0..100 {
        // 1. basic usage
        let w = wg.worker();
        task::spawn(async move {
            // do work...
            drop(w); // drop w means task finished, or just use `let _worker = w;`
        });
        // 2. waiting nested tasks using `Worker::clone`.
        let w = wg.worker();
        task::spawn(async move {
            let worker = w;
            // do work...
            let sub_task = worker.clone();
            task::spawn(async move {
                let _sub_task = sub_task;
                // do work...
            });
        });
        // 3. waiting blocking tasks
        let blocking_worker = wg.worker();
        std::thread::spawn(move || {
            let _blocking_worker = blocking_worker;
            // do blocking work...
        });
    }

    wg.wait().await;
}
```


# License

This project is licensed under Apache License, Version 2.0 ([LICENSE](LICENSE) ).

