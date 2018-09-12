#![feature(crate_visibility_modifier)]

use std::{
    str,
    process::exit,
};
use failure::{
    Error,
};
use crossterm::{
    Screen,
    input,
};
use std::io::Write;
use duct::cmd;
mod term;
mod st;
mod parse;

use self::parse::{
    span,
};

fn main() {
    // Put it in raw mode
    let mut screen = Screen::new(true);

    st::init(&mut screen).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1);
    });

    run(screen).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1);
    });
}


fn run(mut screen: Screen) -> Result<(), Error> {
    let mut line = Vec::new();
    loop {
        let stdin = input(&screen);
        match stdin.read_char()? {
            // ESC Key
            '\u{001B}' => break,
            // Backspace and Delete because on *nix it can send either or to mean the same thing
            '\u{0008}' | '\u{007F}' => {
                if line.len() > 0 {
                    line.pop();
                    term::backspace(&mut screen)?;
                }
            },
            '\u{000D}' /* Enter */ => {
                match str::from_utf8(&line)? {
                    "" => continue,
                    "clear" => term::reset(&mut screen)?,
                    command => {
                        term::newline(&mut screen)?;
                        //  args is a collection of &'line str snippets --
                        //  pointers into the line buffer above
                        let mut args: Vec<&str> = Vec::new();
                        //  The command binding is used below, so we need a
                        //  separate cursor for the tokenizing
                        let mut text: &str = command;
                        while text.len() > 0 {
                            //  Use the span tokenizer to get a snippet
                            let (rest, span) = span(text.into())
                                //  Suppress the errors for now. May be worth
                                //  investigating so that the shell can repont
                                //  invalid syntax?
                                .map_err(|_| failure::format_err!("Invalid text"))?;
                                //  rest and span are CompleteStr, which implements
                                //  Deref down to &str.
                                args.push(*span);
                                text = *rest;
                        }
                        if args.is_empty() {
                            screen.flush()?;
                            continue;
                        }
                        cmd(args[0], &args[1..])
                            .unchecked()
                            .stdout_capture()
                            .stderr_capture()
                            .run()
                            .map_err(Into::into)
                            .and_then(|exec| {
                                if &exec.stdout != b"" {
                                    term::command_output(&mut screen, &exec.stdout)?;
                                } else if &exec.stderr != b"" {
                                    term::command_output(&mut screen, &exec.stderr)?;
                                }
                                Ok(())
                            })
                            .or_else(|_: Error| term::not_found(&mut screen, command))?;

                        term::prompt(&mut screen)?;
                    }
                }
                line.clear();
            },
            // Only printable ASCII characters
            c if c as u8 >= 32 && c as u8 <= 126 => {
                line.push(c as u8);
                screen.write(&[c as u8])?;
            },
            _ => {}
        }
        screen.flush()?;
    }
    Ok(())
}
