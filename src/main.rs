use lc3_emulator::emulator;
use std::env;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        usage(args[0].as_str());
        return Err("Exiting.".into());
    }
    let mut emu = emulator::from_program(args[1].as_str()).map_err(Box::<dyn Error>::from)?;
    emu.execute().map_err(Box::<dyn Error>::from)
}

fn usage(program_name: &str) {
    let program_name = Path::new(program_name).file_name().map_or_else(
        || String::from(file!()),
        |n| String::from_utf8_lossy(n.as_encoded_bytes()).to_string(),
    );
    eprintln!("Usage: {program_name} <FILE>");
    eprintln!("\n<FILE> is a LC-3 obj file usually ending with .obj as output by the");
    eprintln!("lc3as assembler you can download from");
    eprintln!(
        "https://highered.mheducation.com/sites/0072467509/student_view0/lc-3_simulator.html"
    );
}
