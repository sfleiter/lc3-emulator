#[cfg(all(not(test), not(doctest)))]
use std::io;
#[cfg(all(not(test), not(doctest)))]
use termios::{ISIG, OPOST};

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

#[cfg(all(not(test), not(doctest)))]
fn handle_set_raw_error(e: &io::Error) {
    eprintln!("Could not set terminal to raw mode: {e}");
}

/// Set terminal to raw in best effort mode, only log on failure, since it does not work for
/// doctests and disabling does not work because of a
/// [rust issue](https://github.com/rust-lang/rust/issues/67295).
#[cfg(all(not(test), not(doctest)))]
fn set_terminal_raw_real(raw_fd_provider: &impl AsRawFd) -> RawLock {
    let fd = raw_fd_provider.as_raw_fd();
    let mut termios_orig = None;
    match Termios::from_fd(fd) {
        Ok(termios) => {
            termios_orig = Some(termios);
            let mut termios_raw = termios;
            // https://man7.org/linux/man-pages/man3/termios.3.html
            termios::cfmakeraw(&mut termios_raw);
            termios_raw.c_lflag |= ISIG;
            termios_raw.c_oflag |= OPOST;
            if let Err(e) = termios::tcsetattr(fd, termios::TCSAFLUSH, &termios_raw) {
                handle_set_raw_error(&e);
            }
        }
        Err(e) => handle_set_raw_error(&e),
    }
    RawLock { fd, termios_orig }
}

#[cfg(any(test, doctest))]
const fn set_terminal_raw_dummy(_raw_fd_provider: &impl AsRawFd) -> RawLock {
    RawLock {
        fd: -1,
        termios_orig: None,
    }
}

#[allow(
    clippy::missing_const_for_fn,
    reason = "in cfg(test) this looks like it could be const"
)]
pub fn set_terminal_raw(raw_fd_provider: &impl AsRawFd) -> RawLock {
    #[cfg(not(test))]
    return set_terminal_raw_real(raw_fd_provider);
    #[cfg(test)]
    return set_terminal_raw_dummy(raw_fd_provider);
}
