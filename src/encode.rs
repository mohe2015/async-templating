use std::{borrow::Cow, sync::OnceLock};

use regex::{Regex, Captures};

pub fn encode_element_text<'a, I: Into<Cow<'a, str>>>(input: I) -> Cow<'a, str> {
    // https://html.spec.whatwg.org/dev/syntax.html
    // https://www.php.net/manual/en/function.htmlspecialchars.php
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = REGEX.get_or_init(|| Regex::new("[&<]").unwrap());

    let input = input.into();
    match regex.replace_all(&input, |captures: &Captures| {
        match captures.get(0).unwrap().as_str() {
            "&" => "&amp;",
            "<" => "&lt;",
            _ => unreachable!(),
        }
    }) {
        Cow::Borrowed(_) => input,
        Cow::Owned(owned) => Cow::Owned(owned),
    }
}

pub fn encode_double_quoted_attribute<'a, I: Into<Cow<'a, str>>>(input: I) -> Cow<'a, str> {
    // https://html.spec.whatwg.org/dev/dom.html#content-models
    // https://html.spec.whatwg.org/dev/syntax.html
    // https://html.spec.whatwg.org/#escapingString
    // https://html.spec.whatwg.org/
    // In the HTML syntax, authors need only remember to use U+0022 QUOTATION MARK
    // characters (") to wrap the attribute contents and then to escape all U+0026
    // AMPERSAND (&) and U+0022 QUOTATION MARK (") characters, and to specify the
    // sandbox attribute, to ensure safe embedding of content. (And remember to
    // escape ampersands before quotation marks, to ensure quotation marks become
    // &quot; and not &amp;quot;.)
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = REGEX.get_or_init(|| Regex::new("[&\"]").unwrap());

    let input = input.into();
    match regex.replace_all(&input, |captures: &Captures| {
        match captures.get(0).unwrap().as_str() {
            "&" => "&amp;",
            "\"" => "&quot;",
            _ => unreachable!(),
        }
    }) {
        Cow::Borrowed(_) => input,
        Cow::Owned(owned) => Cow::Owned(owned),
    }
}
