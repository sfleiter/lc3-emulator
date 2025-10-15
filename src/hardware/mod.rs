use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::ProgramTooLong;
use std::slice::Iter;

pub const PROGRAM_SECTION_START_BYTES: u16 = 0x3000;
const PROGRAM_SECTION_START_U16: usize = PROGRAM_SECTION_START_BYTES as usize / 2;
const _PROGRAM_SECTION_END_BYTES: usize = 0xFDFF;
const PROGRAM_SECTION_END_U16: usize = _PROGRAM_SECTION_END_BYTES / 2;
pub const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT: usize =
    PROGRAM_SECTION_END_U16 - PROGRAM_SECTION_START_U16 + 1;
const MEMORY_SIZE_U16: usize = PROGRAM_SECTION_START_U16 + PROGRAM_SECTION_MAX_INSTRUCTION_COUNT; // TODO
pub struct Memory {
    /// Index equals memory address
    data: Vec<u16>,
    instruction_count: usize,
}

/// An abstraction for the LC-3 memory including application but excluding registers.
impl Memory {
    pub fn new() -> Self {
        let data = vec![0x0u16; MEMORY_SIZE_U16];
        Self {
            data,
            instruction_count: 0, // TODO Only needed if instruction iter is needed multiple times
        }
    }

    /// Loads a program without an `.ORIG` header into the memory section
    /// starting from address `_PROGRAM_SECTION_START_BYTES`
    /// and returns an iterator over the loaded instructions.
    ///
    /// TODO Should this return the iter or have a separate method for repetitive calls?
    ///
    /// # Errors
    /// - Program too long
    pub fn load_program(&mut self, data: &[u16]) -> Result<Iter<'_, u16>, Lc3EmulatorError> {
        if data.len() > PROGRAM_SECTION_MAX_INSTRUCTION_COUNT {
            return Err(ProgramTooLong {
                actual_instructions: data.len(),
                maximum_instructions: PROGRAM_SECTION_MAX_INSTRUCTION_COUNT,
            });
        }
        self.instruction_count = data.len();
        let program_slice = &mut self.data
            [PROGRAM_SECTION_START_U16..PROGRAM_SECTION_START_U16 + self.instruction_count];
        program_slice.copy_from_slice(data);
        Ok(program_slice.iter())
    }
}
