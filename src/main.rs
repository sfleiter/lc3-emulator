use lc3_emulator::emulator::Emulator;
use lc3_emulator::errors::Lc3EmulatorError;

fn main() -> Result<(), Lc3EmulatorError> {
    let mut emu = Emulator::new();
    let _ = emu.load_program(&vec![0x3000u16].into_boxed_slice())?;
    Ok(())
}
