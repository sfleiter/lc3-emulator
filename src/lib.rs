//! # LC-3 Emulator.
//!
//! `lc3-emulator` is an emulator of the LC-3 system.
//! Usage starts with loading a program via [`emulator::from_program`].
//!
//!  # Example
//! ```
//! use lc3_emulator::emulator;
//! let mut emu = emulator::from_program("examples/hello_world.o").unwrap();
//! let res =  emu.execute();
//! assert!(res.is_ok());
//! ```
//! # Errors
//! - see [`Lc3EmulatorError`](errors::Lc3EmulatorError)
pub mod emulator;
pub mod errors;
pub mod hardware;
pub(crate) mod numbers;
