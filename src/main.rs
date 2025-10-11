mod hardware;

use hardware::Emulator;

fn main()  -> Result<(), &'static str> {
    let mut emu = Emulator::new();
    emu.load_program(&[0x0; 0xFDFF - 0x3000 + 1])
}
