use lc3_emulator::emulator;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut emu = emulator::from_program("examples/memory_mapped_io_keyboard.obj")
        .map_err(Box::<dyn Error>::from)?;
    emu.execute().map_err(Box::<dyn Error>::from)?;
    emu.reset_registers();
    emu.execute().map_err(Box::<dyn Error>::from)
}
