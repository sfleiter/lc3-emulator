use std::fmt::Arguments;
use std::io::{IoSlice, Stdout, Write, stdout};

pub trait CrosstermCompatibility {
    fn will_block_on_size_or_position_queries(&self) -> bool;
}
impl CrosstermCompatibility for Stdout {
    fn will_block_on_size_or_position_queries(&self) -> bool {
        #[cfg(not(test))]
        return false;
        #[cfg(test)]
        return true;
    }
}

pub struct StdoutForDocTest(Stdout);
impl Default for StdoutForDocTest {
    fn default() -> Self {
        Self::new()
    }
}
impl StdoutForDocTest {
    #[must_use]
    pub fn new() -> Self {
        Self(stdout())
    }
}
impl CrosstermCompatibility for StdoutForDocTest {
    fn will_block_on_size_or_position_queries(&self) -> bool {
        true
    }
}

impl Write for StdoutForDocTest {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        self.0.write_vectored(bufs)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.write_all(buf)
    }
    fn write_fmt(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
        self.0.write_fmt(args)
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}
