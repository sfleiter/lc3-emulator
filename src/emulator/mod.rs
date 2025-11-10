use crate::errors::Lc3EmulatorError;
use crate::errors::Lc3EmulatorError::{ProgramLoadedAtWrongAddress, ProgramMissingOrigHeader};
use crate::hardware::memory::{Memory, PROGRAM_SECTION_START_BYTES};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufReader, Read};

const ORIG_HEADER: u16 = PROGRAM_SECTION_START_BYTES;

#[rustfmt::skip]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum Operation {
    Add  = 0b0001,
    And  = 0b0101,
    Not  = 0b1001,
    Br   = 0b0000,
    Jmp  = 0b1100,
    Jsr  = 0b0100,
    //    Ret = 0b1100, // TODO Clash with JMP
    Ld   = 0b0010,
    Ldi  = 0b1010,
    Ldr  = 0b0110,
    Lea  = 0b1110,
    St   = 0b0011,
    Sti  = 0b1011,
    Str  = 0b0111,
    Trap = 0b1111,
    Rti  = 0b1000,
    Reserved = 1101,
}

impl TryFrom<Instruction> for Operation {
    type Error = Lc3EmulatorError;
    fn try_from(i: Instruction) -> Result<Self, Self::Error> {
        let operation = match (i.opcode, i.dr, i.pc_offset) {
            (o, _, _) if o == Self::Add as u8 => Self::Add,
            (o, _, _) if o == Self::And as u8 => Self::And,
            (o, _, _) if o == Self::Not as u8 => Self::Not,
            (o, _, _) if o == Self::Br as u8 => Self::Br,
            (o, _, _) if o == Self::Jmp as u8 => Self::Jmp,
            (o, _, _) if o == Self::Jsr as u8 => Self::Jsr,
            (o, _, _) if o == Self::Ld as u8 => Self::Ld,
            (o, _, _) if o == Self::Ldi as u8 => Self::Ldi,
            (o, _, _) if o == Self::Ldr as u8 => Self::Ldr,
            (o, _, _) if o == Self::Lea as u8 => Self::Lea,
            (o, _, _) if o == Self::St as u8 => Self::St,
            (o, _, _) if o == Self::Sti as u8 => Self::Sti,
            (o, _, _) if o == Self::Str as u8 => Self::Str,
            (o, _, _) if o == Self::Trap as u8 => Self::Trap,
            (o, _, _) if o == Self::Rti as u8 => Self::Rti,
            (o, _, _) => return Err(Lc3EmulatorError::InvalidInstruction(o)),
        };
        Ok(operation)
    }
}
struct Instruction {
    opcode: u8,
    dr: u8,
    pc_offset: u16,
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Op: {:04b}, DR: {:03b}, PC_Off: {:09b}",
            self.opcode, self.dr, self.pc_offset
        )
    }
}

impl From<u16> for Instruction {
    fn from(bits: u16) -> Self {
        // format: OOOO_DDD_P_PPPP_PPPP
        let opcode = (bits >> 12) as u8;
        let dr = (bits >> 9) as u8 & 0b111;
        let pc_offset = bits & 0b1_1111_1111;
        // println!("Ins: {bits:016b}, Op: {opcode:04b}, DR: {dr:03b}, PC_Off: {pc_offset:09b}");
        Self {
            opcode,
            dr,
            pc_offset,
        }
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

    pub fn operations(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = Operation> + Debug, Lc3EmulatorError> {
        Ok(self
            .memory
            .program_slice()?
            .iter()
            .map(|bits| Instruction::from(*bits))
            .map(Operation::try_from)
            .collect::<Result<Vec<Operation>, Lc3EmulatorError>>()?
            .into_iter())
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

    fn no(i: &mut impl Iterator<Item = Operation>) -> Operation {
        i.next().unwrap()
    }

    #[test]
    pub fn test_load_program_short() {
        let mut emu = Emulator::new();
        let mut program = vec![ORIG_HEADER, 0b1110_010_010101010]; // LEA
        emu.load_program_from_memory(program.as_mut_slice())
            .unwrap();
        let mut ops = emu.operations().unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(Operation::Lea, no(&mut ops));
    }
    #[test]
    pub fn test_load_program_disk_hello() {
        let mut emu = Emulator::new();
        emu.load_program("examples/hello_world.o").unwrap();
        let mut ops = emu.operations().unwrap();
        assert_eq!(ops.len(), 15);
        assert_eq!(Operation::Lea, no(&mut ops));
        // TODO add more assertions for further content
    }
    #[test]
    pub fn test_load_program_max_size() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = ORIG_HEADER;
        emu.load_program_from_memory(program.as_mut_slice())
            .unwrap();
        let ops = emu.operations().unwrap();
        assert_eq!(ops.len(), PROGRAM_SECTION_MAX_INSTRUCTION_COUNT);
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
