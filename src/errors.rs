//! Errors that can occur using this crate.
use displaydoc::Display;
use std::fmt::{Debug, Display, Formatter};

/// Issues are split in 2 groups
/// - Errors starting with `Program`: invalid programs or errors during attempts to load them.
/// - other: Errors during execution
///
/// `Display` and `Debug` provide all necessary details.
#[rustfmt::skip]
#[expect(clippy::doc_markdown, reason= "using backticks as suggested would break displaydoc")]
#[derive(Display, PartialEq, Eq)]
pub enum Lc3EmulatorError {
    /// Program is missing valid .ORIG header
    ProgramMissingOrigHeader,
    /// Loading an empty program is not allowed
    ProgramEmpty,
    /// Programs mut be an even amount of bytes (multiple of 16-bit instructions), but is {0} bytes long.
    ProgramNotEvenSize(u64),
    /// Program does not fit into memory, file size: {0} is greater than usize
    ProgramDoesNotFitIntoMemory(u64),
    /// Program too long, got {actual_instructions:?} u16 instructions while limit is {maximum_instructions:?}
    ProgramTooLong { actual_instructions: usize, maximum_instructions: u16 },
    /// Program is not loaded at {expected_address:#06X?} but {actual_address:#06X?}
    ProgramLoadedAtWrongAddress {actual_address: u16, expected_address: u16},
    /// Cannot read program from file '{file}': {message}
    ProgramNotLoadable {
        file: String,
        message: String
    },
    /// The reserved opcode {0:#06b} was found which is not specified. Most probably an invalid program.
    ReservedInstructionFound(u8),
    /// Error during writing program output: {0}
    IOStdoutError(String),
}

impl Debug for Lc3EmulatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
