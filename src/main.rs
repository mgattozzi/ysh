#![feature(crate_visibility_modifier)]

use std::{
    io::{self, Write},
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
use duct::cmd;

mod ast;
mod term;
mod st;
mod parse;

use self::{
    parse::Parse,
    ast::{Cmd, Builtin},
    term::Term,
};

fn main() {
    // Put it in raw mode
    let mut screen = Screen::new(true);
    st::State::default()
        .init(&mut screen)
        .and_then(move |mut state| state.run(screen))
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            exit(1);
        });
}

impl st::State {
    // TODO(eliza): move this out of the main module?
    pub fn run(&mut self, mut screen: Screen) -> Result<(), Error> {
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
                        screen.backspace()?;
                    }
                },
                '\u{000D}' /* Enter */ => {
                    match Cmd::parse_from(str::from_utf8(&line)?) {
                        Err(e) => {
                            // TODO(eliza): handle parse errors!
                            continue;
                        },
                        Ok(Cmd::Builtin(Builtin::Clear)) => { screen.reset(&self)?; },
                        Ok(Cmd::Builtin(Builtin::Cd(to))) => {
                            screen.newline()?;
                            self.cd(to)
                                .or_else(|e| {
                                    screen.error("cd", &e)
                                })?;

                            screen.prompt(&self)?;
                        },
                        Ok(Cmd::Invoke(ref c)) => {
                            screen.newline()?;
                            cmd(c.command, c.args.clone())
                                .unchecked()
                                .stdout_capture()
                                .stderr_capture()
                                .run()
                                .map_err(Into::into)
                                .and_then(|exec| {
                                    if &exec.stdout != b"" {
                                        screen.command_output(&exec.stdout)?;
                                    } else if &exec.stderr != b"" {
                                        screen.command_output(&exec.stderr)?;
                                    }
                                    Ok(())
                                })
                                .or_else(|err: Error| {
                                    if err.find_root_cause()
                                        .downcast_ref::<io::Error>()
                                        .iter()
                                        .any(|e| e.kind() == io::ErrorKind::NotFound)
                                    {
                                        screen.not_found(&c.command.to_string_lossy())
                                    } else {
                                        screen.error("ysh", err)
                                    }
                                })?;

                            screen.prompt(&self)?;
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
}
