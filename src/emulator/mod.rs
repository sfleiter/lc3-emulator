mod instruction;
mod opcodes;

use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::{ProgramLoadedAtWrongAddress, ProgramMissingOrigHeader};
use crate::hardware::memory::{Memory, PROGRAM_SECTION_START};
use crate::hardware::registers::{Registers, from_binary};
use instruction::Instruction;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufReader, Read, Write, stdout};
use std::ops::ControlFlow;

const ORIG_HEADER: u16 = PROGRAM_SECTION_START;

#[rustfmt::skip]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
enum Operation {
    Add  = 0b0001,
    And  = 0b0101,
    Not  = 0b1001,
    Br   = 0b0000,
    JmpOrRet  = 0b1100,
    Jsr  = 0b0100,
    Ld   = 0b0010,
    Ldi  = 0b1010,
    Ldr  = 0b0110,
    Lea  = 0b1110,
    St   = 0b0011,
    Sti  = 0b1011,
    Str  = 0b0111,
    Trap = 0b1111,
    Rti  = 0b1000,
    _Reserved = 0b1101,
}

/// The public facing emulator used to run LC-3 programs.
pub struct Emulator {
    memory: Memory,
    registers: Registers,
}

fn from_program_byes(program: &[u16]) -> Result<Emulator, Lc3EmulatorError> {
    let Some((header, rest)) = program.split_at_checked(1) else {
        return Err(ProgramMissingOrigHeader);
    };
    if header[0] != ORIG_HEADER {
        let result = Err(ProgramLoadedAtWrongAddress {
            actual_address: header[0],
            expected_address: PROGRAM_SECTION_START,
        });
        return result;
    }
    let mut memory = Memory::new();
    memory.load_program(rest)?;
    Ok(Emulator {
        memory,
        registers: Registers::new(),
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
/// - See [`Lc3EmulatorError`]
/// - `Lc3EmulatorError::IoError` reading program object file
/// - Program is missing valid .ORIG header (because it is shorter than one `u16` instruction
/// - Program not loaded at byte offset `0x3000`
/// - Program too long
pub fn from_program(path: &str) -> Result<Emulator, Lc3EmulatorError> {
    let (file, file_size) =
        get_file_with_size(path).map_err(|e| map_err_program_not_loadable(path, e.to_string()))?;
    if file_size % 2 == 1 {
        return Err(Lc3EmulatorError::ProgramNotEvenSize(file_size));
    }
    let u16_file_size = usize::try_from(file_size / 2)
        .map_err(|_| Lc3EmulatorError::ProgramDoesNotFitIntoMemory(file_size))?;
    let mut file_data: Vec<u16> = Vec::with_capacity(u16_file_size);
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 2];
    let mut read_total = 0;
    while read_total < file_size {
        reader
            .read_exact(&mut buf)
            .map_err(|e| map_err_program_not_loadable(path, e.to_string()))?;
        file_data.push(switch_endian_bytes(buf[0], buf[1]));
        read_total += 2;
    }
    from_program_byes(file_data.as_slice())
}

fn map_err_program_not_loadable(path: &str, message: String) -> Lc3EmulatorError {
    Lc3EmulatorError::ProgramNotLoadable {
        file: path.to_owned(),
        message,
    }
}
fn get_file_with_size(path: &str) -> Result<(File, u64), std::io::Error> {
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
    /// - Program not loaded yet
    /// - Unknown instruction
    pub fn execute(&mut self) -> Result<(), Lc3EmulatorError> {
        let mut writer = stdout().lock();
        self.execute_with_writer(&mut writer)
    }

    fn execute_with_writer(&mut self, writer: &mut impl Write) -> Result<(), Lc3EmulatorError> {
        while self.registers.pc() < from_binary(self.memory.program_end()) {
            let data = self.memory[self.registers.pc().as_binary()];
            let i = Instruction::from(data);
            // println!("{i:?}");
            self.registers.inc_pc();
            if let Some(res) = self.execute_instruction(i, writer).break_value() {
                return res;
            }
        }
        writer.flush().expect("Could not flush output writer");
        Ok(())
    }

    #[expect(
        clippy::unnecessary_mut_passed,
        reason = "Needed for all opcodes thus if this fails this expect can be removed"
    )]
    fn execute_instruction(
        &mut self,
        instruction: Instruction,
        writer: &mut impl Write,
    ) -> ControlFlow<Result<(), Lc3EmulatorError>, ()> {
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
            o if o == Operation::Ldi as u8 => opcodes::ldi(instruction, &mut self.registers),
            o if o == Operation::Ldr as u8 => opcodes::ldr(instruction, &mut self.registers),
            o if o == Operation::Lea as u8 => opcodes::lea(instruction, &mut self.registers),
            o if o == Operation::St as u8 => opcodes::st(instruction, &mut self.registers),
            o if o == Operation::Sti as u8 => opcodes::sti(instruction, &mut self.registers),
            o if o == Operation::Str as u8 => opcodes::str(instruction, &mut self.registers),
            o if o == Operation::Trap as u8 => return self.trap(instruction, writer),
            o if o == Operation::Rti as u8 => opcodes::rti(instruction, &mut self.registers),
            o => return ControlFlow::Break(Err(Lc3EmulatorError::InvalidInstruction(o))),
        }
        ControlFlow::Continue(())
    }
    /// Handles Trap Routines
    ///
    /// # Panics
    /// - No access to memory, which can only happen when program is not loaded in which case
    ///   we should never run this method
    pub fn trap(
        &mut self,
        i: Instruction,
        mut writer: impl Write,
    ) -> ControlFlow<Result<(), Lc3EmulatorError>, ()> {
        // TODO test all implemented trap routines
        let trap_routine = i.get_bit_range(0, 7);
        match trap_routine {
            0x22 => {
                // PUTS: print null-delimited char* from register 0's address
                let address = self.registers.get(0).as_binary();
                let mut end = address;
                let mut s = String::with_capacity(120);
                while self.memory[end] != 0 {
                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "Truncation is what is expected here"
                    )]
                    let c = (self.memory[end] as u8) as char;
                    s.push(c);
                    end += 1;
                }
                match writeln!(writer, "{s}").and_then(|()| writer.flush()) {
                    Ok(()) => ControlFlow::Continue(()),
                    Err(e) => {
                        ControlFlow::Break(Err(Lc3EmulatorError::IOStdoutError(e.to_string())))
                    }
                }
            }
            0x25 => {
                // HALT
                println!("\nProgram halted");
                ControlFlow::Break(Ok(()))
            }
            _ => {
                eprintln!("Trap routine {trap_routine:#04X} not implemented yet");
                todo!()
            }
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

