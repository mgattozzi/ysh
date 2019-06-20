//! Tokenizers
//!
//! This module holds the functions that recognize tokens from an input text.
//! These functions are designed to be used by grammar-aware parsers to
//! translate an input text into components of `ysh` data structures. The token
//! producers have no awareness of any language rules other than the syntax
//! required to produce valid tokens of their own type. Parsers that call these
//! token producers are responsible for semantic analysis of the produced data.
//!
//! Note that by default, the tokenizers in this module do not perform any
//! whitespace trimming on their input. The function `trim_start` can modifiy a
//! tokenizer to produce another tokenizer that trims leading whitespace before
//! analyzing the text.

/// Result type for the `nom`-style tokenizer functions.
///
/// Tokenizers always operate on strings that are fully loaded in memory, but
/// may be incompletely entered by the user. As such, `CompleteStr` is not used,
/// so that the parse mechanisms can signal that more input is required and to
/// await another entry cycle.
///
/// The `Ok` variant of this type is a tuple whose left member is the remaining
/// unparsed text and whose right member is the parsed value, which may be
/// supplied by the specific `TokenResult` site. By default, the parsed type is
/// part of the input text, and not a new structure.
///
/// The error codes used by the `nom` routines are **not** `failure::Error`, but
/// are small atoms which must convert to `failure::Error` by the `Parse` trait.
/// By default, the error significand is a `u32` to match the error significands
/// provided by `nom`.
pub type TokenResult<'a, T = &'a str, E = u32> = nom::IResult<&'a str, T, E>;

/// Composes an input transformer with a tokenizer.
///
/// This function modifies an arbitrary tokenizer by running a function that
/// transforms the input before running the tokenizer on the result of the
/// transform. The return value of this function *is a tokenizer function*; this
/// function performs no parsing work, and runs wholly at compile time.
///
/// For example, this can be used to make a non-whitespace-trimming tokenizer
/// trim all leading whitespace before attempting the parse, by changing the
/// call site from `tokenizer(text)` to
/// `compose(str::trim_start, tokenizer)(text)`.
///
/// # Usage
///
/// ```rust
/// use std::str;
/// use ysh::token;
///
/// let (rem, atom) = token::compose(str::trim_start, token::atom)("  'atom'  ")
///     .expect("atom will succeed, because the whitespace will be trimmed");
/// assert_eq!(atom, "atom");
/// assert_eq!(rem, "  ");
/// ```
pub fn compose<'a, T, U, V>(
    modifier: impl Fn(T) -> U,
    tokenizer: impl Fn(U) -> TokenResult<'a, V>,
) -> impl Fn(T) -> TokenResult<'a, V> {
    move |input: T| tokenizer(modifier(input))
}

