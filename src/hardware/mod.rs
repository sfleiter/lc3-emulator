const PROGRAM_SECTION_START: usize = 0x3000;
const PROGRAM_SECTION_END: usize = 0xFDFF;
const PROGRAM_SECTION_LEN: usize = PROGRAM_SECTION_END - PROGRAM_SECTION_START + 1;
const MEMORY_SIZE: usize = 0x3000 + PROGRAM_SECTION_LEN; // TODO
struct Memory {
    /// Index equals memory address
    data:  Vec<u8>,
}

impl Memory {
    fn new() -> Self {
        let mut data = Vec::with_capacity(crate::hardware::MEMORY_SIZE);
        unsafe { data.set_len(MEMORY_SIZE) };
        Self {
            data
        }
    }

    pub fn load_program(&mut self, data: &[u8]) -> Result<&mut [u8], &'static str> {
        if data.len() > PROGRAM_SECTION_LEN {
            return Err("Program too long");
        }
        let program_slice = &mut self.data[PROGRAM_SECTION_START..PROGRAM_SECTION_START + data.len()];
        program_slice.copy_from_slice(data);
        Ok(program_slice)
    }
}

pub struct Emulator {
    memory: Memory,
    pc: usize,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            pc: 0,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<(), &'static str>  {
        let program = self.memory.load_program(program)?;
        // TODO read Opcodes and data
        Ok(())
    }
}
