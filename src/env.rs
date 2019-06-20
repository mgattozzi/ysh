//! Manages the set of environment variables for the shell and its jobs.

use super::token;
use std::iter::Iterator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnvVar<'a> {
    pub(crate) key: &'a str,
    pub(crate) value: &'a str,
}

impl<'a> EnvVar<'a> {
    pub fn new(key: &'a str, value: &'a str) -> Self {
        Self { key, value }
    }
}

impl<'a> From<(&'a str, &'a str)> for EnvVar<'a> {
    fn from((key, value): (&'a str, &'a str)) -> Self {
        Self { key, value }
    }
}

/// Borrows a command text and iterates over it for environment variables.
///
/// Valid environment-variable text has the structure:
///
/// ```text
/// [key=value ]* command text ...
/// ```
///
/// The key=value pairs must occur at the front of the text. The keys must be
/// bare words. They are broken by whitespace, and are not concerned with
/// punctiation. It is valid, though foolish, to use a key `some"text`. There
/// must be no whitespace between the key, the equals sign, and the beginning of
/// the value. The value may be any text span as defined by the `parse::span`
/// function: a single bare word, a single-quoted string, or a double-quoted
/// string.
#[derive(Clone, Debug)]
pub(crate) struct EnvIter<'a> {
    pub(crate) text: &'a str,
}

impl<'a> EnvIter<'a> {
    /// Creates a new envvar iterator from a command text.
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Iterator for EnvIter<'a> {
    /// Environment variables are key=value pairs. The output type is
    /// `EnvVar { key, value }`.
    type Item = EnvVar<'a>;

    /// Seeks the next environment variable in the text.
    ///
    /// Once the next part of the text is not a key=value pair, the scan ends.
    /// The grammar does not permit environment variables to be set after the
    /// command proper has begun.
    fn next(&mut self) -> Option<Self::Item> {
        token::trim_start(token::keyval)(self.text)
            .map(|(rem, (key, value))| {
                self.text = rem;
                EnvVar::new(key, value)
            }).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env() {
        let text = r#"KEY=val TEST="good work" echo $TEST"#;
        let mut envs = EnvIter::new(text);

        let EnvVar { key, value } = envs.next().expect("source text runs twice");
        assert_eq!(key, "KEY");
        assert_eq!(value, "val");

        let EnvVar { key, value } = envs.next().expect("source text runs twice");
        assert_eq!(key, "TEST");
        assert_eq!(value, "good work");

        assert!(envs.next().is_none());
    }
}
