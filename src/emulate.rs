use crate::chip8::Chip8;
use std::fs::read;

fn read_bytecode(path: &String) -> Result<Vec<u8>, std::io::Error> {
    let data = read(path)?;

    if data.len() % 2 != 0 {
        dbg!("Program file has odd number of bytes");
    }

    Ok(data)
}

pub fn emulate(src: String, chip8: &mut Chip8) -> Result<(), std::io::Error> {
    let data = read_bytecode(&src)?;
    chip8.add_program(&data)?;
    chip8.run();
    Ok(())
}
