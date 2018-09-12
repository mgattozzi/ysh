use std::{ffi::OsStr, fmt, iter, str};

use crate::parse::{self,  Parse, ParseError};

/// Invocation of an executable command.
///
/// This is called `Invoke` because the name `Cmd` was already taken.
#[derive(Clone, Debug)]
pub struct Invoke<'a> {
    /// The command to invoke.
    pub command: &'a OsStr,
    /// Zero or more arguments to pass to the command.
    pub args: ArgsIter<'a>,
}

// ===== impl Invoke =====

impl<'a> Parse<'a> for Invoke<'a> {
    type Error = String; // this string is never used, it's a placeholder.
    fn parse_from(text: &'a str) -> Result<Self, ParseError<Self::Error>> {
        let mut args = ArgsIter { text, };
        let command = args.next()
            .map(OsStr::new)
            .ok_or(ParseError::NoInput)?;
        Ok(Invoke {
            command,
            args,
        })

    }
}

#[derive(Debug, Clone)]
pub struct ArgsIter<'a> {
    text: &'a str
}

impl<'a> iter::Iterator for ArgsIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.text.len() <= 0 {
            return None;
        }
        //  Use the span tokenizer to get a snippet
        let (rest, span) = parse::span(self.text.into())
            //  Suppress the errors for now. May be worth
            //  investigating so that the shell can repont
            //  invalid syntax?
            .ok()?;
        self.text = *rest;
        //  rest and span are CompleteStr, which implements
        //  Deref down to &str.
        Some(*span)
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
