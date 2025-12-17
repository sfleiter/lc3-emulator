use crossterm::{ExecutableCommand, cursor, execute, terminal};
use std::io;
use std::io::Write;

use crate::emulator::stdout_helpers::CrosstermCompatibility;

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
pub fn set_terminal_raw(mut stdout: impl Write) -> RawLock {
    if let Err(e) =
        terminal::enable_raw_mode().and_then(|()| stdout.execute(terminal::EnableLineWrap))
    {
        handle_set_raw_error(&e);
    }
    RawLock {}
}

fn can_query_size_or_position(stdout: &(impl Write + CrosstermCompatibility)) -> bool {
    !(*stdout).will_block_on_size_or_position_queries()
}

pub fn print(stdout: &mut (impl Write + CrosstermCompatibility), data: &str) -> io::Result<()> {
    let (_column_count, row_count) = if can_query_size_or_position(stdout) {
        terminal::size()?
    } else {
        (1, 1)
    };
    let (_column, mut row) = if can_query_size_or_position(stdout) {
        cursor::position()?
    } else {
        (0, 0)
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
