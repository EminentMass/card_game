use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake},
    thread::{self, Thread},
};

struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

pub trait BlockOn {
    type Output;
    fn block_on(self) -> Self::Output;
}

impl<T, O> BlockOn for T
where
    T: Future<Output = O>,
{
    type Output = O;
    fn block_on(self) -> Self::Output {
        let mut fut = Box::pin(self);
        let waker = Arc::new(ThreadWaker(thread::current())).into();
        let mut ctx = Context::from_waker(&waker);

        loop {
            match fut.as_mut().poll(&mut ctx) {
                Poll::Ready(res) => return res,
                Poll::Pending => thread::park(),
            }
        }
    }
}
