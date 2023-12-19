#![feature(async_closure, async_iterator, coroutines, gen_blocks, noop_waker)]

pub mod async_iterator_extension;
pub mod encode;

extern crate alloc;

use std::{async_iter::AsyncIterator, pin::pin};

use alloc::borrow::Cow;
use encode::{encode_element_text, encode_double_quoted_attribute};

use crate::async_iterator_extension::AsyncIterExt;

#[derive(Debug)]
pub enum BodyAttribute {
}

#[derive(Debug, derive_more::From)]
pub enum BodyChild {
    Text(&'static str),
}

#[derive(Debug)]
pub enum Body {
    Attribute(BodyAttribute),
    Child(BodyChild)
}

/// Values in *SafeOutput must already be correctly escaped.
#[derive(Debug)]
pub struct BodySafeOutput(Cow<'static, str>);

pub async gen fn body(inner: impl AsyncIterator<Item=Body>) -> BodySafeOutput {
    yield BodySafeOutput(r#"<body"#.into());
    let mut inner = pin!(inner);
    loop {
        match inner.next().await {
            Some(Body::Attribute(attr)) => {
                match attr {
                    
                }
            }
            Some(Body::Child(child)) => {
                yield BodySafeOutput(r#">"#.into());
                match child {
                    BodyChild::Text(text) => {
                        yield BodySafeOutput(encode_element_text(text));
                    },
                }
                break;
            }
            None => {
                BodySafeOutput(r#">"#.into());
                break;
            }
        }
    }
    loop {
        match inner.next().await {
            Some(Body::Child(child)) => {
                // TODO FIXME code duplication
                match child {
                    BodyChild::Text(text) => {
                        yield BodySafeOutput(encode_element_text(text));
                    },
                }
            },
            Some(Body::Attribute(attr)) => {
                panic!("unexpected attribute {attr:?}, already started children")
            }
            None => break,
        }
    }
    yield BodySafeOutput(r#"</body>"#.into());
}

#[derive(Debug)]
pub enum HtmlAttribute {
    Lang(&'static str)
}

#[derive(Debug)]
pub enum HtmlChild {
    Text(&'static str),
    Body(BodySafeOutput),
}

#[derive(Debug)]
pub enum Html {
    Attribute(HtmlAttribute),
    Child(HtmlChild)
}

/// Values in *SafeOutput must already be correctly escaped.
#[derive(Debug)]
pub struct HtmlSafeOutput(Cow<'static, str>);

pub async gen fn html(inner: impl AsyncIterator<Item=Html>) -> HtmlSafeOutput {
    yield HtmlSafeOutput(r#"<!DOCTYPE html>"#.into());
    yield HtmlSafeOutput(r#"<html"#.into());
    let mut inner = pin!(inner);
    loop {
        match inner.next().await {
            Some(Html::Attribute(attr)) => {
                match attr {
                    HtmlAttribute::Lang(lang) => {
                        yield HtmlSafeOutput(r#" lang=""#.into());
                        yield HtmlSafeOutput(encode_double_quoted_attribute(lang));
                        yield HtmlSafeOutput(r#"""#.into());
                    },
                }
            }
            Some(Html::Child(child)) => {
                yield HtmlSafeOutput(r#">"#.into());
                match child {
                    HtmlChild::Text(text) => {
                        yield HtmlSafeOutput(encode_element_text(text));
                    },
                    HtmlChild::Body(body) => yield HtmlSafeOutput(body.0),
                }
                break;
            }
            None => {
                yield HtmlSafeOutput(r#">"#.into());
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
                        yield HtmlSafeOutput(encode_element_text(text));
                    },
                    HtmlChild::Body(body) => yield HtmlSafeOutput(body.0),
                }
            },
            Some(Html::Attribute(attr)) => {
                panic!("unexpected attribute {attr:?}, already started children")
            }
            None => break,
        }
    }
    yield HtmlSafeOutput(r#"</html>"#.into());
}

async fn get_user_language() -> &'static str {
    r#"en&""#
}

async fn get_username() -> &'static str {
    "test&<"
}

pub async fn html_main() -> String {
    // `async` coroutines are not yet supported
    //let mut coroutine = async || {
    //    yield 1;
    //    "foo"
    //};

    let mut async_iterator = pin!(html(async gen {
        yield Html::Attribute(HtmlAttribute::Lang(get_user_language().await));
        yield Html::Child(HtmlChild::Text("test&<"));
        let mut body = pin!(body(async gen {
            yield Body::Child(BodyChild::Text(get_username().await));
        }));
        while let Some(v) = body.next().await {
            yield Html::Child(HtmlChild::Body(v));
        }
    }));
    let mut result = String::new();

    while let Some(v) = async_iterator.next().await {
        result.push_str(&v.0);
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
                    assert_eq!("<!DOCTYPE html><html lang=\"en&amp;&quot;\">test&amp;&lt;<body>test&amp;&lt;</body></html>", result);
                    break
                },
            }
        }
    }
}
