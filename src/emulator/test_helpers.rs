use crate::emulator;
use crate::emulator::Emulator;
use crate::emulator::stdout_helpers::CrosstermCompatibility;
use crate::hardware::keyboard::KeyboardInputProvider;
use crate::hardware::memory::Memory;
use crate::hardware::registers::Registers;
use std::io;
use std::io::Write;

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
impl CrosstermCompatibility for StringWriter {
    fn will_block_on_size_or_position_queries(&self) -> bool {
        true
    }
}

pub struct FakeKeyboardInputProvider {
    input_data: String,
    index: usize,
}
impl FakeKeyboardInputProvider {
    pub fn new(input: &str) -> Self {
        Self {
            input_data: input.into(),
            index: 0,
        }
    }
}
impl KeyboardInputProvider for FakeKeyboardInputProvider {
    fn check_input_available(&mut self) -> io::Result<bool> {
        if self.index >= self.input_data.len() {
            Ok(false)
        } else {
            Ok(true)
        }
    }
    fn get_input_character(&mut self) -> char {
        if self.check_input_available().unwrap() {
            let res = self.input_data.as_bytes()[self.index];
            self.index += 1;
            res as char
        } else {
            panic!("No input available");
        }
    }
    fn is_interrupted(&self) -> bool {
        false
    }
}

pub struct FakeEmulator {
    inner: Emulator,
    stdout: StringWriter,
}
impl FakeEmulator {
    pub fn new(program_no_header: &[u16], input: &str) -> Self {
        let mut program = Vec::with_capacity(program_no_header.len() + 1);
        program.push(0x3000u16);
        if program_no_header.is_empty() {
            program.push(0);
        } else {
            program.extend_from_slice(program_no_header);
        }
        let keyboard_input_provider = FakeKeyboardInputProvider::new(input);
        let emu = emulator::from_program_bytes_with_kbd_input_provider(
            program.as_slice(),
            keyboard_input_provider,
        )
        .unwrap();
        let sw = StringWriter::new();
        Self {
            inner: emu,
            stdout: sw,
        }
    }
    pub fn get_parts(&mut self) -> (&mut Registers, &mut Memory, &mut StringWriter) {
        (
            &mut self.inner.registers,
            &mut self.inner.memory,
            &mut self.stdout,
        )
    }
}
