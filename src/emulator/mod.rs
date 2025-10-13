use std::slice::Iter;
use crate::hardware::Memory;

/// The public facing emulator used to run LC-3 programs.
pub struct Emulator {
    memory: Memory,
}
impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
impl Emulator {
    /// Constructor method, all parameters according to spec.
    #[must_use]
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
        }
    }

    /// Loads a program into the memory section starting from address `_PROGRAM_SECTION_START_BYTES`
    /// and returns an iterator over the loaded instructions.
    ///
    /// TODO Should this return the iter or have a separate method for repetitive calls?
    ///
    /// # Errors
    /// - Program is missing valid .ORIG header (because it is shorter than one `u16` instruction
    /// - Program not loaded at byte offset `0x3000`
    /// - Program too long
    pub fn load_program(&mut self, program: &[u16]) -> Result<Iter<'_, u16>, String> {
        if program.is_empty() {
            return Err("Program is missing valid .ORIG header".into());
        }
        let (header, rest) = program.split_at(1);
        if header[0] != 0x3000 {
            let result = Err(format!("Program is not loaded at '0x3000' but 0x{:016x}", header[0]));
            return result;
        }
        let instructions = self.memory.load_program(rest)?;
        // TODO read Opcodes and data
        // TODO tests
        Ok(instructions)
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::Emulator;
    use crate::hardware::{PROGRAM_SECTION_MAX_INSTRUCTION_COUNT};

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize
    = PROGRAM_SECTION_MAX_INSTRUCTION_COUNT + 1;
    const HEADER: u16 = 0x3000u16;
    #[test]
    pub fn test_load_program_empty() {
        let mut emu = Emulator::new();
        emu.load_program(&vec![].into_boxed_slice())
            .expect_err("Loading empty program without .ORIG header should fail");
    }
    #[test]
    pub fn test_load_program_minimal() {
        let mut emu = Emulator::new();
        let instructions = emu.load_program(&vec![HEADER].into_boxed_slice()).unwrap();
        assert_eq!(instructions.len(), 0);
    }
    #[test]
    pub fn test_load_program_max_size() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = HEADER;
        let instructions = emu.load_program(program.as_slice()).unwrap();
        assert_eq!(instructions.len(), PROGRAM_SECTION_MAX_INSTRUCTION_COUNT);
    }
    #[test]
    pub fn test_load_program_too_large() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1];
        program[0] = HEADER;
        let _ = emu.load_program(program.as_slice())
            .expect_err("Loading too large program should fail");
    }
}
