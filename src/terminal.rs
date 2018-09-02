//! Utility functions to manipulate the terminal and cursor

use super::st::STATE;
use failure::{
    format_err,
    Error
};
use std::{
    str,
    io::{
        Write,
        StdoutLock,
    },
};
use termion::{
    raw::RawTerminal,
    terminal_size,
    clear::All,
    cursor::{
        Left,
        Down,
        Goto,
    },
};

pub fn reset(stdout: &mut RawTerminal<StdoutLock>) -> Result<(), Error> {
    write!(stdout, "{}{}", All, Goto(1,1))?;
    prompt(stdout)?;
    stdout.flush()?;
    Ok(())
}

pub fn newline(stdout: &mut RawTerminal<StdoutLock>) -> Result<(), Error> {
    write!(stdout, "{}", Down(1))?;
    let (x,_) = terminal_size()?;
    write!(stdout, "{}", Left(x))?;
    Ok(())
}

pub fn backspace(stdout: &mut RawTerminal<StdoutLock>) -> Result<(), Error> {
    write!(stdout,"{} {}", Left(1), Left(1))?;
    Ok(())
}

pub fn not_found(stdout: &mut RawTerminal<StdoutLock>, command: &str) -> Result<(), Error> {
    write!(stdout, "ysh: command not found: {}", command)?;
    newline(stdout)?;
    Ok(())
}

pub fn command_output(stdout: &mut RawTerminal<StdoutLock>, out: &Vec<u8>) -> Result<(), Error> {
    for i in str::from_utf8(out)?.lines() {
        write!(stdout, "{}", i)?;
        newline(stdout)?;
    }
    Ok(())
}
pub fn prompt(stdout: &mut RawTerminal<StdoutLock>) -> Result<(), Error> {
    write!(stdout, "{}", STATE.prompt.read().map_err(|_| format_err!("Poisoned Lock"))?);
    Ok(())
}
