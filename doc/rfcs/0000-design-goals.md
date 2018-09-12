# Command Language Design Goals

The purpose of this RFC is to enumerate a number of broad design goals for the
`ysh` command language. It is **not** meant to be a complete or even partial
specification.

## Language Syntax

Since `ysh` is a shell, its syntax has different design considerations than a
general purpose programming language. For example, it is important that newlines
not be syntactically significant --- while it is acceptable for other whitespace
to have a semantic meaning, users will enter expressions at the command prompt
as well as in scripts, and requiring newlines for certain forms complicates
this.

Another concern is that common command-line tasks --- such as invoking a program
on the path --- should be very low-effort. Therefore, while many general-purpose
programming languages use bare identifiers to refer to variables, POSIX shells
prefix variables with the `$` sigil, to disambiguate them from commands. We will
likely want to do something similar.

Potential influences for syntax include:
- [tulip], a language designed for easy use in a REPL
- Redox's [Ion] shell
- Ruby
- ...

[tulip]: https://github.com/tulip-lang/tulip/blob/master/doc/intro.md#readme
[Ion]: https://doc.redox-os.org/book/userspace/ion/what_ion_is.html

## Language Semantics

The `ysh` command language *should* be a primarily expression-based language
language --- at a minimum, control flow constructs such as `if` should be
expressions rather than statements. This facilitates more expressive programming

### Typing Discipline

The `ysh` command language *should not* be "stringly typed". Users *should* be
able to annotate `ysh` expressions with types, but type annotations *should not*
be required for all expressions.

Whether this means that `ysh` scripts are statically typed with type inference
(similar as Haskell, Rust, et cetera), or are dynamically typed with optional
type annotations (similar to Python's [`typing` module]) will likely depend on
how scripts are interpreted by the shell. If scripts are interpreted
line-by-line, it will likely be challenging to implement true static typing, as
type-checking a program correctly requires analyzing the program as a whole, and
type annotations will have to be evaluated at runtime. However, if the entire
text of a script is parsed to an abstract syntax tree prior to executing it, it
will be possible to analyze and infer types.

[`typing` module]: https://docs.python.org/3/library/typing.html

### Functions

POSIX shell script functions are terrible (citation needed). In `ysh`, functions
*should* be capable of returning values.

## Backward Compatibility

TODO(eliza): write stuff here
