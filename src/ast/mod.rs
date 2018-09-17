use std::str;

use crate::parse::{self, Parse, ParseError};
use crate::env::EnvIter;
pub mod builtin;
mod invoke;

pub use self::builtin::Builtin;
pub use self::invoke::Invoke;

#[derive(Debug, Clone)]
pub struct ArgsIter<'a> {
    text: &'a str
}

impl<'a> std::iter::Iterator for ArgsIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        //  Use the tokenizer to get a snippet
        let (rest, span) = parse::apply(str::trim_left, parse::token)(self.text)
            //  Suppress the errors for now. May be worth investigating so that
            //  the shell can report invalid syntax?
            .ok()?;
        self.text = rest;
        Some(span)
    }
}

#[derive(Clone, Debug)]
pub enum Cmd<'a> {
    Builtin(builtin::Builtin<'a>),
    Invoke(Invoke<'a>),
}

impl<'a> Parse<'a> for Cmd<'a> {
    type Error = String; // placeholder
    fn parse_from(s: &'a str) -> Result<Self, ParseError<Self::Error>> {
        Builtin::parse_from(s)
            .map(Cmd::Builtin)
            .or_else(|_| Invoke::parse_from(s).map(Cmd::Invoke))
    }
}

/// Evaluate a command in an environment.
///
/// This represents forms such as
///
/// ```text
/// FOO=foo BAR=bar command
/// ```
///
/// and
///
/// ```text
/// env FOO=foo BAR=bar command
/// ```
#[derive(Clone, Debug)]
pub struct WithEnv<'a> {
    crate env: EnvIter<'a>,
    crate cmd: Cmd<'a>,
}

impl<'a> Parse<'a> for WithEnv<'a> {
    type Error = String; // placeholder
    fn parse_from(s: &'a str) -> Result<Self, ParseError<Self::Error>> {
        let env = EnvIter::new(s);
        //  fast-forward an EnvIter over the string, until it runs out of env
        //  vars
        let mut ei = env.clone();
        ei.by_ref().for_each(drop);
        //  get the fast-forwarded text
        let EnvIter { text } = ei;
        let cmd_str = text;
        let cmd = Cmd::parse_from(cmd_str)?;
        Ok(Self { env, cmd })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::EnvVar;

    #[test]
    fn without_env() {
        let text = r#"command argument "complex argument""#;
        let with_env = WithEnv::parse_from(text).expect("source is correct");

        let WithEnv { env, cmd } = with_env;
        assert_eq!(env.count(), 0);
        match cmd {
            Cmd::Builtin(_) => panic!("'command' is not a builtin"),
            Cmd::Invoke(Invoke { command, args }) => {
                assert_eq!(command, "command");
                assert_eq!(args.clone().count(), 2);
                assert_eq!(args.collect::<Vec<_>>(), &["argument", "complex argument"]);
            },
        }
    }

    #[test]
    fn with_env() {
        let text = r#"TEST=1 AUTHOR=myrrlyn cd 'complex path'"#;
        let with_env = WithEnv::parse_from(text).expect("source is correct");

        let WithEnv { env, cmd } = with_env;
        assert_eq!(env.clone().count(), 2);
        assert_eq!(
            env.collect::<Vec<_>>(),
            vec![("TEST", "1"), ("AUTHOR", "myrrlyn")].into_iter()
                .map(EnvVar::from)
                .collect::<Vec<_>>()
        );
        match cmd {
            Cmd::Invoke(_) => panic!("'cd' is a builtin"),
            Cmd::Builtin(Builtin::Cd(path)) => {
                assert_eq!(
                    path.to_str().expect("source is valid str"),
                    "complex path",
                );
            },
            Cmd::Builtin(_) => panic!("'cd' is only the builtin 'Cd'"),
        }
    }
}
