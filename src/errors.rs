use thiserror::Error;


#[derive(Error, Debug)]
pub enum Lc3EmulatorError {
    #[error("Program too long, got {actual_instructions:?} u16 instructions while limit is {maximum_instructions:?}")]
    ProgramTooLong { actual_instructions: usize, maximum_instructions: usize },
    #[error("Program is too short, got {actual_instructions:?} instructions while minimum is {minimum_instructions:?}")]
    ProgramTooShort { actual_instructions: usize, minimum_instructions: usize },
    #[error("Program is missing valid .ORIG header")]
    ProgramMissingOrigHeader,
    #[error("Program is not loaded at 0x{expected_address:0x?}' but 0x{actual_address:0x?}")]
    ProgramLoadedAtWrongAddress {actual_address: u16, expected_address: u16},
}
