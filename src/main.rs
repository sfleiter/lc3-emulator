use lc3_emulator::hardware::Emulator;

fn main()  -> Result<(), String> {
    let mut emu = Emulator::new();
    let _ = emu.load_program(&vec![0x3000u16].into_boxed_slice())?;
    Ok(())
}
