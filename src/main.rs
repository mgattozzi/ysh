use std::{
    str,
    io::{
        stdin,
        StdinLock,
        stdout,
        StdoutLock,
        Write
    },
    process::{
        exit,
        Command,
    }
};
use termion::{
    raw::{
        RawTerminal,
        IntoRawMode
    },
    input::TermRead,
    event::Key,
};
use failure::{
    Error,
};

mod terminal;
mod st;


fn main() {
    let stdout = stdout();
    let stdin = stdin();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = stdin.lock();

    st::init(&mut stdout).unwrap_or_else(|e| { eprintln!("{}", e); exit(1); });

    run(&mut stdin, &mut stdout).unwrap_or_else(|e| eprintln!("{}", e));
}


fn run(stdin: &mut StdinLock, stdout: &mut RawTerminal<StdoutLock>) -> Result<(), Error> {
    let mut line = Vec::new();
    for c in stdin.keys() {
        match c? {
            Key::Char(c) => {
                if c != '\n' {
                    line.push(c as u8);
                    write!(stdout, "{}", c)?;
                } else /* execute the command */ {
                    match str::from_utf8(&line)? {
                        "clear" => terminal::reset(stdout)?,
                        command => {
                            terminal::newline(stdout)?;
                            // This will mess up when doing something like:
                            // echo "hi hello"
                            // which will be [echo, "hi, hello"] so need to make a
                            // parse_command function that can handle this stuff better
                            let args = command.split_whitespace().collect::<Vec<&str>>();
                            if args.len() == 0 { stdout.flush()?; continue; }
                            Command::new(args[0])
                                .args(&args[1..])
                                .output()
                                .map_err(|e| e.into())
                                .and_then(|exec| terminal::command_output(stdout, &exec.stdout))
                                .or_else(|_| terminal::not_found(stdout, command))?;
                            terminal::prompt(stdout)?;
                        }
                    }
                    line.clear();
                }
            },
            Key::Backspace => {
                if line.len() > 0 {
                    line.pop();
                    terminal::backspace(stdout)?;
                }
            },
            Key::Esc => break,
            _ => {}
        }
        stdout.flush()?;
    }
    Ok(())
}
