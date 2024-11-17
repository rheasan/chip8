use std::error::Error;

mod cli;
mod emulate;
mod cpu;
mod chip8;
mod keyboard;
mod ext;
mod tests;
fn main() -> Result<(), Box<dyn Error>> {

    let mut chip8 = chip8::Chip8::new();
    if let Some(args) = cli::parse_args() {
        match args {
            cli::Chip8Command::Emulate { src } => {
                emulate::emulate(src, &mut chip8)?;
            },
        }
    }

    Ok(())
}
