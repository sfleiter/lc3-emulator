use std::io::stdin;
use std::os::fd::{AsRawFd, RawFd};
use termios::Termios;

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

pub fn set_terminal_raw() -> Result<RawLock, std::io::Error> {
    let fd = stdin().as_raw_fd();
    let termios_orig = termios::Termios::from_fd(fd).unwrap();
    let mut termios_raw = termios_orig;
    // https://man7.org/linux/man-pages/man3/termios.3.html
    termios::cfmakeraw(&mut termios_raw);
    termios::tcsetattr(fd, termios::TCSAFLUSH, &termios_raw)?;
    Ok(RawLock { fd, termios_orig })
}
