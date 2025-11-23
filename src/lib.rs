//! # LC-3 Emulator.
//!
//! `lc3-emulator` is an emulator of the LC-3 system.
//! Usage starts with loading a program via `hardware::Emulator::load_program`.
//!
//!  # Example
//! ```
//! use lc3_emulator::emulator::Emulator;
//! let mut emu = Emulator::new();
//! emu.load_program("examples/hello_world.o").unwrap();
//! let instructions =  emu.instructions();
//! assert_eq!(instructions.unwrap().count(), 15);
//! ```
//! # Errors
//! - Program is missing valid .ORIG header (because it is shorter than one `u16` instruction
//! - Program not loaded at byte offset `0x3000`
//! - Program too long

pub mod emulator;
pub mod errors;
pub(crate) mod hardware;
pub(crate) mod numbers;
