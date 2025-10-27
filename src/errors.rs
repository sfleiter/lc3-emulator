use std::fmt::{Debug, Display, Formatter};
use std::io;
use thiserror::Error;

#[rustfmt::skip]
#[derive(Error)]
pub enum Lc3EmulatorError {
    #[error("Program not loaded yet")]
    ProgramNotLoaded,
    #[error("Program too long, got {actual_instructions:?} u16 instructions while limit is {maximum_instructions:?}")]
    ProgramTooLong { actual_instructions: usize, maximum_instructions: usize },
    #[error("Program is missing valid .ORIG header")]
    ProgramMissingOrigHeader,
    #[error("Program is not loaded at 0x{expected_address:04X?}' but 0x{actual_address:04X?}")]
    ProgramLoadedAtWrongAddress {actual_address: u16, expected_address: u16},
    #[error(transparent)]
    IoError(#[from] io::Error)
}

impl Debug for Lc3EmulatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
