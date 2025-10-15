//! # LC-3 Emulator.
//!
//! `lc3-emulator` is an emulator of the LC-3 system.
//! See TODO Spec and other doc Link.
//! Usage starts with loading a program via `hardware::Emulator::load_program`.
//!
//!  # Example
//! ```
//! use lc3_emulator::emulator::Emulator;
//! let mut emu = Emulator::new();
//! let program = vec![0x3000u16];
//! let instructions = emu.load_program(&program).unwrap();
//! assert_eq!(instructions.count(), 0);
//! ```
//! # Errors
//! - Program is missing valid .ORIG header (because it is shorter than one `u16` instruction
//! - Program not loaded at byte offset `0x3000`
//! - Program too long

pub mod emulator;
pub mod errors;
pub(crate) mod hardware;
