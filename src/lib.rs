//! A WaitGroup waits for a collection of task to finish.
//!
//! ## Examples
//! ```rust
//! use waitgroup::WaitGroup;
//! use async_std::task;
//! # task::block_on(
//! async {
//!     let wg = WaitGroup::new();
//!     for _ in 0..100 {
//!         // 1. basic usage
//!         let w = wg.worker();
//!         task::spawn(async move {
//!             // do work...
//!             drop(w); // drop w means task finished, or just use `let _worker = w;`
//!         });
//!         // 2. waiting nested tasks using `Worker::clone`.
//!         let w = wg.worker();
//!         task::spawn(async move {
//!             let worker = w;
//!             // do work...
//!             let sub_task = worker.clone();
//!             task::spawn(async move {
//!                 let _sub_task = sub_task;
//!                 // do work...
//!             });
//!         });
//!         // 3. waiting blocking tasks
//!         let blocking_worker = wg.worker();
//!         std::thread::spawn(move || {
//!             let _blocking_worker = blocking_worker;
//!             // do blocking work...
//!         });
//!     }
//!
//!     wg.wait().await;
//! }
//! # );
//! ```

use atomic_waker::AtomicWaker;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};

pub struct WaitGroup {
    inner: Arc<Inner>,
}

#[derive(Clone)]
pub struct Worker(Arc<Inner>);

pub struct WaitGroupFuture {
    inner: Weak<Inner>,
}

impl WaitGroupFuture {
    /// Gets the number of active workers.
    pub fn workers(&self) -> usize {
        Weak::strong_count(&self.inner)
    }
}

struct Inner {
    waker: AtomicWaker,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.waker.wake();
    }
}

impl WaitGroup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                waker: AtomicWaker::new(),
            }),
        }
    }

    pub fn worker(&self) -> Worker {
        Worker(self.inner.clone())
    }

    /// Gets the number of active workers.
    pub fn workers(&self) -> usize {
        Arc::strong_count(&self.inner) - 1
    }

    pub fn wait(self) -> WaitGroupFuture {
        WaitGroupFuture {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl Default for WaitGroup {
    fn default() -> Self {
        Self::new()
    }
}

/*
IntoFuture tracking issue: https://github.com/rust-lang/rust/issues/67644
impl IntoFuture for WaitGroup {
    type Output = ();
    type Future = WaitGroupFuture;

    fn into_future(self) -> Self::Future {
        WaitGroupFuture { inner: Arc::downgrade(&self.inner) }
    }
}
*/

impl Future for WaitGroupFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.upgrade() {
            Some(inner) => {
                inner.waker.register(cx.waker());
                Poll::Pending
            }
            None => Poll::Ready(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use async_std::task;

    #[async_std::test]
    async fn smoke() {
        let wg = WaitGroup::new();

        for _ in 0..100 {
            let w = wg.worker();
            task::spawn(async move {
                drop(w);
            });
        }

        wg.wait().await;
    }
}
