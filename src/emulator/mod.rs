use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::{ProgramLoadedAtWrongAddress, ProgramMissingOrigHeader};
use crate::hardware::memory::{Memory, PROGRAM_SECTION_START};
use crate::hardware::registers::Registers;
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

/// Wrapper for LC-3 u16 instruction.
/// format is: `OOOO_DDD_P_PPPP_PPPP`
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Instruction(u16);

impl Instruction {
    const MAX_INDEX: u8 = 15;

    /// Gives the value of only the specified bit range.
    ///
    /// # Parameters
    /// - `from`: starting index
    /// - `to`: end index (inclusive), mut be greater or equal to `from`
    ///
    /// # Panics
    /// - asserts that to is greater or equal from
    #[must_use]
    pub fn get_bit_range(self, from: u8, to: u8) -> u16 {
        assert!(
            to >= from,
            "wrong direction of from: {from:?} and to: {to:?}"
        );
        assert!(
            to <= Self::MAX_INDEX,
            "index: {to:?} to u16 is greater than maximum value {:?}",
            Self::MAX_INDEX
        );
        (self.0 >> from) & ((0b1 << (to - from + 1)) - 1)
    }
    /// Gives the value of only the specified bit range and converts that to u8.
    /// See [`Instruction::get_bit_range()`]
    /// # Panics
    /// - value does not fit into u8 with message from `expect`
    #[must_use]
    pub fn get_bit_range_u8(self, from: u8, to: u8, expect: &str) -> u8 {
        u8::try_from(self.get_bit_range(from, to)).expect(expect)
    }
    #[must_use]
    pub fn op_code(self) -> u8 {
        self.get_bit_range_u8(12, 15, "Error parsing op_code")
    }
    #[must_use]
    pub fn dr_number(self) -> u8 {
        self.get_bit_range_u8(9, 11, "Error parsing dr")
    }
    #[must_use]
    pub fn sr1_number(self) -> u8 {
        self.get_bit_range_u8(6, 8, "Error parsing sr1")
    }
    #[must_use]
    pub fn sr2_number(self) -> u8 {
        self.get_bit_range_u8(0, 2, "Error parsing sr2")
    }
    #[must_use]
    fn is_immediate(self) -> bool {
        self.get_bit_range(5, 5) == 1
    }
    fn get_immediate(self) -> u16 {
        // TODO sign extend
        self.get_bit_range(0, 4)
    }
    /// get the last `len` bits from `0` to `len - 1`
    #[must_use]
    pub fn pc_offset(self, len: u8) -> u16 {
        // TODO sign extend
        self.get_bit_range(0, len - 1)
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Op: {:04b}, DR: {:03b}, PC_Off: {:09b}",
            self.op_code(),
            self.dr_number(),
            self.pc_offset(9)
        )
    }
}

impl From<u16> for Instruction {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}

