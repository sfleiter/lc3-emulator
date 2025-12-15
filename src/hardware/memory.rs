use crate::errors::LoadProgramError;
use crate::hardware::keyboard::KeyboardInputProvider;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};
use std::rc::Rc;

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
    keyboard_input_provider: Rc<RefCell<dyn KeyboardInputProvider>>,
    u8_val_table: [u16; 256],
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
/// Memory regions mapped to IO functionality.
#[repr(u16)]
#[derive(enumn::N)]
pub enum MemoryMappedIOLocations {
    /// Keyboard Status Register
    Kbsr = 0xFE00,
    /// Keyboard Data Register
    Kbdr = 0xFE02,
}
impl Index<u16> for Memory {
    type Output = u16;
    fn index(&self, index: u16) -> &Self::Output {
        MemoryMappedIOLocations::n(index).map_or_else(
            || {
                self.assert_valid_access(index);
                &self.data[usize::from(index)]
            },
            |mapped_io_loc| match mapped_io_loc {
                MemoryMappedIOLocations::Kbsr => {
                    if self
                        .keyboard_input_provider
                        .borrow_mut()
                        .check_input_available()
                        .unwrap_or(false)
                    {
                        &Self::KEYBOARD_STATUS_REGISTER_SET
                    } else {
                        &Self::KEYBOARD_STATUS_REGISTER_UNSET
                    }
                }
                MemoryMappedIOLocations::Kbdr => {
                    let res = self
                        .keyboard_input_provider
                        .borrow_mut()
                        .get_input_character();
                    &self.u8_val_table[res as usize]
                }
            },
        )
    }
}
impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.assert_valid_access(index);
        &mut self.data[usize::from(index)]
    }
}
impl Memory {
    const KEYBOARD_STATUS_REGISTER_SET: u16 = 1 << 15;
    const KEYBOARD_STATUS_REGISTER_UNSET: u16 = 0;
    pub fn new(keyboard_input_provider: Rc<RefCell<dyn KeyboardInputProvider>>) -> Self {
        let data = vec![0x0u16; usize::from(MEMORY_SIZE_U16)];
        let mut u8_val_table: [u16; 256] = [0; 256];
        for (idx, b) in u8_val_table.iter_mut().enumerate() {
            #[expect(clippy::cast_possible_truncation)]
            {
                *b = idx as u16;
            }
        }
        Self {
            data,
            instruction_count: 0,
            keyboard_input_provider,
            u8_val_table,
        }
    }
    #[inline]
    fn assert_valid_access(&self, index: u16) {
        assert!(
            (PROGRAM_SECTION_START..=PROGRAM_SECTION_END).contains(&index),
            "Address {:#06X} is not in program space when indexing, valid range: {:#06X}..{:#06X}",
            index,
            PROGRAM_SECTION_START,
            PROGRAM_SECTION_START + self.instruction_count
        );
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
