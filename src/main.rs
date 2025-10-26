use std::error::Error;

mod chip8;
mod cli;
mod cpu;
mod emulate;
mod ext;
mod keyboard;
mod tests;
fn main() -> Result<(), Box<dyn Error>> {
    if let Some(args) = cli::parse_args() {
        match args {
            cli::Chip8Command::Emulate { src, debug } => {
                let mut chip8 = chip8::Chip8::new(debug);
                emulate::emulate(src, &mut chip8)?;
            }
            cli::Chip8Command::PrintKeyMap => {
                print!(
                    "Keymap: (Chip8 key -> KeyBoard Key)

                    ╔════════╦════════╦════════╦════════╗
                    ║ 1 -> 1 ║ 2 -> 2 ║ 3 -> 3 ║ C -> 4 ║
                    ╠════════╬════════╬════════╬════════╣
                    ║ 4 -> Q ║ 5 -> W ║ 6 -> E ║ D -> R ║
                    ╠════════╬════════╬════════╬════════╣
                    ║ 7 -> A ║ 8 -> S ║ 9 -> D ║ E -> F ║
                    ╠════════╬════════╬════════╬════════╣
                    ║ A -> Z ║ 0 -> X ║ B -> C ║ F -> V ║
                    ╚════════╩════════╩════════╩════════╝

                    "
                )
            }
        }
    }

    Ok(())
}
