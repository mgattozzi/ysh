//! Utility functions to manipulate the terminal and cursor
use failure::{
    format_err,
    Error
};
use crossterm::{
    Screen,
    cursor,
    terminal::{self, ClearType},
};
use std::fmt;
use std::io::Write;
use std::str;
use crate::st;

pub trait Term: Write + Sized {
    fn cursor(&self) -> cursor::TerminalCursor;
    fn terminal(&self) -> terminal::Terminal;

    fn reset(&mut self, state: &st::State) -> Result<(), Error> {
        let cursor = self.cursor();
        let term = self.terminal();
        term.clear(ClearType::All);
        cursor.goto(0,0);
        self.prompt(state)
    }

    fn newline(&mut self) -> Result<(), Error> {
        self.write(b"\r\n")?;
        self.flush()?;
        Ok(())
    }

    fn backspace(&mut self) -> Result<(), Error> {
        let mut cursor = self.cursor();
        let term = self.terminal();
        cursor.move_left(1);
        term.clear(ClearType::UntilNewLine);
        Ok(())
    }

    fn not_found(&mut self, command: &str) -> Result<(), Error> {
        write!(self, "ysh: command not found: {}", command)?;
        self.newline()?;
        self.flush()?;
        Ok(())
    }

    fn error<P, E>(
        &mut self,
        prefix: P,
        error: E,
    ) -> Result<(), Error>
    where
        P: fmt::Display,
        E: fmt::Display,
    {
        write!(self, "{}: {}\n", prefix, error)?;
        self.flush()?;
        Ok(())
    }

    fn command_output(&mut self, out: &Vec<u8>) -> Result<(), Error> {
        #[cfg(unix)]
        for i in str::from_utf8(out)?.lines() {
            self.write(i.as_bytes())?;
            self.newline()?;
        }

        #[cfg(windows)]
        self.write(out)?;

        self.flush()?;
        Ok(())
    }

    fn prompt(&mut self, state: &st::State) -> Result<(), Error> {
        write!(self, "{user}@{host}:{pwd} % ",
            user = state.user,
            host = state.host,
            pwd = state.pwd.display()
        )?;
        self.flush()?;
        Ok(())
    }

}

impl Term for Screen {
    fn cursor(&self) -> cursor::TerminalCursor {
        cursor(self)
    }
    fn terminal(&self) -> terminal::Terminal {
        terminal::terminal(self)
    }
}
