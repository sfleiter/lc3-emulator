use lc3_emulator::emulator;
use lc3_emulator::errors::Lc3EmulatorError;

fn main() -> Result<(), Lc3EmulatorError> {
    let mut emu = emulator::from_program("examples/hello_world.o")?;
    for i in emu.instructions() {
        println!("{i:?}");
    }
    emu.execute().expect("Program execution failed");
    Ok(())
}
