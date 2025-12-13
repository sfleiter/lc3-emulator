mod instruction;
mod opcodes;
#[cfg(test)]
mod test_helpers;
mod trap_routines;

use crate::errors::{ExecutionError, LoadProgramError};
use crate::hardware::keyboard;
use crate::hardware::memory::{Memory, PROGRAM_SECTION_START};
use crate::hardware::registers::{Registers, from_binary};
use crate::terminal;
use instruction::Instruction;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Write};
use std::ops::ControlFlow;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

const ORIG_HEADER: u16 = PROGRAM_SECTION_START;

#[rustfmt::skip]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
enum Operation {
    Br   = 0b0000,
    Add  = 0b0001,
    Ld   = 0b0010,
    St   = 0b0011,
    Jsr  = 0b0100,
    And  = 0b0101,
    Ldr  = 0b0110,
    Str  = 0b0111,
    Rti  = 0b1000,
    Not  = 0b1001,
    Ldi  = 0b1010,
    Sti  = 0b1011,
    JmpOrRet  = 0b1100,
    _Reserved = 0b1101,
    Lea  = 0b1110,
    Trap = 0b1111,
}

/// The public facing emulator used to run LC-3 programs.
pub struct Emulator {
    memory: Memory,
    registers: Registers,
    keyboard_poller: Option<JoinHandle<()>>,
}

pub(crate) fn from_program_bytes(data: &[u16]) -> Result<Emulator, LoadProgramError> {
    let (sender, receiver) = mpsc::channel();
    let mut res = from_program_bytes_with_kbd_input_receiver(data, receiver)?;
    res.keyboard_poller = Some(keyboard::create_keyboard_poller(sender));
    Ok(res)
}

pub(crate) fn from_program_bytes_with_kbd_input_receiver(
    data: &[u16],
    kbd_input_receiver: Receiver<u16>,
) -> Result<Emulator, LoadProgramError> {
    let [header, program @ ..] = data else {
        return Err(LoadProgramError::ProgramMissingOrigHeader);
    };
    if *header != ORIG_HEADER {
        return Err(LoadProgramError::ProgramLoadedAtWrongAddress {
            actual_address: *header,
            expected_address: PROGRAM_SECTION_START,
        });
    }
    if program.is_empty() {
        return Err(LoadProgramError::ProgramEmpty);
    }
    let mut memory = Memory::new(kbd_input_receiver);
    memory.load_program(program)?;
    Ok(Emulator {
        memory,
        registers: Registers::new(),
        keyboard_poller: None,
    })
}

/// Loads a program from disk into the memory section starting from
/// address `_PROGRAM_SECTION_START_BYTES`
/// and returns an iterator over the loaded instructions.
///
/// # Parameters
/// - `path` defines the location of the LC-3 object file to execute
///
/// #  Errors
/// - See [`LoadProgramError`]
pub fn from_program(path: &str) -> Result<Emulator, LoadProgramError> {
    let (file, file_size) =
        get_file_with_size(path).map_err(|e| map_err_program_not_loadable(path, e.to_string()))?;
    if file_size % 2 == 1 {
        return Err(LoadProgramError::ProgramNotEvenSize(file_size));
    }
    let u16_file_size = usize::try_from(file_size / 2)
        .map_err(|_| LoadProgramError::ProgramDoesNotFitIntoMemory(file_size))?;
    let mut file_data: Vec<u16> = Vec::with_capacity(u16_file_size);
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 2];
    let mut read_total = 0;
    while read_total < file_size {
        reader
            .read_exact(&mut buf)
            .map_err(|e| map_err_program_not_loadable(path, e.to_string()))?;
        file_data.push((u16::from(buf[0]) << 8) | u16::from(buf[1]));
        read_total += 2;
    }
    from_program_bytes(file_data.as_slice())
}

fn map_err_program_not_loadable(path: &str, message: String) -> LoadProgramError {
    LoadProgramError::ProgramNotLoadable {
        file: path.to_owned(),
        message,
    }
}
fn get_file_with_size(path: &str) -> Result<(File, u64), io::Error> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    Ok((file, file_size))
}