#[inline]
#[cfg(target_endian = "little")]
fn switch_endian_bytes(data0: u8, data1: u8) -> u16 {
    //eprintln!("input: 0x{data0:02X?}, 0x{data1:02X?}, result: 0x{res:04X?}");
    u16::from(data0) << 8 | u16::from(data1)
}
#[inline]
#[cfg(target_endian = "big")]
fn switch_endian_bytes(data0: u8, data1: u8) -> u16 {
    u16::from(data1) << 8 | u16::from(data0)
}

#[cfg(test)]
mod tests {
    use crate::emulator;
    use crate::emulator::{ORIG_HEADER, Operation};
    use crate::hardware::memory::PROGRAM_SECTION_MAX_INSTRUCTION_COUNT;
    use crate::hardware::registers::from_binary;
    use googletest::prelude::*;
    use std::io::Write;

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize =
        PROGRAM_SECTION_MAX_INSTRUCTION_COUNT as usize + 1;

    struct StringWriter {
        vec: Vec<u8>,
    }
    impl Write for StringWriter {
        fn write(&mut self, data: &[u8]) -> std::result::Result<usize, std::io::Error> {
            self.vec.write(data)
        }
        fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
            Ok(())
        }
    }
    impl StringWriter {
        fn new() -> Self {
            let vec = Vec::<u8>::with_capacity(120);
            Self { vec }
        }
        fn get_string(&self) -> String {
            String::from_utf8(self.vec.clone()).unwrap()
        }
    }

    #[gtest]
    pub fn test_load_program_missing_header() {
        assert_that!(
            emulator::from_program_byes(Vec::with_capacity(0).as_mut_slice())
                .unwrap_err()
                .to_string(),
            eq("Program is missing valid .ORIG header")
        );
    }
    #[gtest]
    pub fn test_load_program_wrong_header() {
        assert_that!(
            emulator::from_program_byes(vec![0x3001].as_mut_slice())
                .unwrap_err()
                .to_string(),
            eq("Program is not loaded at 0x3000' but 0x3001")
        );
    }
    #[gtest]
    pub fn test_load_program_disk_hello() {
        let mut sw = StringWriter::new();
        let mut emu = emulator::from_program("examples/hello_world.o").unwrap();
        {
            let mut ins = emu.instructions();
            assert_that!(ins.len(), eq(15));
            assert_that!(ins.next().unwrap().op_code(), eq(Operation::Lea as u8));
        }
        emu.execute_with_writer(&mut sw).unwrap();
        assert_that!(sw.get_string(), eq("HelloWorld!\n"));
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
    #[gtest]
    pub fn test_load_program_max_size() {
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = ORIG_HEADER;
        let emu = emulator::from_program_byes(program.as_mut_slice()).unwrap();
        let ins = emu.instructions();
        assert_that!(
            ins.len(),
            eq(usize::from(PROGRAM_SECTION_MAX_INSTRUCTION_COUNT))
        );
    }
    #[gtest]
    pub fn test_load_program_too_large() {
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1];
        program[0] = ORIG_HEADER;
        assert_that!(
            emulator::from_program_byes(program.as_mut_slice())
                .unwrap_err()
                .to_string(),
            eq("Program too long, got 52737 u16 instructions while limit is 52736")
        );
    }
}
