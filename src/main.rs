mod hardware;

use hardware::Emulator;

fn main()  -> Result<(), &'static str> {
    let mut emu = Emulator::new();
    emu.load_program(&vec![0u8; 0xFDFF - 0x3000 + 1].into_boxed_slice())
}
