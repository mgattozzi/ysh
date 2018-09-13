use std::{path::Path, str, rc::Rc};
use failure::{
    Fail,
};
use crate::{
    ast,
    parse::{Parse, ParseError},
};

/// Represents all shell builtins.
#[derive(Clone, Debug)]
pub enum Builtin<'a> {
    Clear,
    Cd(&'a Path),
}

#[derive(Clone, Debug, Fail)]
pub enum CdError {
    #[fail(display = "cd: no path provided")]
    NoPath,
}

// ===== impl Builtin =====

impl<'a> Parse<'a> for Builtin<'a> {
    type Error = CdError;
    fn parse_from(text: &'a str) -> Result<Self, ParseError<Self::Error>> {
        let mut args = super::ArgsIter { text };
        match args.next().ok_or(ParseError::NoInput)? {
            "clear" => Ok(Builtin::Clear),
            "cd" => {
                let path = args.next().ok_or(CdError::NoPath)?;
                Ok(Builtin::Cd(Path::new(path)))
            }
            _ => Err(ParseError::Unrecognized),
        }
    }
}
