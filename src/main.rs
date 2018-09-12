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

mod ast;
mod term;
mod st;
mod parse;

use self::{
    parse::Parse,
    ast::{Cmd, Builtin},
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
                match Cmd::parse_from(str::from_utf8(&line)?) {

                    Err(e) => {
                        // TODO(eliza): handle parse errors!
                        continue;
                    },
                    Ok(Cmd::Builtin(Builtin::Clear)) => term::reset(&mut screen)?,
                    Ok(Cmd::Builtin(Builtin::Cd(_))) => unimplemented!("cd doesnt work yet"),
                    Ok(Cmd::Invoke(ref c)) => {
                        term::newline(&mut screen)?;
                        cmd(c.command, c.args.clone())
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
                            .or_else(|_: Error| term::not_found(&mut screen, &c.command.to_string_lossy()))?;

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
