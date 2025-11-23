use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[rustfmt::skip]
#[derive(Error, PartialEq, Eq)]
pub enum Lc3EmulatorError {
    #[error("Emulator can only load and execute once, please use a fresh instance, then load and finally execute")]
    WrongState,
    #[error("Program not loaded yet")]
    ProgramNotLoaded,
    #[error("Program needs to be even size in bytes to contain valid u16 instructions, but is {0} bytes long")]
    ProgramNotEvenSize(u64),
    #[error("Program does not fit into memory, file size: {0} is greater than usize")]
    ProgramDoesNotFitIntoMemory(u64),
    #[error("Program too long, got {actual_instructions:?} u16 instructions while limit is {maximum_instructions:?}")]
    ProgramTooLong { actual_instructions: usize, maximum_instructions: u16 },
    #[error("Program is missing valid .ORIG header")]
    ProgramMissingOrigHeader,
    #[error("Program is not loaded at 0x{expected_address:04X?}' but 0x{actual_address:04X?}")]
    ProgramLoadedAtWrongAddress {actual_address: u16, expected_address: u16},
    #[error("Cannot read program from file '{file}': {message}")]
    ProgramNotLoadable {
        file: String,
        message: String
    },
    #[error("Invalid instruction opcode: 0b{0:04b}")]
    InvalidInstruction(u8),
    #[error("Could not write to output: {0}")]
    IOStdoutError(String),
}

impl Debug for Lc3EmulatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
