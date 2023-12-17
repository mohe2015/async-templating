use std::{async_iter::AsyncIterator, future::Future, pin::Pin, task::{Context, Poll}};

pub trait AsyncIterExt {
    fn next(&mut self) -> Next<'_, Self>;
}

impl<T> AsyncIterExt for T {
    fn next(&mut self) -> Next<'_, Self> {
        Next { s: self }
    }
}

pub struct Next<'s, S: ?Sized> {
    s: &'s mut S,
}

impl<'s, S: AsyncIterator> Future for Next<'s, S> where S: Unpin {
    type Output = Option<S::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<S::Item>> {
        Pin::new(&mut *self.s).poll_next(cx)
    }
}
