use std::os::fd::{AsRawFd, RawFd};
use termios::Termios;

pub struct RawLock {
    fd: RawFd,
    termios_orig: Option<Termios>,
}

impl Drop for RawLock {
    fn drop(&mut self) {
        // terminal stays in raw mode but no means to repair
        if let Some(termios) = self.termios_orig {
            let _ = termios::tcsetattr(self.fd, termios::TCSAFLUSH, &termios);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EchoOptions {
    EchoOn,
    EchoOff,
}

#[cfg(not(test))]
fn set_terminal_raw_prod(raw_fd_provider: &impl AsRawFd) -> Result<RawLock, std::io::Error> {
    let fd = raw_fd_provider.as_raw_fd();
    let termios_orig = Termios::from_fd(fd)?;
    let mut termios_raw = termios_orig;
    // https://man7.org/linux/man-pages/man3/termios.3.html
    termios::cfmakeraw(&mut termios_raw);
    termios::tcsetattr(fd, termios::TCSAFLUSH, &termios_raw)?;
    Ok(RawLock {
        fd,
        termios_orig: Some(termios_orig),
    })
}

#[cfg(test)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "the prod variant needs this and we need tp provide the same API"
)]
const fn set_terminal_raw_test(_raw_fd_provider: &impl AsRawFd) -> Result<RawLock, std::io::Error> {
    Ok(RawLock {
        fd: -1,
        termios_orig: None,
    })
}

#[allow(
    clippy::missing_const_for_fn,
    reason = "in cfg(test) this looks like it could be const"
)]
pub fn set_terminal_raw(raw_fd_provider: &impl AsRawFd) -> Result<RawLock, std::io::Error> {
    #[cfg(not(test))]
    return set_terminal_raw_prod(raw_fd_provider);
    #[cfg(test)]
    return set_terminal_raw_test(raw_fd_provider);
}
