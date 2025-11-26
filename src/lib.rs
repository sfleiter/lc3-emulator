//! # LC-3 Emulator.
//!
//! `lc3-emulator` is an emulator of the LC-3 system.
//! Usage starts with loading a program via [`emulator::from_program`].
//!
//!  # Example
//! ```
//! use lc3_emulator::emulator;
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut emu =
//!     // from_program returns Result<(), LoadProgramError>
//!     emulator::from_program("examples/hello_world.o")
//!         .map_err(Box::<dyn Error>::from)?;
//!
//!     // execute returns Result<(), ExecutionError>
//!     emu.execute()
//!         .map_err(Box::<dyn Error>::from)
//! }
//! ```
//! # Errors
//! - see [`LoadProgramError`](errors::LoadProgramError)
//! - see [`ExecutionError`](errors::ExecutionError)
pub mod emulator;
pub mod errors;
pub mod hardware;
pub(crate) mod numbers;
mod terminal;
