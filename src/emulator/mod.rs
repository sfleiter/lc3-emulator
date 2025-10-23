use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::{ProgramLoadedAtWrongAddress, ProgramMissingOrigHeader};
use crate::hardware::{Memory, PROGRAM_SECTION_START_BYTES};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read};

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

    #[allow(clippy::needless_pass_by_value)]
    fn load_program(
        &mut self,
        // pass by value since former owner does not need data and to allow copy_from_slice
        program: Vec<u16>,
    ) -> Result<(), Lc3EmulatorError> {
        let Some((header, rest)) = program.split_at_checked(1) else {
            return Err(ProgramMissingOrigHeader);
        };
        if header[0] != PROGRAM_SECTION_START_BYTES {
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
    pub fn load_program_from_file(&mut self, path: &str) -> Result<(), Lc3EmulatorError> {
        let file = File::open(path)?;
        let fs = file.metadata()?.len();
        // one u16 equals 2 bytes plus 2 bytes for the .ORIG section
        let mut program: Vec<u16> = Vec::with_capacity(fs as usize / 2 + 2);
        let mut reader = BufReader::new(file);
        let mut buf = [0u8; 2];
        loop {
            let bytes = reader.read(&mut buf)?;
            match bytes {
                0 => break,
                2 => program.push(byte_order_to_little(buf)),
                1 => todo!("Not implemented"),
                _ => unreachable!(),
            }
        }
        self.load_program(program)
    }

    pub fn instructions(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = Instruction> + Debug, Lc3EmulatorError> {
        Ok(self
            .memory
            .program_slice()
            .iter()
            .map(|bits| Instruction::try_from(bits[0]))
            .collect::<Result<Vec<Instruction>, _>>()?
            .into_iter())
    }
}

#[inline]
#[cfg(target_endian = "little")]
const fn byte_order_to_little(data: [u8; 2]) -> u16 {
    // eprintln!("data: 0x{:02X?}{:02X?}", data[0], data[1]);
    data[0] as u16 | (data[1] as u16) << 8
}
#[inline]
#[cfg(target_endian = "big")]
const fn byte_order_to_little(data: [u8; 2]) -> u16 {
    data[1] as u16 | (data[0] as u16) << 8
}

#[cfg(test)]
mod tests {
    use crate::emulator::Emulator;
    use crate::hardware::{PROGRAM_SECTION_MAX_INSTRUCTION_COUNT, PROGRAM_SECTION_START_BYTES};

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize =
        PROGRAM_SECTION_MAX_INSTRUCTION_COUNT + 1;
    const HEADER: u16 = PROGRAM_SECTION_START_BYTES;
    #[test]
    pub fn test_load_program_missing_header() {
        let mut emu = Emulator::new();
        assert_eq!(
            emu.load_program(Vec::with_capacity(0))
                .unwrap_err()
                .to_string(),
            "Program is missing valid .ORIG header"
        );
    }

    #[test]
    pub fn test_load_program_short() {
        let mut emu = Emulator::new();
        let program = vec![HEADER, 0b0111_010_010101010]; // LEA
        emu.load_program(program).unwrap();
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
        emu.load_program_from_file("examples/hello_world.o")
            .unwrap();
        let mut instructions = emu.instructions().unwrap();
        assert_eq!(instructions.len(), 1);
        let instruction = instructions.next().unwrap();
        assert_eq!(instruction.opcode, 0b111);
        assert_eq!(instruction.dr, 0b010);
        assert_eq!(instruction.pc_offset, 0b1010_1010);
    }
    #[test]
    pub fn test_load_program_max_size() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = HEADER;
        emu.load_program(program).unwrap();
        let instructions = emu.instructions().unwrap();
        assert_eq!(instructions.len(), PROGRAM_SECTION_MAX_INSTRUCTION_COUNT);
    }
    #[test]
    pub fn test_load_program_too_large() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1];
        program[0] = HEADER;
        assert_eq!(
            emu.load_program(program).unwrap_err().to_string(),
            "Program too long, got 26369 u16 instructions while limit is 26368"
        );
    }
}
