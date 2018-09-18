//! Collection module for various parsers

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
