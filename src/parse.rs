//! Collection module for various parsers
use nom::{
    IResult,
    Err::Error,
    ErrorKind::Custom,
    alt,
    delimited,
    error_position,
    tag,
    take_till1,
    take_until,
    types::CompleteStr,
};
use failure::{
    Fail,
};

use std::fmt;

pub trait ParseInto<'a, T: Parse<'a>> {
    fn parse_into(&'a self) -> Result<T, ParseError<T::Error>>;
}

/// Trait implemented by types which can be parsed from an `&'a str`.
pub trait Parse<'a>: Sized {
    type Error: fmt::Display + fmt::Debug + Send + Sync + 'static;
    fn parse_from(input: &'a str) -> Result<Self, ParseError<Self::Error>>;
}

// TODO(eliza): add spans!
#[derive(Clone, Debug, Fail)]
pub enum ParseError<E: fmt::Display + fmt::Debug + Send + Sync + 'static> {
    #[fail(display = "more input required")]
    NoInput,
    // TODO(eliza): would it be better to represent parse results as a
    // Result<Option<T>,...> instead?
    #[fail(display = "not recognized")]
    Unrecognized,
    #[fail(display = "{}", 0)]
    Other(E),
    // TODO(eliza): more variants: unrecognized character, too much input,
    // etc...
}

impl<'a, T, P: 'a> ParseInto<'a, T> for P
where
    P: AsRef<str>,
    T: Parse<'a>,
{
    fn parse_into(&'a self) -> Result<T, ParseError<T::Error>> {
        T::parse_from(self.as_ref())
    }
}

impl<E: fmt::Display + fmt::Debug + Send + Sync + 'static> From<E> for ParseError<E> {
    fn from(err: E) -> ParseError<E> {
        ParseError::Other(err)
    }
}

/// Result type for working with `nom` parsers.
///
/// Parsers will always operate on fully-loaded strings, and so can wrap the
/// input in CompleteStr.
///
/// The `Ok` variant of this type is a tuple of two parts. The left
/// element is the remnant, and is always a CompleteStr. The right
/// element is the value, and is the supplied type.
crate type ParseResult<'a, T, E = u32> = IResult<CompleteStr<'a>, T, E>;

/// Pulls out one span of text.
///
/// This span may be:
/// - double-quote-delimited run
/// - single-quote-delimited run
/// - one bare word
crate fn span(text: CompleteStr) -> ParseResult<CompleteStr> {
    //  nom macros are leaky, and need updated for use under 2018 symbol rules.
    let text = CompleteStr::from(text.trim_left());
    alt!(text, dquote | squote | word)
}

/// Takes the first word.
///
/// A word is defined as any run of non-whitespace characters. This parser trims
/// all leading whitespace, and then takes text until it encounters a whitespace
/// character, with no further analysis.
crate fn word(text: CompleteStr) -> ParseResult<CompleteStr> {
    let text = CompleteStr::from(text.trim_left());
    take_till1!(text, char::is_whitespace)
}

/// Parses a single-quote-delimited run of text.
///
/// Single-quoted text has no escape analysis performed. Once an unescaped
/// single quote character (U+0027) is detected, the parser advances until it
/// detects another, and returns the intervening span (without quotes) as the
/// output, and all remaining text *after* the terminating single quote as the
/// remainder.
crate fn squote(text: CompleteStr) -> ParseResult<CompleteStr> {
    let text = CompleteStr::from(text.trim_left());
    delimited!(text, tag!("'"), take_until!("'"), tag!("'"))
}

/// Parses a double-quote-delimited run of text.
///
/// Double-quoted text has escape analyis performed. Once an unescaped double
/// quote character (U+0022) is detected, the parser advances until it detects
/// another **unescaped** double quote character, *also* skipping through
/// subshell strings.
//  Note: subshell strings are not implemented.
//  Note: because the syntax of double-quoted strings is more complex than the
//  nom component parsers can handle, this function is *significantly* more
//  complex than its single-quoted counterpart above.
crate fn dquote(text: CompleteStr) -> ParseResult<CompleteStr> {
    let text = CompleteStr::from(text.trim_left());

    //  If the text is empty, abort.
    if text.trim_right().is_empty() {
        // trace!("Text provided to dquote was empty");
        return Err(Error(error_position!(text, Custom('"' as u32))));
    }

    //  Start crawling the text.
    let mut iter = text.char_indices();

    //  If the first character is not a double quote, abort.
    if let Some((_, '"')) = iter.next() {}
    else {
        // trace!("Text provided to dquote did not begin with a double-quote");
        return Err(Error(error_position!(text, Custom('"' as u32))));
    }

    while let Some((i, c)) = iter.next() {
        //  A double-quote ends the span.
        if c == '"' {
            //  The `text` binding has the opening '"' as its 0th character, and
            //  `i` is the index of the terminating '"'. Therefore, the value
            //  output is after the opening quote and before the closing quote,
            //  and the remnant output is after the closing quote. The quotes
            //  themselves are absent from the output.
            return Ok((text[i + 1 ..].into(), text[1 .. i].into()));
        }
        //  A backslash unconditionally skips the next character.
        if c == '\\' {
            drop(iter.next());
            continue;
        }
        /* TODO(myrrlyn): Implement subshell grammar
        //  A dollar sign switches over to the subshell spanner.
        if c == '$' {
            //  The subshell scanner will collect its entire span, and return
            //  the rest of the string at this level in its remnant output.
            let (rest, _) = subshell(text[i ..])?;
            iter = rest.char_indices();
            continue;
        }
        */
    }

    //  If the loop terminates and reaches here, then no trailing quote was
    //  found.
    // error!("No terminating double quote was found!");
    Err(Error(error_position!(text, Custom('"' as u32))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_word() {
        let text = "hello world";
        let (rest, span) = word(text.into()).expect("source text is correct");
        assert_eq!(*span, "hello");
        assert_eq!(*rest, " world");
    }

    #[test]
    fn span_squote() {
        let text = r"'hello world' is one token";
        let (rest, span) = squote(text.into()).expect("source text is correct");
        assert_eq!(*span, "hello world");
        assert_eq!(*rest, " is one token");

        let text = r"'hello world is unterminated";
        assert!(squote(text.into()).is_err());
    }

    #[test]
    fn span_dquote() {
        let text = r#""hello world" is one token"#;
        let (rest, span) = dquote(text.into()).expect("source text is correct");
        assert_eq!(*span, "hello world");
        assert_eq!(*rest, " is one token");

        let text = r#""hello \\ \"world\"" may include backslashes"#;
        let (rest, span) = dquote(text.into()).expect("source text is correct");
        //  dquote does not process escapes.
        assert_eq!(*span, r#"hello \\ \"world\""#);
        assert_eq!(*rest, " may include backslashes");

        let text = r#""hello world is unterminated"#;
        assert!(dquote(text.into()).is_err());

        //  empty strings fail
        assert!(dquote("   ".into()).is_err());

        //  non-dquote strings fail
        assert!(dquote("hello".into()).is_err());
    }

    #[test]
    fn span_all() {
        let text = r#"now here is a "hard \"one\"" with 'many parts'"#;
        let mut cursor = text;
        let mut parts = Vec::new();
        while !cursor.trim().is_empty() {
            let (rest, span) = span(cursor.into()).expect("source text is correct");
            parts.push(*span);
            cursor = *rest;
        }
        assert_eq!(parts, &[
            "now",
            "here",
            "is",
            "a",
            //  dquote
            r#"hard \"one\""#,
            "with",
            //  squote
            "many parts",
        ]);
    }
}
