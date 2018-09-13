//! Functions and types dealing with the `State` of the shell
use std::{
    env,
    fmt::Write,
    io,
    path::{Path, PathBuf},
};
use crossterm::Screen;
use super::term::Term;
use failure::{
    bail,
    format_err,
    Error,
};

#[cfg(unix)]
use std::fs;

#[cfg(windows)]
use winapi::um::winbase::{
    GetComputerNameA,
    GetUserNameA,
};

#[derive(Default)]
pub struct State {
    pub pwd: PathBuf,
    pub host: String,
    pub user: String,
}

impl State {

    pub fn init(self, screen: &mut Screen) -> Result<Self, Error> {
        let host = hostname()?;
        let user = user()?;
        env::set_var("HOST", &host);
        env::set_var("USER", &user);

        let pwd = env::current_dir()?;

        let this = Self {
            pwd,
            host,
            user,
            ..self
        };
        screen.reset(&this)?;

        Ok(this)
    }

    pub fn cd<P: AsRef<Path>>(&mut self, to: P) -> io::Result<()> {
        let to = to.as_ref()
            .canonicalize()?;
        env::set_current_dir(&to)?;
        self.pwd = to;
        Ok(())
    }
}

/// Gets the hostname of the machine running the shell.
#[cfg(target_family = "unix")]
pub fn hostname() -> Result<String, Error> {
    let mut buf: Vec<u8> = Vec::with_capacity(32);
    //  The linter doesn't set any targets, and so is unable to see that we are,
    //  in fact, assigning to this.
    #[allow(unused_mut)]
    let mut res: libc::c_int;
    loop {
        //  When the buffer gets ridiculously large, abort
        //  (note: POSIX currently says the maximum is 256)
        if buf.len() > 1024 {
            bail!("Hostname is too large!");
        }
        //  C has a bad API for buffers. Get the pointer and capacity of our Vec
        let (ptr, cap) = (buf.as_mut_ptr() as *mut libc::c_char, buf.capacity());
        //  and call gethostname so it can write into it.
        res = unsafe { libc::gethostname(ptr, cap) };

        //  gethostname returns -1 on failure and sets errno, instead of
        //  returning the error directly.
        if res == -1 {
            //  cfg can't be placed on if-statements, so the errno checks are
            //  wrapped in blocks.
            //  this is only a problem because linux and mac libc have different
            //  functions for finding errno
            #[cfg(target_os = "linux")] {
            if unsafe { *libc::__errno_location() } == libc::ENAMETOOLONG {
                buf.reserve(buf.capacity());
                continue;
            } }
            #[cfg(target_os = "mac")] {
            if unsafe { *libc::__error() } == libc::ENAMETOOLONG {
                buf.reserve(buf.capacity());
                continue;
            } }
            unreachable!("gethostname can only fail with ENAMETOOLONG");
        }
        //  gethostname has now succeeded! buf holds the contents of a CStr.
        //  This code will attempt to reinterpret buf as a String in place, so
        //  that no reallocation is required. Performance!
        break unsafe {std::ffi::CStr::from_ptr(ptr) }.to_str()
            .map(|cs| unsafe {
                buf.set_len(cs.len());
                buf.shrink_to_fit();
                String::from_utf8_unchecked(buf)
            })
            .or_else(|_| bail!("The hostname is invalid UTF-8"));
    }
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
    //  Get a pointer into thread-local storage for the user structure matching
    //  the obtained uid.
    let passwd = unsafe { libc::getpwuid(uid) };
    //  Abort if nullptr
    if passwd.is_null() {
        bail!("User for ID {} not found in system registry!", uid);
    }
    //  Get the name member, which is *const c_char
    let name = unsafe { (*passwd).pw_name };
    //  Abort if that is nullptr
    if name.is_null() {
        bail!("Username string for ID {} is a null pointer!", uid);
    }
    //  Read the data behind the pointer as a CStr
    unsafe { std::ffi::CStr::from_ptr(name) }
        //  Attempt to read it as UTF-8
        .to_str()
        //  And take ownership of it as a String if it is
        .map(Into::into)
        //  Or fail if it is not
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