impl Emulator {
    /// Return instructions parsed from loaded program.
    #[must_use]
    pub fn instructions(&self) -> impl ExactSizeIterator<Item = Instruction> + Debug {
        self.memory
            .program_slice()
            .iter()
            .map(|bits| Instruction::from(*bits))
    }

    /// Executes the loaded program.
    /// # Errors
    /// - See [`ExecutionError`]
    pub fn execute(&mut self) -> Result<(), ExecutionError> {
        if let Some(join_handle) = self.keyboard_poller.as_ref()
            && join_handle.is_finished()
        {
            return Err(ExecutionError::IOInputOutputError(String::from(
                "Error in keyboard polling thread caused its halt",
            )));
        }
        let mut stdout = io::stdout().lock();
        let _lock = terminal::set_terminal_raw(&io::stdin());
        self.execute_with_stdout(&mut stdout)
    }

    fn execute_with_stdout(&mut self, stdout: &mut impl Write) -> Result<(), ExecutionError> {
        while self.registers.pc() < from_binary(self.memory.program_end()) {
            let data = self.memory[self.registers.pc().as_binary()];
            let i = Instruction::from(data);
            // println!("{i:?}");
            self.registers.inc_pc();
            if let Some(res) = self.execute_instruction(i, stdout).break_value() {
                return res;
            }
        }
        stdout.flush().map_err(|e| {
            ExecutionError::IOInputOutputError(format!("Error flushing stdout: {e}"))
        })?;
        Ok(())
    }

    #[expect(
        clippy::unnecessary_mut_passed,
        reason = "Needed for all opcodes thus if this fails this expect can be removed"
    )]
    fn execute_instruction(
        &mut self,
        instruction: Instruction,
        stdout: &mut impl Write,
    ) -> ControlFlow<Result<(), ExecutionError>, ()> {
        match instruction.op_code() {
            o if o == Operation::Add as u8 => opcodes::add(instruction, &mut self.registers),
            o if o == Operation::And as u8 => opcodes::and(instruction, &mut self.registers),
            o if o == Operation::Not as u8 => opcodes::not(instruction, &mut self.registers),
            o if o == Operation::Br as u8 => opcodes::br(instruction, &mut self.registers),
            o if o == Operation::JmpOrRet as u8 => {
                opcodes::jmp_or_ret(instruction, &mut self.registers);
            }
            o if o == Operation::Jsr as u8 => opcodes::jsr(instruction, &mut self.registers),
            o if o == Operation::Ld as u8 => {
                opcodes::ld(instruction, &mut self.registers, &self.memory);
            }
            o if o == Operation::Ldi as u8 => {
                opcodes::ldi(instruction, &mut self.registers, &mut self.memory);
            }
            o if o == Operation::Ldr as u8 => {
                opcodes::ldr(instruction, &mut self.registers, &mut self.memory);
            }
            o if o == Operation::Lea as u8 => opcodes::lea(instruction, &mut self.registers),
            o if o == Operation::St as u8 => {
                opcodes::st(instruction, &self.registers, &mut self.memory);
            }
            o if o == Operation::Sti as u8 => {
                opcodes::sti(instruction, &self.registers, &mut self.memory);
            }
            o if o == Operation::Str as u8 => {
                opcodes::str(instruction, &self.registers, &mut self.memory);
            }
            o if o == Operation::Trap as u8 => return self.trap(instruction, stdout),
            o if o == Operation::Rti as u8 => opcodes::rti(instruction, &mut self.registers),
            o if o == Operation::_Reserved as u8 => {
                return ControlFlow::Break(Err(ExecutionError::ReservedInstructionFound(o)));
            }
            _ => unreachable!("All variants of 4 bit opcodes checked"),
        }
        ControlFlow::Continue(())
    }

    /// Handles Trap Routines.
    ///
    /// # Result
    /// - [`ControlFlow::Continue`] when the program should continue as normal
    /// - [`ControlFlow::Break`] with a [`Result`] when the program should end
    ///
    /// # Errors
    /// - see [`ExecutionError`]
    pub fn trap(
        &mut self,
        i: Instruction,
        mut stdout: impl Write,
    ) -> ControlFlow<Result<(), ExecutionError>, ()> {
        let trap_routine = i.get_bit_range(0, 7);
        match trap_routine {
            0x20 => trap_routines::get_c(&mut self.registers, &self.memory, &mut stdout),
            0x21 => trap_routines::out(&self.registers, &mut stdout),
            0x22 => trap_routines::put_s(&self.registers, &self.memory, &mut stdout),
            0x23 => trap_routines::in_trap(&mut self.registers, &self.memory, &mut stdout),
            0x24 => trap_routines::put_sp(&self.registers, &self.memory, &mut stdout),
            0x25 => trap_routines::halt(&mut stdout),
            tr => ControlFlow::Break(Err(ExecutionError::UnknownTrapRoutine(tr))),
        }
    }
}

