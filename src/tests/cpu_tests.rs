use rand::{Rng, RngCore};
use crate::cpu::Cpu;
#[test]
fn large_program_fails() {
	let mut chip8 = Cpu::init();
	let large_data = vec![0u8; 3216];
	let res = chip8.add_program(&large_data);
	assert!(res.is_err());
}
#[test]
fn program_init_success() {
	let mut chip8 = Cpu::init();
	let mut rng = rand::thread_rng();

	let mut program : Vec<u8> = vec![0u8; rng.gen_range(0..=800)];
	rng.fill_bytes(&mut program);

	let res = chip8.add_program(&program);
	assert!(res.is_ok());
	let mem = &chip8.mem;
	// 0..200 bytes are reserverd for the emulator. program starts from 0x200
	for i in 0..0x200 {
		assert_eq!(mem[i], 0u8);
	}

	for i in 0x200..=program.len() {
		assert_eq!(mem[i], program[i - 0x200]);
	}

}