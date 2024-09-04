use rand::{thread_rng, Rng};

use crate::cpu::{Cpu, ExecuteError, HEIGHT, WIDTH};

#[test]
fn instruction_0x00e0() {
	// instruction == 0x00e0
	// clear the screen

	let mut cpu = Cpu::init();
	let res = cpu.add_program(&vec![0x00, 0xe0]);
	assert!(res.is_ok(), "Should be able to add the program.");
	// add dummy data to the d_buffer
	{
		let mut d_buffer = cpu.d_buffer.borrow_mut();
		let mut rng = thread_rng();
		let data = (0..WIDTH*HEIGHT).map(|_| rng.gen_range(0..=1)).collect::<Vec<u8>>();
		d_buffer.copy_from_slice(&data);
	}
	let exec_res = cpu.step();
	match exec_res {
		Ok(()) => {},
		Err(e) => {
			cpu.dump(true, 6);
			panic!("Failed to execute instruction {:?}", e);
		}
	}

	let d_buffer = cpu.d_buffer.borrow();
	if d_buffer.contains(&0x1u8) {
		cpu.dump(true, 0);
		panic!("After clearing the d_buffer shouldn't contain any set bit");
	}
}

#[test]
fn instruction_0x00ee_0x2nnn() {
	// instruction == 0x00ee
	// return from a subroutine
	// instruction == 0x2nnn
	// execute subroutine starting at address NNN
	let mut cpu = Cpu::init();
	// program -> 0x2208, 0x0111, 0x0111, 0x0111, 0x0111, 0x00ee
	//              ^        ^      ^       ^       ^       ^
	// address -> 0x0200, 0x0202, 0x0204, 0x0206, 0x0208, 0x0210
	// we call the procedure at 0x208 (first instruction is nop)
	// the 0x0210 contains 0x00ee so the pc should go to 0x0202.
	let program : Vec<u8> = vec![0x22, 0x08, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11, 0x00, 0xee];

	cpu.add_program(&program).expect("Should be able to add the program");
	cpu.step().expect("Should execute 0x2NNN (call).");
	let stack = &cpu.stack;
	assert_eq!(stack[0], 0x200, "Stack should contain 0x200");
	assert_eq!(cpu.pc, 0x208, "pc should be 0x208 after call");
	cpu.step().expect("Should execute 0x0111 nop");
	cpu.step().expect("Should execute 0x00ee ret");
	assert_eq!(cpu.pc, 0x0202, "Should go back to 0x0202 after returning");
}

#[test]
fn instruction_0x1nnn() {
	// instruction == 0x1NNN
	// jump to address NNN
	// will return error if the address is out of program bounds
	let mut cpu = Cpu::init();

	// program -> 0x0111, 0x120a, 0x0111, 0x0111, 0x0111, 0x0111
	//              ^       ^       ^       ^       ^       ^
	// address -> 0x0200, 0x0202, 0x0204, 0x0206, 0x0208, 0x020a
	// should jump to 0x020a
	let mut program : Vec<u8> = vec![0x01, 0x11, 0x12, 0x0a, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11];

	cpu.add_program(&program).expect("Should be able to add the program");
	cpu.step().expect("should execute nop");
	cpu.step().expect("should execute jump (0x1NNN)");

	assert_eq!(cpu.pc, 0x020a, "pc should be 0x020a after jump");

	// test bad jump (jump to 0x02ff)
	program[3] = 0xff;
	cpu.reset();
	cpu.add_program(&program).expect("Should be able to add the program");
	cpu.step().expect("should execute nop");
	let err = cpu.step().expect_err("should panic because jump to 0x02ff");
	match err {
		ExecuteError::BadJumpAddr(instruction) => {
			assert_eq!(instruction, 0x12ff, "Wrong instruction returned");
		}
		_ => {
			unreachable!("Bad jump always returns ExecuteError::BadJumpAddr");
		}
	}
}


