mod hardware;

use hardware::Emulator;

fn main()  -> Result<(), String> {
    let mut emu = Emulator::new();
    emu.load_program(&vec![0x3000u16].into_boxed_slice())?;
    Ok(())
}