impl Debug for Emulator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Emulator:")?;
        writeln!(f, "{:?}", self.memory)?;
        writeln!(f, "Registers:\n{:?}", self.registers)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator;
    use crate::emulator::test_helpers::StringWriter;
    use crate::emulator::{Emulator, ORIG_HEADER, Operation};
    use crate::errors::LoadProgramError;
    use crate::errors::LoadProgramError::*;
    use crate::hardware::memory::PROGRAM_SECTION_MAX_INSTRUCTION_COUNT;
    use crate::hardware::registers::from_binary;
    use googletest::prelude::*;
    use std::error::Error;
    use std::sync::mpsc;
    use yare::parameterized;

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize =
        PROGRAM_SECTION_MAX_INSTRUCTION_COUNT as usize + 1;

    fn emu_with_program_from_vec_wo_kdb(
        data: &Vec<u16>,
    ) -> std::result::Result<Emulator, LoadProgramError> {
        let (_sender, receiver) = mpsc::channel();
        emulator::from_program_bytes_with_kbd_input_receiver(data.as_slice(), receiver)
    }

    #[parameterized(
        missing_header = {Vec::with_capacity(0), ProgramMissingOrigHeader },
        wrong_header = {vec![0x3001], ProgramLoadedAtWrongAddress
            {actual_address: 0x3001, expected_address: 0x3000 } },
        too_large = {vec![0x3000u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1],
            ProgramTooLong {actual_instructions: 52737,
            maximum_instructions: PROGRAM_SECTION_MAX_INSTRUCTION_COUNT} },
        empty = { vec![0x3000u16; 1], ProgramEmpty }
    )]
    #[test_macro(gtest)]
    pub fn test_load_program_errors(data: Vec<u16>, error: LoadProgramError) {
        let abstract_error =
            Box::<dyn Error>::from(emu_with_program_from_vec_wo_kdb(&data).unwrap_err());
        let res = abstract_error.downcast_ref::<LoadProgramError>();
        assert_that!(res.unwrap(), eq(&error));
    }

    #[gtest]
    pub fn test_load_program_max_size() {
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = ORIG_HEADER;
        let emu = emu_with_program_from_vec_wo_kdb(&program).unwrap();
        let ins = emu.instructions();
        assert_that!(
            ins.len(),
            eq(usize::from(PROGRAM_SECTION_MAX_INSTRUCTION_COUNT))
        );
    }
    #[gtest]
    pub fn test_load_program_disk_hello() {
        let mut sw = StringWriter::new();
        let mut emu = emulator::from_program("examples/hello_world_puts.o").unwrap();
        {
            let mut ins = emu.instructions();
            assert_that!(ins.len(), eq(15));
            assert_that!(ins.next().unwrap().op_code(), eq(Operation::Lea as u8));
        }
        emu.execute_with_stdout(&mut sw).unwrap();
        assert_that!(sw.get_string(), eq("HelloWorld!\nProgram halted\n"));
        // TODO add more assertions for further content
    }
    #[gtest]
    pub fn test_program_add_ld_break_times_ten() {
        let mut emu = emulator::from_program("examples/times_ten.o").unwrap();
        emu.execute().unwrap();
        assert_that!(emu.registers.get(2), eq(from_binary(0)));
        assert_that!(emu.registers.get(3), eq(from_binary(30)));
        // TODO add more assertions for further content
    }
}
