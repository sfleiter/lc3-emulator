//! This code does emulate the trap routines but does not implement them via the opcodes of the LC3
//! but directly.
//!
//! In the real system the code for these routines is at the target of the
//! [Trap Vector Tables](https://cs131.info/Assembly/Instructions/TRAPRoutines.html#trap-vector-table).
use crate::errors::ExecutionError;
use crate::hardware::memory::{Memory, MemoryMappedIOLocations};
use crate::hardware::registers::{Registers, from_binary};
use crate::terminal;
use crate::terminal::EchoOptions;
use std::io;
use std::io::Write;
use std::ops::ControlFlow;
use std::thread::sleep;
use std::time::Duration;

fn read_character_from_console(
    regs: &mut Registers,
    eo: EchoOptions,
    memory: &Memory,
    stdout: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    loop {
        if memory[MemoryMappedIOLocations::Kbsr as u16] != 0 {
            let c = memory[MemoryMappedIOLocations::Kbdr as u16];
            regs.set(0, from_binary(c));
            if eo == EchoOptions::EchoOn {
                #[allow(clippy::cast_possible_truncation)]
                {
                    let arr = &[c as u8];
                    let output = String::from_utf8_lossy(arr);
                    return write_str_out(output.as_ref(), stdout);
                }
            }
            return ControlFlow::Continue(());
        }
        sleep(Duration::from_millis(100));
    }
}

/// GETC: Read a single character from the keyboard. The character is not echoed onto the console.
///
/// Its ASCII code is copied into R0. The high eight bits of R0 are cleared.
pub fn get_c(
    regs: &mut Registers,
    memory: &Memory,
    stdout: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    read_character_from_console(regs, EchoOptions::EchoOff, memory, stdout)
}

/// IN: Print a prompt on the screen and read a single character echoed back from the keyboard.
///
/// Otherwise, like 0x20 GETC.
pub fn in_trap(
    regs: &mut Registers,
    memory: &Memory,
    stdout: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    write_str_out("Input: ", stdout)?;
    read_character_from_console(regs, EchoOptions::EchoOn, memory, stdout)
}

/// OUT: Write a character in R0\[7:0\] to the console display.
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
/// The ASCII code contained in bits \[7:0\] of a memory location is written to the console first.
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
    match terminal::print(stdout, message) {
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
    use crate::emulator::test_helpers::FakeEmulator;
    use googletest::prelude::*;

    fn check_register_value(regs: &Registers, idx: u8, expected: u16) {
        expect_that!(
            regs.get(idx).as_binary(),
            eq(expected),
            "{:?}",
            regs.get(idx)
        );
    }

    #[gtest]
    pub fn test_get_c() {
        let mut emu = FakeEmulator::new(&[0u16; 0], "a");
        let (regs, mem, mut writer) = emu.get_parts();
        let res = get_c(regs, mem, &mut writer);
        check_register_value(regs, 0, u16::from(b'a'));
        assert_that!(res, eq(&ControlFlow::Continue(())));
    }
    #[gtest]
    pub fn test_put_sp() {
        let data = [
            0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0x6548u16, 0x6c6c, 0x206f, 0x6f57, 0x6c72,
            0x2164, 0x0000,
        ];
        let mut emu = FakeEmulator::new(&data, "");
        let (regs, mem, writer) = emu.get_parts();
        regs.set(0, from_binary(0x3005));
        let res = put_sp(regs, mem, writer);
        assert!(res.is_continue());
        assert_that!(writer.get_string(), eq("Hello World!"));
    }
    #[gtest]
    pub fn test_in() {
        let mut emu = FakeEmulator::new(&[], "abc");
        let (regs, mem, writer) = emu.get_parts();

        let res = in_trap(regs, mem, writer);
        assert!(res.is_continue());
        check_register_value(regs, 0, u16::from(b'a'));

        let res = in_trap(regs, mem, writer);
        assert!(res.is_continue());
        check_register_value(regs, 0, u16::from(b'b'));

        let res = in_trap(regs, mem, writer);
        assert!(res.is_continue());
        check_register_value(regs, 0, u16::from(b'c'));

        expect_that!(writer.get_string(), eq("Input: aInput: bInput: c"));
    }

    #[gtest]
    pub fn test_out() {
        let mut emu = FakeEmulator::new(&[], "");
        let (regs, _mem, writer) = emu.get_parts();
        regs.set(0, from_binary(u16::from(b'k')));
        let res = out(regs, writer);
        assert!(res.is_continue());
        assert_that!(writer.get_string(), eq("k"));
    }
}
