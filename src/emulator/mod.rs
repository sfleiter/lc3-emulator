use std::fmt::Debug;
use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::{ProgramLoadedAtWrongAddress, ProgramMissingOrigHeader};
use crate::hardware::Memory;

#[derive(Debug)]
pub struct Instruction {
    opcode: u8,
    dr: u8,
    pc_offset: u16,
}

impl TryFrom<u16> for Instruction {
    type Error = Lc3EmulatorError;

    fn try_from(bits: u16) -> Result<Self, Self::Error> {
        // format: OOOO_DDD_P_PPPP_PPPP
        let opcode = (bits >> 12) as u8;
        let dr = (bits >> 9) as u8 & 0b111;
        let pc_offset = bits & 0b1_1111_1111;
        Ok(Self { opcode, dr, pc_offset })
    }
}

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
    pub fn load_program(&mut self, program: &[u16]) -> Result<
        impl ExactSizeIterator<Item=Instruction> + Debug, Lc3EmulatorError> {

        if program.is_empty() {
            return Err(ProgramMissingOrigHeader);
        }
        let (header, rest) = program.split_at(1);
        if header[0] != 0x3000 {
            let result = Err(ProgramLoadedAtWrongAddress {actual_address: header[0], expected_address: 0x3000});
            return result;
        }
        Ok(self.memory.load_program(rest)?
            .map(|bits| Instruction::try_from(*bits))
            .collect::<Result<Vec<Instruction>,_>>()?
            .into_iter()
        )
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
        assert_eq!(emu.load_program(&vec![].into_boxed_slice()).unwrap_err().to_string(),
            "Program is missing valid .ORIG header");
    }
    #[test]
    pub fn test_load_program_short() {
        let mut emu = Emulator::new();
        let program = vec![HEADER, 0b0111_010_010101010].into_boxed_slice(); // LEA
        let mut instructions = emu.load_program(&program).unwrap();
        assert_eq!(instructions.len(), 1);
        let instruction = instructions.next().unwrap();
        assert_eq!(instruction.opcode, 0b111);
        assert_eq!(instruction.dr, 0b010);
        assert_eq!(instruction.pc_offset, 0b010101010);
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
        let mut program =
            vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1];
        program[0] = HEADER;
        assert_eq!(emu.load_program(program.as_slice()).unwrap_err().to_string(),
            "Program too long, got 26369 u16 instructions while limit is 26368");
    }
}