/// Modifies a tokenizer to call `trim_start` on the input before processing it.
///
/// # Usage
///
/// ```rust
/// use ysh::token;
///
/// let (rem, atom) = token::trim_start(token::atom)("  'an atom'  ")
///     .expect("atom will succeed, because the whitespace will be trimmed");
/// assert_eq!(atom, "an atom");
/// assert_eq!(rem, "  ");
/// ```
pub fn trim_start<'a, T>(
    tokenizer: impl Fn(&'a str) -> TokenResult<'a, T>,
) -> impl Fn(&'a str) -> TokenResult<'a, T> {
    compose(str::trim_start, tokenizer)
}

/// Finds any token element.
///
/// This token may be one of:
///
/// - a shell meta-sequence (`shell_meta`)
/// - a double-quoted string (`dquote`)
/// - a single-quoted string (`squote`)
/// - a bare word (`word`)
///
/// This tokenizer is highly general, and should only be used when the token
/// type produced is not needed. In the future, it may be modified to return an
/// enum indicating the token type returned.
///
/// `atom` returns the unmodified result of the first tokenizer to succeed. The
/// quoted string tokenizers do not return the quotes, and the shell tokenizer
/// does not return the dollar sign.
///
/// # Usage
///
/// ```rust
/// use ysh::token::{atom, trim_start};
///
/// let (rem, dquo) = trim_start(atom)("\"hello\" 'world' ${shell:-1} word")
///     .expect("a double-quoted string is a valid atom");
/// assert_eq!(dquo, "hello");
///
/// let (rem, squo) = trim_start(atom)(rem)
///     .expect("a single-quoted string is a valid atom");
/// assert_eq!(squo, "world");
///
/// let (rem, shell) = trim_start(atom)(rem)
///     .expect("a shell meta-sequence is a valid atom");
/// assert_eq!(shell, "{shell:-1}");
///
/// let (_, word) = trim_start(atom)(rem)
///     .expect("a bare word is a valid atom");
/// assert_eq!(word, "word");
/// ```
pub fn atom(text: &str) -> TokenResult {
    use nom::alt;
    //  TODO(myrrlyn): Patch nom to not leak error_position from alt
    use nom::error_position;
    alt!(text, shell_meta | dquote | squote | word)
}

/// Finds a bare word.
///
/// A word is defined as any run of non-whitespace characters. This tokenizer
/// currently does not attempt to interpret any character significance other
/// than whitespace, and will happily include punctuation in its concept of a
/// word. This means that the text `"hello world"` will, under `word()`, produce
/// two tokens: `"hello` and `world"`.
///
/// # Usage
///
/// ```rust
/// use ysh::token::word;
///
/// let (rem, val) = word("hello world")
///     .expect("bare words are easily tokenized");
/// assert_eq!(val, "hello");
/// assert_eq!(rem, " world");
/// ```
///
/// `word` does not perform any whitespace manipulation (none of the tokenizers
/// do), and it will not accept an empty sequence as a valid word.
///
/// ```rust
/// # use ysh::token::word;
/// assert!(word(" hello").is_err());
/// ```
pub fn word(text: &str) -> TokenResult {
    use nom::take_till1;
    use nom::Err;
    take_till1!(text, char::is_whitespace)
        .or_else(|e| match e {
            //  If take_till grabbed text, it is a valid word even at EOF.
            Err::Incomplete(_) if !text.is_empty() => Ok(("", text)),
            e => Err(e),
        })
}

/// Finds a single-quote-delimited string.
///
/// This tokenizer is the regex `/'([^']*)'/`. If the text it is given begins
/// with a single quote character (U+0027), it produces all text between that
/// character and the next occurrence of single quote U+0027. It performs no
/// escape analysis, and will permit backslash `\` (U+005C) as the penultimate
/// character before the terminating single quote.
///
/// The success value is the text between the quotes, and the text after the
/// terminating quote.
///
/// # Usage
///
/// ```rust
/// use ysh::token::squote;
///
/// let (rest, squo) = squote("'hello world' is a single token")
///     .expect("single-quoted strings are easily tokenized");
/// assert_eq!(squo, "hello world");
/// assert_eq!(rest, " is a single token");
/// ```
///
/// Note the leading space in the `rest` return value. These tokenizers will
/// never produce a remnant less than the full text immediately at their halting
/// point.
pub fn squote(text: &str) -> TokenResult {
    use nom::{delimited, tag, take_until};
    delimited!(text, tag!("'"), take_until!("'"), tag!("'"))
}

/// Finds a double-quote-delimited string.
///
/// Double-quoted text is able to embed escape and shell-meta sequences. Shell
/// meta-sequences may *themselves* contain double-quote spans.
///
/// If the given text begins with a double-quote character (U+0022), then the
/// tokenizer advances through the text, detecting and fast-forwarding through
/// escape and shell-meta sequences, until it detects an unescaped double-quote
/// character terminating the string.
///
/// The success value is the text between the quotes, and the text after the
/// terminating quote. This tokenizer performs no escape analysis on any
/// backslash sequences, and only uses backslashes to skip the next character.
/// Any encountered backslashes and their suffixes will be returned as-received
/// in the output.
///
/// # Usage
///
/// ```rust
/// use ysh::token::dquote;
///
/// let text = r#""dquotes \"may nest\" and $(even "nest shells")"excluded"#;
///
/// let (rest, dq) = dquote(text)
///     .expect("double-quoted strings are tokenized with difficulty");
/// assert_eq!(dq, r#"dquotes \"may nest\" and $(even "nest shells")"#);
/// assert_eq!(rest, "excluded");
/// ```
pub fn dquote(text: &str) -> TokenResult {
    use nom::Err;
    use nom::Needed;
    use nom::tag;
    if text.trim().is_empty() {
        return Err(Err::Incomplete(Needed::Size(2)));
    }

    //  Find the opening quote and create a crawler over the text.
    let (text, _) = tag!(text, "\"")?;
    let mut iter = text.char_indices().fuse();

    while let Some((i, c)) = iter.next() {
        match c {
            //  A double quote ends the token. Return the text up to the current
            //  position as the token and the text after the current position as
            //  the remnant.
            '"' => return Ok((&text[i + 1 ..], &text[.. i])),
            //  A backslash (U+005C) skips the next character.
            //  TODO(myrrlyn): Make a backslash processor
            '\\' => drop(iter.next()),
            //  An unescaped dollar sign (U+0024) begins a shell meta-sequence.
            //  - process the entire sequence, `shell_meta(...)?`
            //  - take the shell sequence, `.1`
            //  - iterate over its characters, `.chars()`
            //  - advance the main iterator for each of them, `.for_each(...)`
            '$' => shell_meta(&text[i ..])?.1.chars()
                .for_each(|_| drop(iter.next())),
            //  ALl other characters are uninteresting
            _ => continue,
        }
    }

    //  If the iterator exhausts without reaching the exit point above, the
    //  sequence is incomplete.
    Err(Err::Incomplete(Needed::Size(1)))
}

/// Finds a `key=value` sequence and splits it into the key and the value.
///
/// This tokenizer uses `word` on the left side of the equals sign and `atom` on
/// the right. It rejects any whitespace between the key, the equals sign, and
/// the value.
///
/// # Usage
///
/// ```rust
/// use ysh::token::keyval;
///
/// let (_, (key, value)) = keyval("hello=\"dear reader\"")
///     .expect("key/value pairs can have any atom as their value");
/// assert_eq!(key, "hello");
/// assert_eq!(value, "dear reader");
/// ```
pub fn keyval(text: &str) -> TokenResult<(&str, &str)> {
    use nom::tag;
    use nom::take_until1;
    //  TODO(myrrlyn): Patch nom to not leak error_position from take_until1
    use nom::error_position;
    //  Take from the start up to the equals sign
    let (rem, key) = take_until1!(text, "=")
        //  And grab the first word of that sequence. `word` cannot return
        //  Incomplete, and will unconditionally provide a single word.
        .and_then(|(_, t)| word(t))
        //  The remnant is now everything after `word`'s success value.
        .map(|(_, w)| (&text[w.len() ..], w))?;
    //  The next character after `word` **must** be `=`.
    let (rem, _) = tag!(rem, "=")?;
    //  Take the next atom.
    let (rem, val) = atom(rem)?;
    Ok((rem, (key, val)))
}

/// Finds a shell meta-sequence.
///
/// A shell meta-sequence begins with a dollar sign character, `$` (U+0024), and
/// is followed by one of:
///
/// - a parentheses-enclesed sequence, `(text)`, indicating a subshell command
/// - a brace-enclosed sequence, `{text}`, indicating a variable expansion
/// - a bare word, `text`, indicating a variable expansion
///
/// Shell meta-sequences can create arbitrarily deep recursive structures with
/// other shell meta-sequences or with double-quoted strings. For example, the
/// text below nests subshell, variable, and double-quote tokens to demonstrate
/// that each begins a new sequence which must end before its enveloping
/// sequence:
///
/// ```text
/// "top $(high "middle $(low ${bottom} low) middle" high) top"
///                           ^^^^^^^^
///                     ^^^^^^^^^^^^^^^^^^^
///             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
///
/// The text above tokenizes as shown, and does **not** end the "top"
/// double-quoted string at the quote before `middle`, nor does it end the
/// "high" shell meta-sequence at the parenthesis after `low`, etc.
///
/// # Usage
///
/// Find a bare variable name, like `$foo`.
///
/// ```rust
/// use ysh::token::shell_meta;
///
/// let (_, var) = shell_meta("$var")
///     .expect("bare words are valid");
/// assert_eq!(var, "var");
/// ```
///
/// Find a more complex variable use, like `${foo}` or `${foo:-default}`.
///
/// ```rust
/// # use ysh::token::shell_meta;
/// let (_, var) = shell_meta(r#"${var:-"default value"}"#)
///     .expect("brace sequences are valid");
/// assert_eq!(var, "{var:-\"default value\"}");
/// ```
///
/// Find a subshell invocation, like `$(command arguments...)`.
///
/// ```rust
/// # use ysh::token::shell_meta;
/// let (_, shell) = shell_meta("$(cmd $(inner))")
///     .expect("subshells can have inner subshells or other constructs");
/// assert_eq!(shell, "(cmd $(inner))");
/// ```
pub fn shell_meta(text: &str) -> TokenResult {
    use nom::tag;
    use nom::Err;
    use nom::Needed;
    let (text, _) = tag!(text, "$")?;
    let close = match text.clone().chars().next() {
        Some('(') => ')',
        Some('{') => '}',
        //  If no opening punctuation was found, seek a bare word and return it
        //  directly.
        Some(_) => return word(text),
        //  If no characters come after the `$`, then abort as incomplete.
        None => return Err(Err::Incomplete(Needed::Unknown)),
    };
    let mut rem = &text[1 ..];
    //  Search the text for the end of the meta sequence. Large tokens (dquote,
    //  squote, and shell_meta) jump the search. This loop is finite: all the
    //  called tokenizers advance the text, and it will eventually become empty
    //  and terminate as incomplete.
    'outer: loop {
        if rem.trim().is_empty() {
            return Err(Err::Incomplete(Needed::Unknown));
        }
        //  If any of these bulk tokenizers match on the text, fast-forward
        //  through them.
        for tokenizer in &[dquote, squote, shell_meta] {
            if let Ok((rest, _)) = trim_start(tokenizer)(rem) {
                rem = rest;
                continue 'outer;
            }
        }
        //  Otherwise inspect the next character
        match rem.clone().chars().next() {
            //  If it's the matching closer to the opener found above, return
            Some(c) if c == close => {
                let len = text.len() - rem.len();
                return Ok((&text[len + 1 ..], &text[..= len]));
            },
            //  Otherwise, advance the cursor by the UTF-8 length of the char
            Some(c) => rem = &rem[c.len_utf8() ..],
            //  If there is no next character, then no terminator was found, and
            //  the sequence is incomplete.
            None => return Err(Err::Incomplete(Needed::Unknown)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applicative() {
        use std::str;

        compose(str::trim_start, squote)("  'squoted'")
            .expect("squote must succeed");

        trim_start(squote)("  'squoted'")
            .expect("squote must succeed");
    }

    #[test]
    fn token_word() {
        let (rest, part) = word("hello world").expect("word must succeed");
        assert_eq!(part, "hello");
        assert_eq!(rest, " world");

        //  a blank string is NOT a word
        assert!(word("").is_err());
    }

    #[test]
    fn token_squote() {
        let (rest, part) = squote("'hello world' is one token")
            .expect("squote must succeed");
        assert_eq!(part, "hello world");
        assert_eq!(rest, " is one token");
    }

    #[test]
    fn dquote_plain() {
        let (rest, part) = dquote(r#""hello world" is one token"#)
            .expect("dquote must succeed");
        //  dquote's return does not include the quotes
        assert_eq!(part, "hello world");
        assert_eq!(rest, " is one token");
    }

    #[test]
    fn dquote_escapes() {
        let (rest, part) = dquote(r#""hello \\ \"world\"" may have backslashes"#)
            .expect("dquote must succeed");
        //  dquote does nothing with \ except skip one character
        assert_eq!(part, r#"hello \\ \"world\""#);
        assert_eq!(rest, " may have backslashes");
    }

    #[test]
    fn dquote_shell() {
        let (_, part) = dquote(r#""dquote $(may "nest")""#)
            .expect("dquote must succeed");

        assert_eq!(part, "dquote $(may \"nest\")");
    }

    #[test]
    fn dquote_edge() {
        //  must have opening and closing quotes
        assert!(dquote("hello").is_err());
        assert!(dquote("\"unterminated").is_err());
        //  must not be empty
        assert!(dquote("").is_err());
        //  may be empty inside
        assert!(dquote(r#""""#).is_ok());
    }

    #[test]
    fn subshell() {
        let (_, v) = shell_meta("$var").expect("shell_meta must succeed");
        assert_eq!(v, "var");

        let (r, v) = shell_meta("${var},").expect("shell_meta must succeed");
        assert_eq!(v, "{var}");
        assert_eq!(r, ",");

        let (r, s) = shell_meta("$(subshell),").expect("shell_meta must succeed");
        assert_eq!(s, "(subshell)");
        assert_eq!(r, ",");
    }

    #[test]
    fn nested_subshell() {
        let (_, s) = shell_meta("$(cmd \"inner string\")")
            .expect("shell_meta must succeed");

        assert_eq!(s, "(cmd \"inner string\")");
    }

    #[test]
    fn token_keyval() {
        let (rest, (key, val)) = keyval("hello=world pair")
            .expect("keyval must succeed");
        assert_eq!(key, "hello");
        assert_eq!(val, "world");
        assert_eq!(rest, " pair");
    }

    #[test]
    fn keyval_edge() {
        assert!(keyval("one two=three").is_err());
        assert!(keyval("one=\"two three\"").is_ok());
        match keyval("one=\"two three\"") {
            Ok((_, (_, val))) => assert_eq!(val, "two three"),
            _ => panic!("keyval must succeed"),
        }
    }
}