#[test]
fn instruction_0x3xnn(){
	// instruction == 0x3xnn
	// skip the following instruction if VX == NN
	let mut cpu = Cpu::init();
	cpu.gp_registers[0xa] = 0xfb;
	// program -> 0x3afb, 0x120a, 0x0111, 0x3afc, 0x0111, 0x0111 	
	//              ^       ^       ^       ^       ^       ^
	// address -> 0x0200, 0x0202, 0x0204, 0x0206, 0x0208, 0x020a
	// should skip instruction at 0x0202
	let program : Vec<u8> = vec![0x3a, 0xfb, 0x12, 0x0a, 0x01, 0x11, 0x3a, 0xfc, 0x01, 0x11, 0x01, 0x11];
	cpu.add_program(&program).expect("should be able to add the program");

	cpu.step().expect("should execute instruction (0x3XNN)");
	assert_eq!(cpu.pc, 0x0204, "should skip the instruction at 0x0202");

	cpu.step().expect("should execute nop");
	cpu.step().expect("should execute instruction (0x3XNN)");
	assert_eq!(cpu.pc, 0x0208, "should not skip the instruction at 0x0208");
}

#[test]
fn instruction_0x4xnn(){
	// instruction == 0x4xnn
	// skip the following instruction if VX != NN
	let mut cpu = Cpu::init();
	cpu.gp_registers[0xa] = 0xfb;
	// program -> 0x4afb, 0x0111, 0x4afc, 0x0111, 0x0111 	
	//              ^       ^       ^       ^       ^
	// address -> 0x0200, 0x0202, 0x0204, 0x0206, 0x0208
	// should not skip instruction at 0x0202
	let program : Vec<u8> = vec![0x4a, 0xfb, 0x01, 0x11, 0x4a, 0xfc, 0x01, 0x11, 0x01, 0x11];
	cpu.add_program(&program).expect("should be able to add the program");

	cpu.step().expect("should execute instruction (0x4XNN)");
	assert_eq!(cpu.pc, 0x0202, "should not skip the instruction at 0x0202");

	cpu.step().expect("should execute nop");
	cpu.step().expect("should execute instruction (0x4XNN)");
	assert_eq!(cpu.pc, 0x0208, "should skip the instruction at 0x0206");
}

#[test]
fn instruction_0x5xy0() {
	// instruction === 0x5XY0
	// skip the following instruction if the value of VX == VY

	let mut cpu = Cpu::init();
	// should return error if (instruction & 0x000f) != 0
	let program: Vec<u8> = vec![0x5a, 0x51];
	cpu.add_program(&program).expect("should be able to add the program");
	cpu.step().expect_err("should return an error for bad instruction");

	cpu.reset();
	cpu.gp_registers[0xa] = 0xfb;
	cpu.gp_registers[0xb] = 0xfb;
	let program: Vec<u8> = vec![0x5a, 0xb0, 0x01, 0x11, 0x4a, 0xfc];
	// program -> 0x5ab0, 0x0111, 0x4afc
	//              ^       ^       ^    
	// address -> 0x0200, 0x0202, 0x0204
	// should skip instruction at 0x0202

	cpu.add_program(&program).expect("should be able to add the program");
	cpu.step().expect("should execute instruction (0x5XY0)");
	assert_eq!(cpu.pc, 0x0204, "should skip instruction at 0x0202");

	cpu.reset();
	cpu.gp_registers[0xa] = 0xfa;
	cpu.gp_registers[0xb] = 0xfb;
	let program: Vec<u8> = vec![0x5a, 0xb0, 0x01, 0x11, 0x4a, 0xfc];
	// program -> 0x5ab0, 0x0111, 0x4afc
	//              ^       ^       ^    
	// address -> 0x0200, 0x0202, 0x0204
	// should not skip instruction at 0x0202

	cpu.add_program(&program).expect("should be able to add the program");
	cpu.step().expect("should execute instruction (0x5XY0)");
	assert_eq!(cpu.pc, 0x0202, "should not skip instruction at 0x0202");
}