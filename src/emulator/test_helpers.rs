use crate::emulator;
use crate::emulator::Emulator;
use crate::hardware::memory::Memory;
use crate::hardware::registers::Registers;
use std::io;
use std::io::{Cursor, Read, Write};
use std::os::fd::AsRawFd;

pub struct StringReader<'a> {
    bytes: &'a [u8],
    cursor: Cursor<&'a [u8]>,
    is_error: bool,
}
impl<'a> StringReader<'a> {
    pub const fn from_bytes(data: &'a [u8]) -> Self {
        Self {
            bytes: data,
            cursor: Cursor::new(data),
            is_error: false,
        }
    }
    pub const fn with_error(error_message: &'a [u8]) -> Self {
        Self {
            bytes: error_message,
            cursor: Cursor::new(b""),
            is_error: true,
        }
    }
}
impl Read for StringReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.is_error {
            Err(io::Error::other(String::from_utf8_lossy(self.bytes)))
        } else {
            self.cursor.read(buf)
        }
    }
}
impl AsRawFd for StringReader<'_> {
    fn as_raw_fd(&self) -> i32 {
        // In test code the result is not used, so this is okay
        -1
    }
}

pub struct StringWriter {
    vec: Vec<u8>,
}
impl Write for StringWriter {
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        self.vec.write(data)
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}
impl StringWriter {
    pub fn new() -> Self {
        let vec = Vec::<u8>::with_capacity(120);
        Self { vec }
    }
    pub fn get_string(&self) -> String {
        String::from_utf8(self.vec.clone()).unwrap()
    }
}

pub struct FakeEmulator<'a> {
    inner: Emulator,
    stdin_data: &'a [u8],
    stdout: StringWriter,
}
impl<'a> FakeEmulator<'a> {
    pub fn new(program_no_header: &[u16]) -> Self {
        let mut program = Vec::with_capacity(program_no_header.len() + 1);
        program.push(0x3000u16);
        if program_no_header.is_empty() {
            program.push(0);
        } else {
            program.extend_from_slice(program_no_header);
        }

        let emu = emulator::from_program_byes(program.as_slice()).unwrap();
        let sw = StringWriter::new();
        Self {
            inner: emu,
            stdin_data: b"",
            stdout: sw,
        }
    }
    pub fn add_stdin_input(&'_ mut self, input: &'a [u8]) -> &mut Self {
        self.stdin_data = input;
        self
    }
    // TODO check for existing bug on that wrong lifetime assumption
    #[expect(
        mismatched_lifetime_syntaxes,
        reason = "wrong same lifetime assumption for StringReader"
    )]
    pub fn get_parts(
        &'a mut self,
    ) -> (
        &'a mut Registers,
        &'a mut Memory,
        StringReader,
        &'a mut StringWriter,
    ) {
        let sr = StringReader::from_bytes(self.stdin_data);
        (
            &mut self.inner.registers,
            &mut self.inner.memory,
            sr,
            &mut self.stdout,
        )
    }
}
