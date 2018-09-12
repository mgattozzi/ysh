use std::str;

use crate::parse::{Parse, ParseError};
pub mod builtin;
mod invoke;

pub use self::builtin::Builtin;
pub use self::invoke::Invoke;

#[derive(Clone, Debug)]
pub enum Cmd<'a> {
    Builtin(builtin::Builtin<'a>),
    Invoke(Invoke<'a>),
}

/// Evaluate a command in an environment.
///
/// This represents forms such as
/// ```notrust
/// FOO=foo BAR=bar command
/// ```
/// and
/// ```notrust
/// env FOO=foo BAR=bar command
/// ```
#[derive(Clone, Debug)]
pub struct WithEnv<'a> {
    cmd: Cmd<'a>,
    // TODO(eliza): what kind of fucked up iterator is the env going to be?
}

impl<'a> Parse<'a> for Cmd<'a> {
    type Error = String; // placeholder
    fn parse_from(s: &'a str) -> Result<Self, ParseError<Self::Error>> {
        Builtin::parse_from(s)
            .map(Cmd::Builtin)
            .or_else(|_| Invoke::parse_from(s).map(Cmd::Invoke))
    }
}

impl<'a> Parse<'a> for WithEnv<'a> {
    type Error = String; // placeholder
    fn parse_from(s: &'a str) -> Result<Self, ParseError<Self::Error>> {
        // TODO(eliza): alex pls implement me
        unimplemented!()
    }
}
