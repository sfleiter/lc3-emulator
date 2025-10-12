const _PROGRAM_SECTION_START_BYTES: usize = 0x3000;
const PROGRAM_SECTION_START_U16: usize = _PROGRAM_SECTION_START_BYTES / 2;
const _PROGRAM_SECTION_END_BYTES: usize = 0xFDFF;
const PROGRAM_SECTION_END_U16: usize = _PROGRAM_SECTION_END_BYTES / 2;
const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT: usize = PROGRAM_SECTION_END_U16 - PROGRAM_SECTION_START_U16 + 1;
const MEMORY_SIZE_U16: usize = PROGRAM_SECTION_START_U16 + PROGRAM_SECTION_MAX_INSTRUCTION_COUNT; // TODO
struct Memory {
    /// Index equals memory address
    data:  Vec<u16>,
}

impl Memory {
    fn new() -> Self {
        let data = vec![0x0u16; MEMORY_SIZE_U16];
        Self {
            data
        }
    }

    pub fn load_program(&mut self, data: &[u16]) -> Result<&mut [u16], &'static str> {
        if data.len() > PROGRAM_SECTION_MAX_INSTRUCTION_COUNT {
            return Err("Program too long");
        }
        let program_slice = &mut self.data[PROGRAM_SECTION_START_U16..PROGRAM_SECTION_START_U16 + data.len()];
        program_slice.copy_from_slice(data);
        Ok(program_slice)
    }
}

pub struct Emulator {
    memory: Memory,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
        }
    }

    pub fn load_program(&mut self, program: &[u16]) -> Result<(), String>  {
        if program.is_empty() {
            return Err("Program is missing valid .ORIG header".into());
        }
        let (header, rest) = program.split_at(1);
        if header[0] != 0x3000 {
            let result = Err(format!("Program is not loaded at '0x3000' but 0x{:016x}", header[0]));
            return result;
        }
        let _program = self.memory.load_program(rest)?;
        // TODO read Opcodes and data
        // TODO tests
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::hardware::{Emulator, PROGRAM_SECTION_MAX_INSTRUCTION_COUNT};

    const PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER: usize = PROGRAM_SECTION_MAX_INSTRUCTION_COUNT + 1;
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
        emu.load_program(&vec![HEADER].into_boxed_slice()).unwrap();
    }
    #[test]
    pub fn test_load_program_max_size() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER];
        program[0] = HEADER;
        emu.load_program(program.as_slice()).unwrap();
    }
    #[test]
    pub fn test_load_program_too_large() {
        let mut emu = Emulator::new();
        let mut program = vec![0x0u16; PROGRAM_SECTION_MAX_INSTRUCTION_COUNT_WITH_HEADER + 1];
        program[0] = HEADER;
        emu.load_program(program.as_slice()).expect_err("Loading too large program should fail");
    }


}