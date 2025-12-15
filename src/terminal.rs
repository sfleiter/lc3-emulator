use crossterm::{ExecutableCommand, cursor, execute, terminal};
use std::io;
use std::io::{Write, stdout};
use std::os::fd::AsRawFd;

pub struct RawLock {}

impl Drop for RawLock {
    fn drop(&mut self) {
        // terminal stays in raw mode but no means to repair
        if let Err(e) = terminal::disable_raw_mode() {
            eprintln!("Error resetting terminal {e}");
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EchoOptions {
    EchoOn,
    EchoOff,
}

fn handle_set_raw_error(e: &io::Error) {
    eprintln!("Could not set terminal to raw mode: {e}");
}

/// Set terminal to raw in best-effort mode, only log on failure, since it does not work for
/// cargo doc tests and disabling does not work because of a
/// [rust issue](https://github.com/rust-lang/rust/issues/67295).
pub fn set_terminal_raw(_raw_fd_provider: &impl AsRawFd) -> RawLock {
    if let Err(e) = terminal::enable_raw_mode() {
        handle_set_raw_error(&e);
    }
    let mut stdout = stdout();
    stdout.execute(terminal::EnableLineWrap).unwrap();
    RawLock {}
}

const fn in_test() -> bool {
    #[cfg(test)]
    {
        true
    }
    #[cfg(not(test))]
    {
        false
    }
}

pub fn print(stdout: &mut impl Write, data: &str) -> io::Result<()> {
    let (_column_count, row_count) = terminal::size()?;
    let (_column, mut row) = if in_test() {
        (0, 0)
    } else {
        cursor::position()?
    };
    for (idx, part) in data.split('\n').enumerate() {
        row += 1;
        if idx > 0 {
            if row >= row_count {
                execute!(stdout, terminal::ScrollUp(1))?;
                execute!(stdout, cursor::MoveToColumn(0))?;
                row -= 1;
            } else {
                execute!(stdout, cursor::MoveToNextLine(1))?;
            }
            stdout.flush()?;
        }
        //stdout.write_all(format!("{row}/{row_count}: ").as_bytes())?;
        stdout.write_all(part.as_bytes())?;
        stdout.flush()?;
    }
    Ok(())
}
