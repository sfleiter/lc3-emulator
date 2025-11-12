use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::ProgramTooLong;
use std::fmt::{Debug, Formatter};

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
        match self.program_slice() {
            Ok(p) => {
                let ins = p.len();
                write!(f, "Instructions: {ins:?}, Program section contents: {p:?}")
            }
            Err(e) => match e {
                Lc3EmulatorError::ProgramNotLoaded => write!(f, "Program not yet loaded"),
                _ => write!(f, "Error: {e:?}"),
            },
        }
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

    /// Loads a program without an `.ORIG` header into the memory section
    /// starting from address `_PROGRAM_SECTION_START_BYTES`
    /// and returns an iterator over the loaded instructions.
    ///
    /// # Errors
    /// - Program too long
    pub fn load_program(&mut self, data: &[u16]) -> Result<(), Lc3EmulatorError> {
        if data.len() > usize::from(PROGRAM_SECTION_MAX_INSTRUCTION_COUNT) {
            return Err(ProgramTooLong {
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
    pub fn program_slice(&self) -> Result<&[u16], Lc3EmulatorError> {
        if self.instruction_count != 0 {
            Ok(&self.data[usize::from(PROGRAM_SECTION_START)
                ..usize::from(PROGRAM_SECTION_START + self.instruction_count)])
        } else {
            Err(Lc3EmulatorError::ProgramNotLoaded)
        }
    }
    pub const fn memory(&self) -> Result<&[u16], Lc3EmulatorError> {
        if self.instruction_count != 0 {
            Ok(self.data.as_slice())
        } else {
            Err(Lc3EmulatorError::ProgramNotLoaded)
        }
    }
}
