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

#[derive(Debug)]
pub enum BodyChild {
    Text(&'static str),
    Html(HtmlOutput),
}

#[derive(Debug)]
pub enum Body {
    Attribute(BodyAttribute),
    Child(BodyChild)
}

pub async gen fn body(inner: impl AsyncIterator<Item=Body>) -> Cow<'static, str> {
    yield r#"<body"#.into();
    let mut inner = pin!(inner);
    loop {
        match inner.next().await {
            Some(Body::Attribute(attr)) => {
                match attr {
                    
                }
            }
            Some(Body::Child(child)) => {
                yield r#">"#.into();
                match child {
                    BodyChild::Text(text) => {
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
            Some(Body::Child(child)) => {
                // TODO FIXME code duplication
                match child {
                    BodyChild::Text(text) => {
                        yield encode_element_text(text);
                    },
                }
            },
            Some(Body::Attribute(attr)) => {
                panic!("unexpected attribute {attr:?}, already started children")
            }
            None => break,
        }
    }
    yield r#"</body>"#.into();
}

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
        let a = body(async gen {
            yield Body::Child(BodyChild::Text("test"));
        });
        while let Some(v) = a.next().await {
            yield Body::Child(BodyChild::Html(v));
        }
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
