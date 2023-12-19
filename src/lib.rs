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

impl From<BodyChild> for &str {
    fn from(value: BodyChild) -> Self {
        match value {
            BodyChild::Text(v) => v,
        }
    }
}

#[derive(Debug)]
pub enum Body {
    Attribute(BodyAttribute),
    Child(BodyChild)
}

#[derive(Debug)]
pub enum BodyOutput {
    Internal(Cow<'static, str>),
    Inner(Body),
}

pub async gen fn body(inner: impl AsyncIterator<Item=Body>) -> BodyOutput {
    yield BodyOutput::Internal(r#"<body"#.into());
    let mut inner = pin!(inner);
    loop {
        match inner.next().await {
            Some(Body::Attribute(attr)) => {
                match attr {
                    
                }
            }
            Some(Body::Child(child)) => {
                yield BodyOutput::Internal(r#">"#.into());
                match child {
                    BodyChild::Text(text) => {
                        yield BodyOutput::Internal(encode_element_text(text));
                    },
                }
                break;
            }
            None => {
                BodyOutput::Internal(r#">"#.into());
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
                        yield BodyOutput::Internal(encode_element_text(text));
                    },
                }
            },
            Some(Body::Attribute(attr)) => {
                panic!("unexpected attribute {attr:?}, already started children")
            }
            None => break,
        }
    }
    yield BodyOutput::Internal(r#"</body>"#.into());
}

#[derive(Debug)]
pub enum HtmlAttribute {
    Lang(&'static str)
}

#[derive(Debug)]
pub enum HtmlChild {
    Text(&'static str),
    Body(BodyOutput),
}

#[derive(Debug)]
pub enum Html {
    Attribute(HtmlAttribute),
    Child(HtmlChild)
}

#[derive(Debug)]
pub enum HtmlOutput {
    Internal(Cow<'static, str>),
    Inner(Html),
    BodyOutput(BodyOutput),
}

pub async gen fn html(inner: impl AsyncIterator<Item=Html>) -> HtmlOutput {
    yield HtmlOutput::Internal(r#"<!DOCTYPE html>"#.into());
    yield HtmlOutput::Internal(r#"<html"#.into());
    let mut inner = pin!(inner);
    loop {
        match inner.next().await {
            Some(Html::Attribute(attr)) => {
                match attr {
                    HtmlAttribute::Lang(lang) => {
                        yield HtmlOutput::Internal(r#" lang=""#.into());
                        yield HtmlOutput::Internal(encode_double_quoted_attribute(lang));
                        yield HtmlOutput::Internal(r#"""#.into());
                    },
                }
            }
            Some(Html::Child(child)) => {
                yield HtmlOutput::Internal(r#">"#.into());
                match child {
                    HtmlChild::Text(text) => {
                        yield HtmlOutput::Internal(encode_element_text(text));
                    },
                    HtmlChild::Body(body) => match body {
                        BodyChild::Text(text) => {
                            yield HtmlOutput::BodyOutput(BodyOutput::Internal(encode_element_text(text)));
                        },
                    },
                }
                break;
            }
            None => {
                yield HtmlOutput::Internal(r#">"#.into());
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
                        yield HtmlOutput::Internal(encode_element_text(text));
                    },
                    HtmlChild::Body(body) => match body {
                        BodyChild::Text(text) => {
                            yield HtmlOutput::BodyOutput(BodyOutput::Internal(encode_element_text(text)));
                        },
                    },
                }
            },
            Some(Html::Attribute(attr)) => {
                panic!("unexpected attribute {attr:?}, already started children")
            }
            None => break,
        }
    }
    yield HtmlOutput::Internal(r#"</html>"#.into());
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
        let a = body(async gen {
            yield Body::Child(BodyChild::Text("test"));
        });
        while let Some(v) = a.next().await {
            yield Html::Child(HtmlChild::Body(v));
        }
    }));
    let mut result = String::new();

    while let Some(v) = async_iterator.next().await {
        result.push_str(v.into());
    }
    result
}

#[cfg(test)]
mod tests {
    use std::{task::{Waker, Context, Poll}, future::Future};

    use super::*;

    #[test]
    fn it_works() {
        let test: &str = BodyChild::Text("hi").into();

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
