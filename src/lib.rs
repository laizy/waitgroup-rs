// AtomicWaker is exposed in futures_utils, here use the internal one from futures_core to avoid
// introducing a lot of dependencies.
use futures_core::task::__internal::AtomicWaker;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};

pub struct WaitGroup {
    inner: Weak<Inner>,
}

#[derive(Clone)]
pub struct Done {
    inner: Arc<Inner>,
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
    pub fn new() -> (Self, Done) {
        let inner = Arc::new(Inner {
            waker: AtomicWaker::new(),
        });
        let wg = WaitGroup {
            inner: Arc::downgrade(&inner),
        };
        let done = Done { inner };

        (wg, done)
    }
}

impl Future for WaitGroup {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.upgrade() {
            Some(inner) => {
                inner.waker.register(cx.waker());
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
        let (wg, done) = WaitGroup::new();

        for _ in 0..100 {
            let d = done.clone();
            task::spawn(async move {
                drop(d);
            });
        }

        drop(done);
        wg.await;
    }
}
