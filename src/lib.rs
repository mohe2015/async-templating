#![feature(async_iterator, gen_blocks, noop_waker)]

pub mod async_iterator_extension;

extern crate alloc;

use std::{async_iter::AsyncIterator, pin::pin};

use alloc::borrow::Cow;

use crate::async_iterator_extension::AsyncIterExt;

pub async gen fn html(inner: impl AsyncIterator<Item=Cow<'static, str>>) -> Cow<'static, str> {
    yield r#"<!DOCTYPE html>"#.into();
    yield r#"<html lang="en">"#.into();
    let mut inner = pin!(inner);
    while let Some(v) = inner.next().await {
        yield v;
    }
    yield r#"</html>"#.into();
}

pub async fn html_main() -> String {
    let mut async_iterator = pin!(html(async gen { yield "test".into() }));
    let mut result = String::new();

    while let Some(v) = async_iterator.next().await {
        result.push_str(&v);
    }
    result
}

#[cfg(test)]
mod tests {
    use std::{task::{Waker, Context, Poll}, future::Future};

    use super::*;

    #[test]
    fn it_works() {
        let mut fut = pin!(html_main());

        // Poll loop, just to test the future...
        let waker = Waker::noop();
        let ctx = &mut Context::from_waker(&waker);

        loop {
            match fut.as_mut().poll(ctx) {
                Poll::Pending => {}
                Poll::Ready(result) => {
                    assert_eq!("test", result);
                    break
                },
            }
        }
    }
}
