use lc3_emulator::emulator;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut emu =
        emulator::from_program("examples/hello_world.o").map_err(Box::<dyn Error>::from)?;
    emu.execute().map_err(Box::<dyn Error>::from)
}
