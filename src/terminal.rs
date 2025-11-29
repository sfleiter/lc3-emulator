use std::io::stdin;
use std::os::fd::{AsRawFd, RawFd};
use termios::{ECHO, Termios};

pub struct RawLock {
    fd: RawFd,
    termios_orig: Termios,
}

impl Drop for RawLock {
    fn drop(&mut self) {
        // terminal stays in raw mode but no means to repair
        let _ = termios::tcsetattr(self.fd, termios::TCSAFLUSH, &self.termios_orig);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EchoOptions {
    EchoOn,
    EchoOff,
}
pub fn set_terminal_raw(eo: EchoOptions) -> Result<RawLock, std::io::Error> {
    let fd = stdin().as_raw_fd();
    let termios_orig = termios::Termios::from_fd(fd)?;
    let mut termios_raw = termios_orig;
    // https://man7.org/linux/man-pages/man3/termios.3.html
    termios::cfmakeraw(&mut termios_raw);
    // c_lflag ECHO needed if we want to echo characters back after all
    if eo == EchoOptions::EchoOn {
        termios_raw.c_lflag |= ECHO;
    }
    termios::tcsetattr(fd, termios::TCSAFLUSH, &termios_raw)?;
    Ok(RawLock { fd, termios_orig })
}
