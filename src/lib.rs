//! # LC-3 Emulator.
//!
//! `lc3-emulator` is an emulator of the LC-3 system.
//! Usage starts with loading a program via [`emulator::from_program`] to receive an [`Emulator`](emulator::Emulator) to then execute it
//! by [`Emulator::execute`](emulator::Emulator::execute).
//!
//!  # Example
//! ```
//! use lc3_emulator::emulator;
//! use lc3_emulator::hardware;
//! use lc3_emulator::emulator::stdout_helpers::StdoutForDocTest;
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut emu =
//!         // from_program returns Result<(), LoadProgramError>
//!        emulator::from_program("examples/times_ten.obj")
//!            .map_err(Box::<dyn Error>::from)?;
//!
//!     let mut stdout = StdoutForDocTest::new();
//!     // execute returns Result<(), ExecutionError>
//!     emu.execute_with_stdout(&mut stdout).map_err(Box::<dyn Error>::from)?;
//!     assert_eq!(30, emu.registers().get(3).as_decimal());
//!     Ok(())
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
