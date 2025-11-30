//! Errors that can occur using this crate.
//!
//! The crate's code is designed in a way that functions/method _can_ trigger all the enum variants
//! specified in the returned [`Result`]

use displaydoc::Display;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Possible errors during program load.
///
/// Issues are invalid programs or errors during attempts to load them.
/// `Display` and `Debug` provide all necessary details.
#[rustfmt::skip]
#[expect(clippy::doc_markdown, reason= "using backticks as suggested would break displaydoc")]
#[derive(Display, PartialEq, Eq)]
pub enum LoadProgramError {
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
    /// Program is not loaded at {expected_address:#06X} but {actual_address:#06X}
    ProgramLoadedAtWrongAddress {actual_address: u16, expected_address: u16},
    /// Cannot read program from file '{file}': {message}
    ProgramNotLoadable {
        file: String,
        message: String
    },
}
impl Debug for LoadProgramError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
impl Error for LoadProgramError {}

/// Possible errors during program execution.
///
/// `Display` and `Debug` provide all necessary details.
#[rustfmt::skip]
#[derive(Display, PartialEq, Eq)]
pub enum ExecutionError {
    /// The reserved opcode {0:#06b} was found which is not specified. Most probably an invalid program.
    ReservedInstructionFound(u8),
    /// Error during reading Stdin or writing program output to Stdout: {0}
    IOInputOutputError(String),
    /// Unknown trap routine found: {0:#06X}
    UnknownTrapRoutine(u16),
}
impl Debug for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
impl Error for ExecutionError {}
