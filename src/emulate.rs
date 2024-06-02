use std::fs::read;
use crate::cpu::Cpu;

fn read_bytecode(path: &String) -> Result<Vec<u8>, std::io::Error> {
	let data = read(path)?;

	if data.len() % 2 != 0 {
		return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
			 String::from("Program file contains a broken instruction. Each instruction should be 2 bytes wide")
		));
	}

	// only addresses in the range 0x200 - 0x69F are available for programs (1184 bytes)
	if data.len() > 1184 {
		return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
			String::from("Program is too large. Maximum allowed size is 1184 bytes")
		));
	}

	return Ok(data);
}

pub fn emulate(src: String, chip8: &mut Cpu) -> Result<(), std::io::Error> {
	let data = read_bytecode(&src)?;
	chip8.add_program(&data)?;
	Ok(())
}