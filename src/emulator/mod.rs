mod instruction;

use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::{ProgramLoadedAtWrongAddress, ProgramMissingOrigHeader};
use crate::hardware::memory::{Memory, PROGRAM_SECTION_START};
use crate::hardware::registers::Registers;
use instruction::Instruction;
use std::cmp::PartialEq;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufReader, Read};

const ORIG_HEADER: u16 = PROGRAM_SECTION_START;

#[rustfmt::skip]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum Operation {
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
    Reserved = 0b1101,
}

#[derive(Debug, PartialEq, Eq)]
enum EmulatorState {
    Start,
    Loaded,
    Executed,
}
/// The public facing emulator used to run LC-3 programs.
pub struct Emulator {
    memory: Memory,
    registers: Registers,
    state: EmulatorState,
}
impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
impl Emulator {
    /// Constructor method
    #[must_use]
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            registers: Registers::new(),
            state: EmulatorState::Start,
        }
    }

    fn enforce_state(&self, state: &EmulatorState) -> Result<(), Lc3EmulatorError> {
        if *state == self.state {
            Ok(())
        } else {
            Err(Lc3EmulatorError::WrongState)
        }
    }

    fn load_program_from_memory(&mut self, program: &[u16]) -> Result<(), Lc3EmulatorError> {
        self.enforce_state(&EmulatorState::Start)?;
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
        let res = self.memory.load_program(rest);
        self.state = EmulatorState::Loaded;
        res
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
    pub fn load_program(&mut self, path: &str) -> Result<(), Lc3EmulatorError> {
        self.enforce_state(&EmulatorState::Start)?;
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();
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
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    file_data.push(switch_endian_bytes(buf[0], buf[1]));
                    read_total += 2;
                }
                Err(e) => return Err(Lc3EmulatorError::IoError(e)),
            }
        }
        self.load_program_from_memory(file_data.as_slice())
    }

    /// Return instructions parsed from loaded program.
    /// # Errors
    /// - Program not loaded yet
    pub fn instructions(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = Instruction> + Debug, Lc3EmulatorError> {
        self.enforce_state(&EmulatorState::Loaded)?;
        Ok(self
            .memory
            .program_slice()?
            .iter()
            .map(|bits| Instruction::from(*bits))
            .collect::<Vec<Instruction>>()
            .into_iter())
    }

    /// Executes the loaded program.
    /// # Errors
    /// - Program not loaded yet
    /// - Unknown instruction
    pub fn execute(&mut self) -> Result<(), Lc3EmulatorError> {
        self.enforce_state(&EmulatorState::Loaded)?;
        self.state = EmulatorState::Executed;
        while self.registers.pc() < self.memory.program_end() {
            let data = self.memory.memory()?[usize::from(self.registers.pc().as_u16())];
            let i = Instruction::from(data);
            println!("{i:?}");
            self.execute_instruction(i)?;
            self.registers.inc_pc();
        }
        Ok(())
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), Lc3EmulatorError> {
        match instruction.op_code() {
            o if o == Operation::Add as u8 => self.add(instruction),
            o if o == Operation::And as u8 => self.and(instruction),
            o if o == Operation::Not as u8 => self.not(instruction),
            o if o == Operation::Br as u8 => self.br(instruction),
            o if o == Operation::JmpOrRet as u8 => self.jmp_or_ret(instruction),
            o if o == Operation::Jsr as u8 => self.jsr(instruction),
            o if o == Operation::Ld as u8 => self.ld(instruction),
            o if o == Operation::Ldi as u8 => self.ldi(instruction),
            o if o == Operation::Ldr as u8 => self.ldr(instruction),
            o if o == Operation::Lea as u8 => self.lea(instruction),
            o if o == Operation::St as u8 => self.st(instruction),
            o if o == Operation::Sti as u8 => self.sti(instruction),
            o if o == Operation::Str as u8 => self.str(instruction),
            o if o == Operation::Trap as u8 => self.trap(instruction),
            o if o == Operation::Rti as u8 => self.rti(instruction),
            o => return Err(Lc3EmulatorError::InvalidInstruction(o)),
        }
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn add(&mut self, i: Instruction) {
        self.registers.set(
            i.dr_number(),
            (self.registers.get(i.sr1_number()).as_u32()
                + (if i.is_immediate() {
                    u32::from(i.get_immediate())
                } else {
                    self.registers.get(i.sr2_number()).as_u32()
                })) as u16,
        );
        self.registers.update_conditional_register(i.dr_number());
    }
    fn and(&mut self, i: Instruction) {
        self.registers.set(
            i.dr_number(),
            self.registers.get(i.sr1_number()).as_u16()
                & (if i.is_immediate() {
                    i.get_immediate()
                } else {
                    self.registers.get(i.sr2_number()).as_u16()
                }),
        );
        self.registers.update_conditional_register(i.dr_number());
    }
    fn not(&mut self, i: Instruction) {
        self.registers
            .set(i.dr_number(), !self.registers.get(i.sr1_number()).as_u16());
        self.registers.update_conditional_register(i.dr_number());
    }
    fn br(&self, _i: Instruction) {
        unimplemented!()
    }
    fn jmp_or_ret(&self, _i: Instruction) {
        unimplemented!()
    }
    fn jsr(&self, _i: Instruction) {
        unimplemented!()
    }
    fn ld(&self, _i: Instruction) {
        unimplemented!()
    }
    fn ldi(&self, _i: Instruction) {
        unimplemented!()
    }
    fn ldr(&self, _i: Instruction) {
        unimplemented!()
    }
    fn lea(&self, _i: Instruction) {
        unimplemented!()
    }
    fn st(&self, _i: Instruction) {
        unimplemented!()
    }
    fn sti(&self, _i: Instruction) {
        unimplemented!()
    }
    fn str(&self, _i: Instruction) {
        unimplemented!()
    }
    fn trap(&self, _i: Instruction) {
        unimplemented!()
    }
    fn rti(&self, _i: Instruction) {
        unimplemented!()
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
    use crate::emulator::{Emulator, ORIG_HEADER, Operation};
    use crate::hardware::memory::PROGRAM_SECTION_MAX_INSTRUCTION_COUNT;
    use crate::hardware::registers::ConditionFlag;
    use googletest::prelude::*;

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize =
        PROGRAM_SECTION_MAX_INSTRUCTION_COUNT as usize + 1;
    #[gtest]
    pub fn test_load_program_missing_header() {
        let mut emu = Emulator::new();
        assert_that!(
            emu.load_program_from_memory(Vec::with_capacity(0).as_mut_slice())
                .unwrap_err()
                .to_string(),
            eq("Program is missing valid .ORIG header")
        );
    }
    #[gtest]
    pub fn test_opcode_add() {
        let mut emu = Emulator::new();
        emu.registers.set(0, 22);
        emu.registers.set(1, 128);
        // Add: DR: 2, SR1: 0: 22, Immediate: false, SR2: 1: 128 => R2: 150
        let i1: u16 = 0b0001_010_000_0_00_001;
        // Add: DR: 3, SR1: 2: 150, Immediate: true, imm5: 14 => R3: 164
        let i2: u16 = 0b0001_011_010_1_01110;
        let program = vec![ORIG_HEADER, i1, i2];
        emu.load_program_from_memory(program.as_slice()).unwrap();
        emu.execute().unwrap();
        expect_that!(emu.registers.get(0), eq(22));
        expect_that!(emu.registers.get(1), eq(128));
        expect_that!(emu.registers.get(2), eq(150));
        expect_that!(emu.registers.get(3), eq(164));
        expect_that!(
            emu.registers.get_conditional_register(),
            eq(ConditionFlag::Pos)
        );
    }
    #[gtest]
    pub fn test_opcode_add_underflow() {
        let mut emu = Emulator::new();
        emu.registers.set(0, 0x7FFF); // largest positive number in 2's complement
        emu.registers.set(1, 1);
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 32768
        let i1: u16 = 0b0001_010_000_0_00_001;
        let program = vec![ORIG_HEADER, i1];
        emu.load_program_from_memory(program.as_slice()).unwrap();
        emu.execute().unwrap();
        expect_that!(emu.registers.get(0), eq(0x7FFF));
        expect_that!(emu.registers.get(1), eq(1));
        expect_that!(emu.registers.get(2), eq(32768));
        expect_that!(
            emu.registers.get_conditional_register(),
            eq(ConditionFlag::Neg)
        );
    }
    #[gtest]
    pub fn test_opcode_add_result_0() {
        let mut emu = Emulator::new();
        emu.registers.set(0, 0x7FFF); // largest positive number in 2's complement
        emu.registers.set(1, !0x7FFF + 1);
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 0
        let i1: u16 = 0b0001_010_000_0_00_001;
        let program = vec![ORIG_HEADER, i1];
        emu.load_program_from_memory(program.as_slice()).unwrap();
        emu.execute().unwrap();
        expect_that!(emu.registers.get(0), eq(0x7FFF));
        expect_that!(emu.registers.get(1), eq(!0x7FFF + 1));
        expect_that!(emu.registers.get(2), eq(0));
        expect_that!(
            emu.registers.get_conditional_register(),
            eq(ConditionFlag::Zero)
        );
    }
    #[gtest]
    pub fn test_opcode_and() {
        let mut emu = Emulator::new();
        emu.registers.set(0, 0b1101_1001_0111_0101);
        emu.registers.set(1, 0b0100_1010_0010_1001);
        // Add: DR: 2, SR1: 0, Immediate: false, SR2: 1 => R2: 0
        let i1: u16 = 0b0101_010_000_0_00_001;
        let program = vec![ORIG_HEADER, i1];
        emu.load_program_from_memory(program.as_slice()).unwrap();
        emu.execute().unwrap();
        expect_that!(emu.registers.get(0), eq(0b1101_1001_0111_0101));
        expect_that!(emu.registers.get(1), eq(0b0100_1010_0010_1001));
        expect_that!(emu.registers.get(2), eq(0b0100_1000_0010_0001));
        expect_that!(
            emu.registers.get_conditional_register(),
            eq(ConditionFlag::Pos)
        );
    }
    #[gtest]
    pub fn test_opcode_and_immediate() {
        let mut emu = Emulator::new();
        emu.registers.set(0, 0b1101_1001_0111_0101);
        // Add: DR: 2, SR1: 0, Immediate: true: 21, 0xFFF5 => R2: 0
        let i1: u16 = 0b0101_010_000_1_10101;
        let program = vec![ORIG_HEADER, i1];
        emu.load_program_from_memory(program.as_slice()).unwrap();
        emu.execute().unwrap();
        expect_that!(emu.registers.get(0), eq(0b1101_1001_0111_0101));
        // Immediate sign extended:           0b1111_1111_1111_0101
        expect_that!(emu.registers.get(2), eq(0b1101_1001_0111_0101));
        expect_that!(
            emu.registers.get_conditional_register(),
            eq(ConditionFlag::Neg)
        );
    }
    #[gtest]
    pub fn test_opcode_not() {
        let mut emu = Emulator::new();
        emu.registers.set(0, 0x7FFF); // largest positive number in 2's complement
        // Add: DR: 1, SR1: 0 => R1: 0xFFFE
        let i1: u16 = 0b1001_001_000_111111;
        let program = vec![ORIG_HEADER, i1];
        emu.load_program_from_memory(program.as_slice()).unwrap();
        emu.execute().unwrap();
        expect_that!(emu.registers.get(0), eq(0x7FFF));
        expect_that!(emu.registers.get(1), eq(0x8000));
        expect_that!(
            emu.registers.get_conditional_register(),
            eq(ConditionFlag::Neg)
        );
    }

    #[gtest]
    pub fn test_load_program_disk_hello() {
        let mut emu = Emulator::new();
        emu.load_program("examples/hello_world.o").unwrap();
        let mut ins = emu.instructions().unwrap();
        assert_that!(ins.len(), eq(15));
        assert_that!(ins.next().unwrap().op_code(), eq(Operation::Lea as u8));
        // TODO add more assertions for further content
    }
    #[gtest]
    pub fn test_load_program_max_size() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = ORIG_HEADER;
        emu.load_program_from_memory(program.as_mut_slice())
            .unwrap();
        let ins = emu.instructions().unwrap();
        assert_that!(
            ins.len(),
            eq(usize::from(PROGRAM_SECTION_MAX_INSTRUCTION_COUNT))
        );
    }
    #[gtest]
    pub fn test_load_program_too_large() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1];
        program[0] = ORIG_HEADER;
        assert_that!(
            emu.load_program_from_memory(program.as_mut_slice())
                .unwrap_err()
                .to_string(),
            eq("Program too long, got 52737 u16 instructions while limit is 52736")
        );
    }
}
