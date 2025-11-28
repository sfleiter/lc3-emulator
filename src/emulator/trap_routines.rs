use crate::errors::ExecutionError;
use crate::hardware::memory::Memory;
use crate::hardware::registers::{Registers, from_binary};
use crate::terminal;
use std::io;
use std::io::{Read, Write, stdin};
use std::ops::ControlFlow;

// TODO test all implemented trap routines

/// GETC: Read a single character from the keyboard. The character is not echoed onto the console.
///
/// Its ASCII code is copied into R0. The high eight bits of R0 are cleared.
pub fn get_c(regs: &mut Registers) -> ControlFlow<Result<(), ExecutionError>> {
    // Workaround for still unstable try blocks
    match (|| {
        let _lock = terminal::set_terminal_raw()?;
        let mut b = [0; 1];
        stdin().read_exact(&mut b)?;
        regs.set(0, from_binary(u16::from(b[0])));
        io::Result::Ok(())
    })() {
        Ok(()) => ControlFlow::Continue(()),
        Err(e) => wrap_io_error_in_cf(&e),
    }
}

/// OUT: Write a character in R0[7:0] to the console display.
pub fn out(regs: &Registers, write: &mut impl Write) -> ControlFlow<Result<(), ExecutionError>> {
    let c: char = (regs.get(0).as_binary() & 0xFF) as u8 as char;
    match write!(write, "{c}").and_then(|()| write.flush()) {
        Ok(()) => ControlFlow::Continue(()),
        Err(e) => wrap_io_error_in_cf(&e),
    }
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
    write: &mut impl Write,
    handle_char: fn(u16, &mut String),
) -> ControlFlow<Result<(), ExecutionError>> {
    let address = regs.get(0).as_binary();
    let mut end = address;
    let mut s = String::with_capacity(120);
    while mem[end] != 0 {
        handle_char(mem[end], &mut s);
        end += 1;
    }
    match writeln!(write, "{s}").and_then(|()| write.flush()) {
        Ok(()) => ControlFlow::Continue(()),
        Err(e) => wrap_io_error_in_cf(&e),
    }
}
/// PUTS: print null-delimited char* from register 0's address
pub fn put_s(
    regs: &Registers,
    mem: &Memory,
    write: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    put(regs, mem, write, put_one_char_per_u16)
}
/// PUTSP: Packed version of PUTS
///
/// The ASCII code contained in bits [7:0] of a memory location
/// is written to the console first. The second character of the last memory location
/// can be 0x00.
/// Writing terminates with a 0x000 char
pub fn put_sp(
    regs: &Registers,
    mem: &Memory,
    write: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    put(regs, mem, write, put_two_chars_per_u16)
}

/// IN: Print a prompt on the screen and read a single echoed character from the keyboard.
///
/// Otherwise, like 0x20 GETC.
pub fn in_trap(
    _regs: &mut Registers,
    _write: &mut impl Write,
) -> ControlFlow<Result<(), ExecutionError>> {
    todo!()
}

/// HALT: End program and write a message
pub fn halt(write: &mut impl Write) -> ControlFlow<Result<(), ExecutionError>> {
    match writeln!(write, "\nProgram halted").and_then(|()| write.flush()) {
        Ok(()) => ControlFlow::Break(Ok(())),
        Err(e) => wrap_io_error_in_cf(&e),
    }
}

fn wrap_io_error_in_cf(error: &impl ToString) -> ControlFlow<Result<(), ExecutionError>, ()> {
    ControlFlow::Break(Err(ExecutionError::IOStdoutError(error.to_string())))
}
