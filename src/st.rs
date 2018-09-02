//! Functions and types dealing with the `State` of the shell
use std::{
    fs::{
        self,
    },
    env,
    sync::RwLock,
    io::{
        StdoutLock,
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

/// Gets the username that launched the shell.
///
/// This function will fail if:
/// - the current uid as returned by `libc::getuid()` does not have a valid
///     entry in the system user registry (unlikely)
/// - the user structure for the current uid has a nullpointer for the name
///     field (even more unlikely)
/// - the text behind the user structure's name pointer is invalid UTF-8 (less
///     unlikely)
#[cfg(target_family = "unix")]
pub fn user() -> Result<String, Error> {
    //  Unixy libc represent users as uid tokens (secretly u32, but details)
    let uid = unsafe { libc::getuid() };
    let passwd = unsafe { libc::getpwuid(uid) };
    if passwd.is_null() {
        bail!("User for ID {} not found in system registry!", uid);
    }
    let name = unsafe { (*passwd).pw_name };
    if name.is_null() {
        bail!("Username string for ID {} is a null pointer!", uid);
    }
    unsafe { std::ffi::CStr::from_ptr(name) }
        .to_str()
        .map(Into::into)
        .or_else(|_| bail!("Username is not UTF-8"))
}

// TODO(mgattozzi): Implement a version of user() for winapi
