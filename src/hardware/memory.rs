use crate::errors::LoadProgramError;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};
#[allow(unused_imports)] // TODO RustRover false positive
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

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
    keyboard_input_receiver: Receiver<u16>,
    keyboard_status_register: Cell<u16>,
    keyboard_data_register: Cell<u16>,
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
                    if self.keyboard_status_register.get() == Self::KEYBOARD_STATUS_REGISTER_UNSET {
                        self.keyboard_input_receiver.try_recv().map_or(
                            &Self::KEYBOARD_STATUS_REGISTER_UNSET,
                            |data| {
                                self.keyboard_status_register
                                    .set(Self::KEYBOARD_STATUS_REGISTER_SET);
                                self.keyboard_data_register.set(data);
                                &Self::KEYBOARD_STATUS_REGISTER_SET
                            },
                        )
                    } else {
                        &Self::KEYBOARD_STATUS_REGISTER_SET
                    }
                }
                MemoryMappedIOLocations::Kbdr => {
                    self.keyboard_status_register
                        .set(Self::KEYBOARD_STATUS_REGISTER_UNSET);
                    let res = self.keyboard_data_register.get();
                    self.keyboard_data_register.set(0);
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
    pub fn new(keyboard_input_receiver: Receiver<u16>) -> Self {
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
            keyboard_input_receiver,
            keyboard_status_register: Cell::from(0),
            keyboard_data_register: Cell::from(0),
            u8_val_table,
        }
    }
    pub(crate) fn set_keyboard_input_receiver(&mut self, receiver: Receiver<u16>) {
        self.keyboard_input_receiver = receiver;
    }
    #[inline]
    fn assert_valid_access(&self, index: u16) {
        assert!(
            (PROGRAM_SECTION_START..=(PROGRAM_SECTION_END)).contains(&index),
            "Address {:#06X} is not in program space when indexing, valid range: {:#06X}..{:#06X}",
            index,
            PROGRAM_SECTION_START,
            PROGRAM_SECTION_START + self.instruction_count
        );
    }
    #[cfg(test)]
    pub(crate) fn with_program_no_kbd_receiver(
        program: &Vec<u16>,
    ) -> Result<Self, LoadProgramError> {
        let (_sender, receiver) = mpsc::channel();
        let mut res = Self::new(receiver);
        res.load_program(program.as_ref())?;
        Ok(res)
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
