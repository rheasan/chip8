use std::fs::read;
use crate::chip8::Chip8;

fn read_bytecode(path: &String) -> Result<Vec<u8>, std::io::Error> {
	let data = read(path)?;

	if data.len() % 2 != 0 {
		return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
			 String::from("Program file contains a broken instruction. Each instruction should be 2 bytes wide")
		));
	}

	Ok(data)
}

pub fn emulate(src: String, chip8: &mut Chip8) -> Result<(), std::io::Error> {
	let data = read_bytecode(&src)?;
	chip8.add_program(&data)?;
	chip8.run();
	Ok(())
}