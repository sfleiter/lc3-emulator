use lc3_emulator::emulator::Emulator;
use lc3_emulator::errors::Lc3EmulatorError;

fn main() -> Result<(), Lc3EmulatorError> {
    let mut emu = Emulator::new();
    emu.load_program("examples/hello_world.o")?;
    for i in emu.instructions()? {
        println!("{i:?}");
    }
    Ok(())
}
