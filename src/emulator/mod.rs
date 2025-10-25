use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::{ProgramLoadedAtWrongAddress, ProgramMissingOrigHeader};
use crate::hardware::{Memory, PROGRAM_SECTION_START_BYTES};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ptr::from_mut;

const ORIG_HEADER: u16 = switch_endian_bytes(PROGRAM_SECTION_START_BYTES);

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
        Ok(Self {
            opcode,
            dr,
            pc_offset,
        })
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
    /// Constructor method
    #[must_use]
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
        }
    }

    fn load_program_from_memory(
        &mut self,
        // pass by value since former owner does not need data and to allow copy_from_slice
        program: &[u16],
    ) -> Result<(), Lc3EmulatorError> {
        let Some((header, rest)) = program.split_at_checked(1) else {
            return Err(ProgramMissingOrigHeader);
        };
        if header[0] != ORIG_HEADER {
            let result = Err(ProgramLoadedAtWrongAddress {
                actual_address: header[0],
                expected_address: PROGRAM_SECTION_START_BYTES,
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
        let size = file.metadata()?.len();
        // one u16 equals 2 bytes plus 2 bytes for the .ORIG section
        let mut file_data: Vec<u8> = Vec::with_capacity(size as usize + 2);
        let mut reader = BufReader::new(file);
        reader.read_to_end(file_data.as_mut())?;
        let final_data: &mut [u16] = unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                from_mut::<[u8]>(file_data.as_mut_slice()).cast::<u16>(),
                file_data.len() / 2,
            )
        };
        self.load_program_from_memory(final_data)
    }

    pub fn instructions(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = Instruction> + Debug, Lc3EmulatorError> {
        Ok(self
            .memory
            .program_slice()?
            .iter()
            .map(|bits| Instruction::try_from(*bits))
            .collect::<Result<Vec<Instruction>, _>>()?
            .into_iter())
    }
}

#[inline]
#[cfg(target_endian = "little")]
const fn switch_endian_bytes(data: u16) -> u16 {
    // eprintln!("data: 0x{:04X?}", data);
    data.rotate_right(8)
}
#[inline]
#[cfg(target_endian = "big")]
const fn switch_endian_bytes(data: u16) -> u16 {
    data
}

#[cfg(test)]
mod tests {
    use crate::emulator::{Emulator, ORIG_HEADER};
    use crate::hardware::PROGRAM_SECTION_MAX_INSTRUCTION_COUNT;

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize =
        PROGRAM_SECTION_MAX_INSTRUCTION_COUNT + 1;
    #[test]
    pub fn test_load_program_missing_header() {
        let mut emu = Emulator::new();
        assert_eq!(
            emu.load_program_from_memory(Vec::with_capacity(0).as_mut_slice())
                .unwrap_err()
                .to_string(),
            "Program is missing valid .ORIG header"
        );
    }

    #[test]
    pub fn test_load_program_short() {
        let mut emu = Emulator::new();
        let mut program = vec![ORIG_HEADER, 0b0111_010_010101010_]; // LEA
        emu.load_program_from_memory(program.as_mut_slice())
            .unwrap();
        let mut instructions = emu.instructions().unwrap();
        assert_eq!(instructions.len(), 1);
        let instruction = instructions.next().unwrap();
        assert_eq!(instruction.opcode, 0b111);
        assert_eq!(instruction.dr, 0b010);
        assert_eq!(instruction.pc_offset, 0b1010_1010);
    }
    #[test]
    pub fn test_load_program_disk_hello() {
        let mut emu = Emulator::new();
        emu.load_program("examples/hello_world.o").unwrap();
        let instructions = emu.instructions().unwrap();
        assert_eq!(instructions.len(), 15);
    }
    #[test]
    pub fn test_load_program_max_size() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = ORIG_HEADER;
        emu.load_program_from_memory(program.as_mut_slice())
            .unwrap();
        let instructions = emu.instructions().unwrap();
        assert_eq!(instructions.len(), PROGRAM_SECTION_MAX_INSTRUCTION_COUNT);
    }
    #[test]
    pub fn test_load_program_too_large() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1];
        program[0] = ORIG_HEADER;
        assert_eq!(
            emu.load_program_from_memory(program.as_mut_slice())
                .unwrap_err()
                .to_string(),
            "Program too long, got 26369 u16 instructions while limit is 26368"
        );
    }
}
