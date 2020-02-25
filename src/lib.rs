//! A WaitGroup waits for a collection of task to finish.
//!
//! ## Examples
//! ```rust
//! use waitgroup::WaitGroup;
//! use async_std::task;
//! async {
//!     let wg = WaitGroup::new();
//!     for _ in 0..100 {
//!         let w = wg.worker();
//!         task::spawn(async move {
//!             // do work
//!             drop(w); // drop d means task finished
//!         };
//!     }
//!
//!     wg.wait().await;
//! }
//! ```
//!  
use std::cell::UnsafeCell;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll, Waker};

pub struct WaitGroup {
    inner: Arc<Inner>,
}

#[derive(Clone)]
pub struct Worker {
    inner: Arc<Inner>,
}

// Safety: the Inner field can not be accessed, except dropping.
unsafe impl Sync for Worker {}
unsafe impl Send for Worker {}

pub struct WaitGroupFuture {
    inner: Weak<Inner>,
}

struct Inner {
    waker: UnsafeCell<Option<Waker>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        let cell = mem::replace(&mut self.waker, UnsafeCell::new(None));
        if let Some(waker) = cell.into_inner() {
            waker.wake();
        }
    }
}

impl WaitGroup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                waker: UnsafeCell::new(None),
            }),
        }
    }

    pub fn worker(&self) -> Worker {
        Worker {
            inner: self.inner.clone(),
        }
    }

    pub fn wait(self) -> WaitGroupFuture {
        WaitGroupFuture {
            inner: Arc::downgrade(&self.inner),
        }
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
                // Safety: since we have a Arc instance now, so Inner::drop can not be called
                // concurrently and this mutable access is unique.
                let waker = unsafe { &mut *inner.waker.get() };
                *waker = Some(cx.waker().clone());
                Poll::Pending
            }
            None => return Poll::Ready(()),
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
