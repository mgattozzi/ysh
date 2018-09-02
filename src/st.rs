//! Functions and types dealing with the `State` of the shell
use std::{
    fs::{
        self,
        File,
    },
    env,
    sync::RwLock,
    io::{
        BufReader,
        StdoutLock,
        BufRead,
    },
};
use super::terminal;
use termion::raw::RawTerminal;
use failure::{
    bail,
    format_err,
    Error,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref STATE: State = State::new();
}

pub struct State {
    pub prompt: RwLock<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            prompt: RwLock::new(String::new()),
        }
    }
}

pub fn init(stdout: &mut RawTerminal<StdoutLock>) -> Result<(), Error> {
    let host = hostname()?;
    let user = user()?;
    env::set_var("HOST", &host);
    env::set_var("USER", &user);

    // We need to drop the write lock here so that the terminal can print out the prompt
    {
        let mut prompt = STATE.prompt.write().map_err(|_| format_err!("Poisoned Lock"))?;
        prompt.push_str(&user);
        prompt.push('@');
        prompt.push_str(&host);
        prompt.push_str(" % ");
    }

    terminal::reset(stdout)?;

    Ok(())
}

pub fn hostname() -> Result<String, Error> {
    Ok(fs::read_to_string("/etc/hostname")?.trim().into())
}

pub fn user() -> Result<String, Error> {
    let uid = unsafe { libc::getuid() };
    let mut buf = BufReader::new(File::open("/etc/passwd")?);
    let mut line = String::new();
    while buf.read_line(&mut line)? > 0 {
        if line.contains(&uid.to_string()) {
            return Ok(line.split(':').next().unwrap().into())
        }
        line.clear();
    }
    bail!("User not found in /etc/passwd")
}
