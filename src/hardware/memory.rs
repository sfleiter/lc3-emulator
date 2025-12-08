use crate::errors::LoadProgramError;
use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};

pub const PROGRAM_SECTION_START: u16 = 0x3000;
pub const PROGRAM_SECTION_END: u16 = 0xFDFF;
pub const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT: u16 =
    PROGRAM_SECTION_END - PROGRAM_SECTION_START + 1;
const MEMORY_SIZE_U16: u16 = PROGRAM_SECTION_START + PROGRAM_SECTION_MAX_INSTRUCTION_COUNT; // TODO

/// An abstraction for the LC-3 memory including application but excluding registers.
pub struct Memory {
    /// Index equals memory address
    data: Vec<u16>,
    instruction_count: u16,
}
impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let slice = self.program_slice();
        write!(
            f,
            "Instructions: {:?}, Program section contents: {slice:?}",
            slice.len()
        )
    }
}

impl Index<u16> for Memory {
    type Output = u16;
    fn index(&self, index: u16) -> &Self::Output {
        self.assert_valid_access(index);
        &self.data[usize::from(index)]
    }
}
impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.assert_valid_access(index);
        &mut self.data[usize::from(index)]
    }
}
impl Memory {
    pub fn new() -> Self {
        let data = vec![0x0u16; usize::from(MEMORY_SIZE_U16)];
        Self {
            data,
            instruction_count: 0,
        }
    }
    #[inline]
    fn assert_valid_access(&self, index: u16) {
        assert!(
            (PROGRAM_SECTION_START..(PROGRAM_SECTION_START + self.instruction_count))
                .contains(&index),
            "Address {:#06X} is not in program space when indexing, valid range: {:#06X}..{:#06X}",
            index,
            PROGRAM_SECTION_START,
            PROGRAM_SECTION_START + self.instruction_count
        );
    }
    #[cfg(test)]
    pub(crate) fn with_program(program: &Vec<u16>) -> Result<Self, LoadProgramError> {
        let mut res = Self::new();
        res.load_program(program.as_ref())?;
        Ok(res)
    }
    /// Loads a program without an `.ORIG` header into the memory section
    /// starting from address `_PROGRAM_SECTION_START_BYTES`
    /// and returns an iterator over the loaded instructions.
    ///
    /// # Errors
    /// - Program too long
    pub fn load_program(&mut self, data: &[u16]) -> Result<(), LoadProgramError> {
        if data.len() > usize::from(PROGRAM_SECTION_MAX_INSTRUCTION_COUNT) {
            return Err(LoadProgramError::ProgramTooLong {
                actual_instructions: data.len(),
                maximum_instructions: PROGRAM_SECTION_MAX_INSTRUCTION_COUNT,
            });
        }
        self.instruction_count = u16::try_from(data.len()).expect("instruction count too long");
        let program_slice = &mut self.data[usize::from(PROGRAM_SECTION_START)
            ..usize::from(PROGRAM_SECTION_START + self.instruction_count)];
        program_slice.copy_from_slice(data);
        Ok(())
    }
    pub const fn program_end(&self) -> u16 {
        PROGRAM_SECTION_START + self.instruction_count
    }
    pub fn program_slice(&self) -> &[u16] {
        &self.data[usize::from(PROGRAM_SECTION_START)
            ..usize::from(PROGRAM_SECTION_START + self.instruction_count)]
    }
}
