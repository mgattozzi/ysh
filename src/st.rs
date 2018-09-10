//! Functions and types dealing with the `State` of the shell
use std::{
    env,
    sync::RwLock,
};
use crossterm::Screen;
use super::term;
use failure::{
    bail,
    format_err,
    Error,
};
use lazy_static::lazy_static;

#[cfg(unix)]
use std::fs;

#[cfg(windows)]
use winapi::um::winbase::{
    GetComputerNameA,
    GetUserNameA,
};

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

pub fn init(mut screen: &mut Screen) -> Result<(), Error> {
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

    term::reset(&mut screen)?;

    Ok(())
}

#[cfg(target_family = "unix")]
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


#[cfg(target_family = "windows")]
pub fn hostname() -> Result<String, Error> {
    // We want the Max length for NetBIOS names which is 15 chars
    // https://docs.microsoft.com/en-us/windows/desktop/sysinfo/computer-names
    const MAX_COMPUTERNAME_LENGTH: usize = 15;
    // The buffer needs to hold the constant above + 1 chars
    // https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-getcomputernamea
    const LENGTH: usize = MAX_COMPUTERNAME_LENGTH + 1;
    // Create a zeroed out buffer. Windows uses i8 to represent chars
    let mut buffer = [0 as i8; LENGTH];
    unsafe {
        // Make the call, a value of 0 means it was unable to get the Hostname, we should
        // fail the program here with an error message
        if GetComputerNameA(buffer.as_mut_ptr(), &mut (LENGTH as u32)) == 0 {
            bail!("Unable to get hostname from call to GetComputerNameA in Windows API")
        };
    }
    // winapi uses C Strings but we want it without all the null chars so we convert the
    // buffer into a CStr and then into a proper Rust String failing if it's not UTF-8
    unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }
        .to_str()
        .map(Into::into)
        .or_else(|_| bail!("Hostname is not UTF-8"))
}
#[cfg(target_family = "windows")]
pub fn user() -> Result<String, Error> {
    // We want to have this value UNLEN for below. UNLEN is defined here as 256
    // https://msdn.microsoft.com/en-us/library/cc761107.aspx
    const UNLEN: usize = 256;
    // The buffer needs to hold the constant above + 1 chars
    // https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-getusernamea
    const LENGTH: usize = UNLEN + 1;
    // Create a zeroed out buffer. Windows uses i8 to represent chars
    let mut buffer = [0 as i8; LENGTH];
    unsafe {
        // Make the call, a value of 0 means it was unable to get the username, we should
        // fail the program here with an error message
        if GetUserNameA(buffer.as_mut_ptr(), &mut (LENGTH as u32)) == 0 {
            bail!("Unable to get username from call to GetUserNameA in Windows API")
        };
    }
    // winapi uses C Strings but we want it without all the null chars so we convert the
    // buffer into a CStr and then into a proper Rust String failing if it's not UTF-8
    unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }
        .to_str()
        .map(Into::into)
        .or_else(|_| bail!("Username is not UTF-8"))
}
