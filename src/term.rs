//! Utility functions to manipulate the terminal and cursor

use super::st::STATE;
use failure::{
    format_err,
    Error
};
use crossterm::{
    Screen,
    cursor,
    terminal::terminal,
    terminal::ClearType,
};
use std::fmt;
use std::io::Write;
use std::str;

pub fn reset(screen: &mut Screen) -> Result<(), Error> {
    let cursor = cursor(&screen);
    let term = terminal(&screen);
    term.clear(ClearType::All);
    cursor.goto(0,0);
    prompt(screen)?;
    Ok(())
}

pub fn newline(screen: &mut Screen) -> Result<(), Error> {
    screen.write(b"\r\n")?;
    screen.flush()?;
    Ok(())
}

pub fn backspace(screen: &mut Screen) -> Result<(), Error> {
    let mut cursor = cursor(&screen);
    let term = terminal(&screen);
    cursor.move_left(1);
    term.clear(ClearType::UntilNewLine);
    Ok(())
}

pub fn not_found<C: fmt::Display>(screen: &mut Screen, command: &C) -> Result<(), Error> {
    write!(screen, "ysh: command not found: {}", command)?;
    newline(screen)?;
    screen.flush()?;
    Ok(())
}

pub fn command_output(screen: &mut Screen, out: &Vec<u8>) -> Result<(), Error> {
    #[cfg(unix)]
    for i in str::from_utf8(out)?.lines() {
        screen.write(i.as_bytes())?;
        newline(screen)?;
    }

    #[cfg(windows)]
    screen.write(out)?;

    screen.flush()?;
    Ok(())
}

pub fn prompt(screen: &mut Screen) -> Result<(), Error> {
    screen.write(STATE.prompt.read().map_err(|_| format_err!("Poisoned Lock"))?.as_bytes())?;
    screen.flush()?;
    Ok(())
}
