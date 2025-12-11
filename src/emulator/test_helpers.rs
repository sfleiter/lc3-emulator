use crate::emulator;
use crate::emulator::Emulator;
use crate::hardware::memory::Memory;
use crate::hardware::registers::Registers;
use std::io;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::SendError;

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

        let emu = emulator::from_program_bytes(program.as_slice()).unwrap();
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
    pub fn get_parts(
        &'a mut self,
    ) -> Result<(&'a mut Registers, &'a mut Memory, &'a mut StringWriter), SendError<u16>> {
        let (sender, receiver) = mpsc::channel();
        self.inner.memory.set_keyboard_input_receiver(receiver);
        for b in self.stdin_data {
            sender.send(u16::from(*b))?;
        }
        Ok((
            &mut self.inner.registers,
            &mut self.inner.memory,
            &mut self.stdout,
        ))
    }
}
