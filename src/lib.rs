#![feature(async_closure, async_iterator, coroutines, gen_blocks, noop_waker)]

pub mod async_iterator_extension;
pub mod encode;

extern crate alloc;

use std::{async_iter::AsyncIterator, pin::pin};

use alloc::borrow::Cow;
use encode::{encode_element_text, encode_double_quoted_attribute};

use crate::async_iterator_extension::AsyncIterExt;

#[derive(Debug)]
pub enum HtmlAttribute {
    Lang(&'static str)
}

#[derive(Debug)]
pub enum HtmlChild {
    Text(&'static str)
}

#[derive(Debug)]
pub enum Html {
    Attribute(HtmlAttribute),
    Child(HtmlChild)
}

pub async gen fn html(inner: impl AsyncIterator<Item=Html>) -> Cow<'static, str> {
    yield r#"<!DOCTYPE html>"#.into();
    yield r#"<html"#.into();
    let mut inner = pin!(inner);
    loop {
        match inner.next().await {
            Some(Html::Attribute(attr)) => {
                match attr {
                    HtmlAttribute::Lang(lang) => {
                        yield r#" lang=""#.into();
                        yield encode_double_quoted_attribute(lang);
                        yield r#"""#.into();
                    },
                }
            }
            Some(Html::Child(child)) => {
                yield r#">"#.into();
                match child {
                    HtmlChild::Text(text) => {
                        yield encode_element_text(text);
                    },
                }
                break;
            }
            None => {
                yield r#">"#.into();
                break;
            }
        }
    }
    loop {
        match inner.next().await {
            Some(Html::Child(child)) => {
                // TODO FIXME code duplication
                match child {
                    HtmlChild::Text(text) => {
                        yield encode_element_text(text);
                    },
                }
            },
            Some(Html::Attribute(attr)) => {
                panic!("unexpected attribute {attr:?}, already started children")
            }
            None => break,
        }
    }
    yield r#"</html>"#.into();
}

pub async fn html_main() -> String {
    // `async` coroutines are not yet supported
    //let mut coroutine = async || {
    //    yield 1;
    //    "foo"
    //};

    let mut async_iterator = pin!(html(async gen {
        yield Html::Attribute(HtmlAttribute::Lang("en"));
        yield Html::Child(HtmlChild::Text("test"));
    }));
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

        let waker = Waker::noop();
        let ctx = &mut Context::from_waker(&waker);

        loop {
            match fut.as_mut().poll(ctx) {
                Poll::Pending => {}
                Poll::Ready(result) => {
                    assert_eq!("<!DOCTYPE html><html lang=\"en\">test</html>", result);
                    break
                },
            }
        }
    }
}
