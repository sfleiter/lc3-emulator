use crate::errors::ExecutionError;
use crate::hardware::memory::Memory;
use crate::hardware::registers::{Registers, from_binary};
use crate::terminal;
use crate::terminal::EchoOptions;
use std::io;
use std::io::Write;
use std::ops::ControlFlow;
use std::os::fd::AsRawFd;

fn read_character_from_console<R: io::Read + AsRawFd>(
    regs: &mut Registers,
    eo: EchoOptions,
    stdin: &mut R,
) -> ControlFlow<Result<(), ExecutionError>> {
    // Workaround for still unstable try blocks
    match (|| {
        let _lock = terminal::set_terminal_raw(stdin, eo)?;
        let mut b = [0; 1];
        stdin.read_exact(&mut b)?;
        regs.set(0, from_binary(u16::from(b[0])));
        Ok(())
    })() {
        Ok(()) => ControlFlow::Continue(()),
        Err(e) => wrap_io_error_in_cf(&e),
    }
}

/// GETC: Read a single character from the keyboard. The character is not echoed onto the console.
///
/// Its ASCII code is copied into R0. The high eight bits of R0 are cleared.
pub fn get_c<R: io::Read + AsRawFd>(
    regs: &mut Registers,
    stdin: &mut R,
) -> ControlFlow<Result<(), ExecutionError>> {
    read_character_from_console(regs, EchoOptions::EchoOff, stdin)
}

/// IN: Print a prompt on the screen and read a single character echoed back from the keyboard.
///
/// Otherwise, like 0x20 GETC.
pub fn in_trap<R: io::Read + AsRawFd>(
    regs: &mut Registers,
    stdin: &mut R,
    stdout: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    write_str_out("Input: ", stdout)?;
    read_character_from_console(regs, EchoOptions::EchoOn, stdin)
}

/// OUT: Write a character in R0[7:0] to the console display.
pub fn out(regs: &Registers, stdout: &mut impl Write) -> ControlFlow<Result<(), ExecutionError>> {
    let c: char = (regs.get(0).as_binary() & 0xFF) as u8 as char;
    write_str_out(&String::from(c), stdout)
}

fn put_one_char_per_u16(input: u16, append_to: &mut String) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Truncation is what is expected here"
    )]
    let c = (input as u8) as char;
    append_to.push(c);
}

fn put_two_chars_per_u16(input: u16, append_to: &mut String) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Truncation is what is expected here"
    )]
    let c = (input as u8) as char;
    append_to.push(c);
    let c = ((input >> 8) as u8) as char;
    if c != '\0' {
        append_to.push(c);
    }
}

fn put(
    regs: &Registers,
    mem: &Memory,
    stdout: &mut impl Write,
    handle_char: fn(u16, &mut String),
) -> ControlFlow<Result<(), ExecutionError>> {
    let address = regs.get(0).as_binary();
    let mut end = address;
    let mut s = String::with_capacity(120);
    while mem[end] != 0 {
        handle_char(mem[end], &mut s);
        end += 1;
    }
    write_str_out(s.as_str(), stdout)
}

/// PUTS: print null-delimited char* from register 0's address
pub fn put_s(
    regs: &Registers,
    mem: &Memory,
    stdout: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    put(regs, mem, stdout, put_one_char_per_u16)
}

/// PUTSP: Packed version of PUTS
///
/// The ASCII code contained in bits [7:0] of a memory location is written to the console first.
/// The second character of the last memory location can be 0x00.
/// Writing terminates with a 0x000 char.
pub fn put_sp(
    regs: &Registers,
    mem: &Memory,
    stdout: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    put(regs, mem, stdout, put_two_chars_per_u16)
}

/// HALT: End program and stdout a message
pub fn halt(stdout: &mut impl Write) -> ControlFlow<Result<(), ExecutionError>> {
    write_str_out("\nProgram halted\n", stdout)?;
    ControlFlow::Break(Ok(()))
}

fn write_str_out(
    message: &str,
    stdout: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    match write!(stdout, "{message}").and_then(|()| stdout.flush()) {
        Ok(()) => ControlFlow::Continue(()),
        Err(e) => wrap_io_error_in_cf(&e),
    }
}

fn wrap_io_error_in_cf(error: &io::Error) -> ControlFlow<Result<(), ExecutionError>, ()> {
    ControlFlow::Break(Err(ExecutionError::IOInputOutputError(error.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::test_helpers::{FakeEmulator, StringReader};
    use crate::hardware::registers::Register;
    use googletest::prelude::*;

    #[gtest]
    pub fn test_get_c() {
        let mut stdin = StringReader::from_bytes(b"a");
        let mut regs = Registers::new();
        let res = get_c(&mut regs, &mut stdin);
        assert_that!(res, eq(&ControlFlow::Continue(())));
    }
    #[gtest]
    pub fn test_get_c_read_error() {
        let mut stdin = StringReader::with_error(b"Error during read");
        let mut regs = Registers::new();
        let res = get_c(&mut regs, &mut stdin);
        assert!(res.is_break());
        let execution_error = res.break_value().unwrap().unwrap_err();
        assert_that!(
            execution_error.to_string(),
            eq("Error during reading Stdin or writing program output to Stdout: Error during read")
        );
    }
    #[gtest]
    pub fn test_put_sp() {
        let data = [
            0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0x6548u16, 0x6c6c, 0x206f, 0x6f57, 0x6c72,
            0x2164, 0x0000,
        ];
        let mut emu = FakeEmulator::new(&data);
        let (regs, mem, _reader, writer) = emu.get_parts();
        regs.set(0, from_binary(0x3005));
        let res = put_sp(regs, mem, writer);
        assert!(res.is_continue());
        assert_that!(writer.get_string(), eq("Hello World!"));
    }
    #[gtest]
    pub fn test_in() {
        let mut emu = FakeEmulator::new(&[]);
        emu.add_stdin_input(b"abc");
        let (regs, _mem, mut reader, writer) = emu.get_parts();
        let res = in_trap(regs, &mut reader, writer);
        assert!(res.is_continue());
        assert_that!(writer.get_string(), eq("Input: "));
        #[expect(clippy::cast_possible_truncation)]
        {
            assert_that!(regs.get(0).as_binary() as u8, eq(b'a'), "{:?}", regs.get(0));
        }
    }
    #[gtest]
    pub fn test_out() {
        let mut emu = FakeEmulator::new(&[]);
        let (regs, _mem, _reader, writer) = emu.get_parts();
        regs.set(0, Register::from_binary(u16::from(b'k')));
        let res = out(regs, writer);
        assert!(res.is_continue());
        assert_that!(writer.get_string(), eq("k"));
    }
}
