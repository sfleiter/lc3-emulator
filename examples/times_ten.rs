use lc3_emulator::emulator;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut emu =
        emulator::from_program("examples/times_ten.obj").map_err(Box::<dyn Error>::from)?;
    emu.execute().map_err(Box::<dyn Error>::from)?;
    println!("Result: {:?}", emu.registers().get(3).as_decimal());
    Ok(())
}
