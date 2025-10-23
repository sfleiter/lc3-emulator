use lc3_emulator::emulator::Emulator;

fn main() {
    let mut emu = Emulator::new();
    let res = emu.load_program_from_file("examples/hello_world.o");
    match res {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {e}"),
    }
}