/// The public facing emulator used to run LC-3 programs.
#[derive(Debug)]
pub struct Emulator {
    memory: Memory,
    registers: Registers,
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
        }
    }

    fn load_program_from_memory(&mut self, program: &[u16]) -> Result<(), Lc3EmulatorError> {
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
        self.memory.load_program(rest)
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
        while self.registers.pc < self.memory.program_end() {
            let data = self.memory.memory()?[usize::from(self.registers.pc)];
            let i = Instruction::from(data);
            println!("{i:?}");
            self.execute_instruction(i)?;
            self.registers.pc += 1;
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

    fn add(&mut self, i: Instruction) {
        self.registers.set(
            i.dr_number(),
            self.registers.get(i.sr1_number())
                + if i.is_immediate() {
                    i.get_immediate()
                } else {
                    self.registers.get(i.sr2_number())
                },
        );
        self.registers.update_conditional_register(i.dr_number());
    }
    fn and(&self, i: Instruction) {
        unimplemented!()
    }
    fn not(&self, i: Instruction) {
        unimplemented!()
    }
    fn br(&self, i: Instruction) {
        unimplemented!()
    }
    fn jmp_or_ret(&self, i: Instruction) {
        unimplemented!()
    }
    fn jsr(&self, i: Instruction) {
        unimplemented!()
    }
    fn ld(&self, i: Instruction) {
        unimplemented!()
    }
    fn ldi(&self, i: Instruction) {
        unimplemented!()
    }
    fn ldr(&self, i: Instruction) {
        unimplemented!()
    }
    fn lea(&self, i: Instruction) {
        unimplemented!()
    }
    fn st(&self, i: Instruction) {
        unimplemented!()
    }
    fn sti(&self, i: Instruction) {
        unimplemented!()
    }
    fn str(&self, i: Instruction) {
        unimplemented!()
    }
    fn trap(&self, i: Instruction) {
        unimplemented!()
    }
    fn rti(&self, i: Instruction) {
        unimplemented!()
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
    use crate::emulator::{Emulator, Instruction, ORIG_HEADER, Operation};
    use crate::hardware::memory::PROGRAM_SECTION_MAX_INSTRUCTION_COUNT;
    use crate::hardware::registers::ConditionFlag;
    use googletest::prelude::*;

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize =
        PROGRAM_SECTION_MAX_INSTRUCTION_COUNT as usize + 1;
    #[gtest]
    pub fn test_instr_get_bit_range_valid() {
        let sut = Instruction::from(0b1010_101_101010101);
        expect_that!(sut.op_code(), eq(0b1010));
        expect_that!(sut.dr_number(), eq(0b101));
        expect_that!(sut.pc_offset(9), eq(0b101010101));

        // Add: DR: 3, SR1: 2, Immediate: false, SR2: 1
        let sut = Instruction::from(0b0001_011_010_0_00_001);
        expect_that!(sut.op_code(), eq(1));
        expect_that!(sut.dr_number(), eq(3));
        expect_that!(sut.sr1_number(), eq(2));
        expect_that!(sut.sr2_number(), eq(1));
        expect_that!(sut.is_immediate(), eq(false));
        expect_that!(sut.sr2_number(), eq(1));

        // Add: DR: 7, SR1: 0, Immediate: true, imm5: 30
        let sut = Instruction::from(0b0001_111_000_1_11110);
        expect_that!(sut.op_code(), eq(1));
        expect_that!(sut.dr_number(), eq(7));
        expect_that!(sut.sr1_number(), eq(0));
        expect_that!(sut.is_immediate(), eq(true));
        expect_that!(sut.get_immediate(), eq(30));
    }
    #[gtest]
    #[should_panic(expected = "wrong direction of from: 2 and to: 1")]
    pub fn test_instr_get_bit_range_wrong_order() {
        let sut = Instruction::from(0b1010_101_101010101);
        let _ = sut.get_bit_range(2, 1);
    }
    #[gtest]
    #[should_panic(expected = "index: 16 to u16 is greater than maximum value 15")]
    pub fn test_instr_get_bit_range_index_too_large() {
        let sut = Instruction::from(0b1010_101_101010101);
        let _ = sut.get_bit_range(2, 16);
    }
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

    fn ni(i: &mut impl Iterator<Item = Instruction>) -> Instruction {
        i.next().unwrap()
    }

    #[gtest]
    pub fn test_load_program_short_add() {
        // Add: DR: 2, SR1: 0: 22, Immediate: false, SR2: 1: 128 => R2: 150
        let i1: u16 = 0b0001_010_000_0_00_001;
        // Add: DR: 3, SR1: 2: 150, Immediate: true, imm5: 30 => R3: 180
        let i2: u16 = 0b0001_011_010_1_11110;

        let program = vec![ORIG_HEADER, i1, i2];
        let mut emu = Emulator::new();
        emu.load_program_from_memory(program.as_slice()).unwrap();
        emu.registers.set(0, 22);
        emu.registers.set(1, 128);
        emu.execute().unwrap();
        expect_that!(emu.registers.get(0), eq(22));
        expect_that!(emu.registers.get(1), eq(128));
        expect_that!(emu.registers.get(2), eq(150));
        expect_that!(emu.registers.get(3), eq(180));
        expect_that!(
            emu.registers.get_conditional_register(),
            eq(ConditionFlag::Pos)
        );
    }
    #[gtest]
    pub fn test_load_program_disk_hello() {
        let mut emu = Emulator::new();
        emu.load_program("examples/hello_world.o").unwrap();
        let mut ins = emu.instructions().unwrap();
        assert_that!(ins.len(), eq(15));
        assert_that!(ni(&mut ins).op_code(), eq(Operation::Lea as u8));
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
