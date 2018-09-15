use std::{ffi::OsStr, fmt, str};

use crate::parse::{Parse, ParseError};

/// Invocation of an executable command.
///
/// This is called `Invoke` because the name `Cmd` was already taken.
#[derive(Clone, Debug)]
pub struct Invoke<'a> {
    /// The command to invoke.
    pub command: &'a OsStr,
    /// Zero or more arguments to pass to the command.
    pub args: super::ArgsIter<'a>,
}

// ===== impl Invoke =====

impl<'a> Parse<'a> for Invoke<'a> {
    type Error = String; // this string is never used, it's a placeholder.
    fn parse_from(text: &'a str) -> Result<Self, ParseError<Self::Error>> {
        let mut args = super::ArgsIter { text };
        let command = args.next().map(OsStr::new).ok_or(ParseError::NoInput)?;
        Ok(Invoke { command, args })
    }
}

impl<'a> fmt::Display for Invoke<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.command.to_string_lossy())?;
        for arg in self.args.clone() {
            write!(f, " {}", arg)?;
        }
        Ok(())
    }
}
