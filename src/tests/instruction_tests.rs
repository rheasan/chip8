use rand::{thread_rng, Rng};

use crate::{
    cpu::{Cpu, ExecuteError, HEIGHT, WIDTH},
    keyboard::KeyBoard,
};

const KEY_PRESSED: KeyBoard = KeyBoard {
    key_pressed: Some(0u8),
};

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
        let data = (0..WIDTH * HEIGHT)
            .map(|_| rng.gen_range(0..=1))
            .collect::<Vec<u8>>();
        d_buffer.copy_from_slice(&data);
    }
    let exec_res = cpu.step(&KEY_PRESSED);
    match exec_res {
        Ok(()) => {}
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
    let program: Vec<u8> = vec![
        0x22, 0x08, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11, 0x00, 0xee,
    ];

    cpu.add_program(&program)
        .expect("Should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("Should execute 0x2NNN (call).");
    let stack = &cpu.stack;
    assert_eq!(stack[0], 0x200, "Stack should contain 0x200");
    assert_eq!(cpu.pc, 0x208, "pc should be 0x208 after call");
    cpu.step(&KEY_PRESSED).expect("Should execute 0x0111 nop");
    cpu.step(&KEY_PRESSED).expect("Should execute 0x00ee ret");
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
    let mut program: Vec<u8> = vec![
        0x01, 0x11, 0x12, 0x0a, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11, 0x01, 0x11,
    ];

    cpu.add_program(&program)
        .expect("Should be able to add the program");
    cpu.step(&KEY_PRESSED).expect("should execute nop");
    cpu.step(&KEY_PRESSED)
        .expect("should execute jump (0x1NNN)");

    assert_eq!(cpu.pc, 0x020a, "pc should be 0x020a after jump");

    // test bad jump (jump to 0x02ff)
    program[3] = 0xff;
    cpu.reset();
    cpu.add_program(&program)
        .expect("Should be able to add the program");
    cpu.step(&KEY_PRESSED).expect("should execute nop");
    let err = cpu
        .step(&KEY_PRESSED)
        .expect_err("should panic because jump to 0x02ff");
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
fn instruction_0x3xnn() {
    // instruction == 0x3xnn
    // skip the following instruction if VX == NN
    let mut cpu = Cpu::init();
    cpu.gp_registers[0xa] = 0xfb;
    // program -> 0x3afb, 0x120a, 0x0111, 0x3afc, 0x0111, 0x0111
    //              ^       ^       ^       ^       ^       ^
    // address -> 0x0200, 0x0202, 0x0204, 0x0206, 0x0208, 0x020a
    // should skip instruction at 0x0202
    let program: Vec<u8> = vec![
        0x3a, 0xfb, 0x12, 0x0a, 0x01, 0x11, 0x3a, 0xfc, 0x01, 0x11, 0x01, 0x11,
    ];
    cpu.add_program(&program)
        .expect("should be able to add the program");

    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x3XNN)");
    assert_eq!(cpu.pc, 0x0204, "should skip the instruction at 0x0202");

    cpu.step(&KEY_PRESSED).expect("should execute nop");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x3XNN)");
    assert_eq!(cpu.pc, 0x0208, "should not skip the instruction at 0x0208");
}

#[test]
fn instruction_0x4xnn() {
    // instruction == 0x4xnn
    // skip the following instruction if VX != NN
    let mut cpu = Cpu::init();
    cpu.gp_registers[0xa] = 0xfb;
    // program -> 0x4afb, 0x0111, 0x4afc, 0x0111, 0x0111
    //              ^       ^       ^       ^       ^
    // address -> 0x0200, 0x0202, 0x0204, 0x0206, 0x0208
    // should not skip instruction at 0x0202
    let program: Vec<u8> = vec![0x4a, 0xfb, 0x01, 0x11, 0x4a, 0xfc, 0x01, 0x11, 0x01, 0x11];
    cpu.add_program(&program)
        .expect("should be able to add the program");

    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x4XNN)");
    assert_eq!(cpu.pc, 0x0202, "should not skip the instruction at 0x0202");

    cpu.step(&KEY_PRESSED).expect("should execute nop");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x4XNN)");
    assert_eq!(cpu.pc, 0x0208, "should skip the instruction at 0x0206");
}

#[test]
fn instruction_0x5xy0() {
    // instruction === 0x5XY0
    // skip the following instruction if the value of VX == VY

    let mut cpu = Cpu::init();
    // should return error if (instruction & 0x000f) != 0
    let program: Vec<u8> = vec![0x5a, 0x51];
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect_err("should return an error for bad instruction");

    cpu.reset();
    cpu.gp_registers[0xa] = 0xfb;
    cpu.gp_registers[0xb] = 0xfb;
    let program: Vec<u8> = vec![0x5a, 0xb0, 0x01, 0x11, 0x4a, 0xfc];
    // program -> 0x5ab0, 0x0111, 0x4afc
    //              ^       ^       ^
    // address -> 0x0200, 0x0202, 0x0204
    // should skip instruction at 0x0202

    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x5XY0)");
    assert_eq!(cpu.pc, 0x0204, "should skip instruction at 0x0202");

    cpu.reset();
    cpu.gp_registers[0xa] = 0xfa;
    cpu.gp_registers[0xb] = 0xfb;
    let program: Vec<u8> = vec![0x5a, 0xb0, 0x01, 0x11, 0x4a, 0xfc];
    // program -> 0x5ab0, 0x0111, 0x4afc
    //              ^       ^       ^
    // address -> 0x0200, 0x0202, 0x0204
    // should not skip instruction at 0x0202

    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x5XY0)");
    assert_eq!(cpu.pc, 0x0202, "should not skip instruction at 0x0202");
}

#[test]
fn instruction_0x6xnn() {
    // instruction == 0x6XNN
    // store number nn in register VX

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x6a, 0x51, 0x01, 0x11];

    // program -> 0x6a51, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202

    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x6XNN)");

    assert_eq!(
        cpu.gp_registers[0xa], 0x51,
        "Register 0xa should contain 0x51"
    );
}

#[test]
fn instruction_0x7xnn() {
    // instruction == 0x7XNN
    // add value NN to register VX (wrapping addition)
    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x7a, 0xff, 0x01, 0x11];
    cpu.gp_registers[0xa] = 0xff;

    // program -> 0x7aff, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202

    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x7XNN)");

    assert_eq!(
        cpu.gp_registers[0xa], 0xfe,
        "Register 0xa should contain 0xfe"
    );
}

#[test]
fn instruction_0x8xy0() {
    // instruction == 0x8XY0
    // store value of VY in VX

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8b, 0xa0, 0x01, 0x11];
    cpu.gp_registers[0xa] = 0xff;

    // program -> 0x8ba0, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY0)");

    assert_eq!(
        cpu.gp_registers[0xa], cpu.gp_registers[0xb],
        "Registers 0xa and 0xb should have same value"
    );
}

#[test]
fn instruction_0x8xy1() {
    // instruction == 0x8XY1
    // set VX = VX | VY

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xb1, 0x01, 0x11];
    cpu.gp_registers[0xa] = 0xf0;
    cpu.gp_registers[0xb] = 0x0f;

    // program -> 0x8ab1, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY1)");

    assert_eq!(cpu.gp_registers[0xa], 0xff, "Register ");
}

#[test]
fn instruction_0x8xy2() {
    // instruction == 0x8XY2
    // set VX = VX & VY

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xb2, 0x01, 0x11];
    cpu.gp_registers[0xa] = 0xf0;
    cpu.gp_registers[0xb] = 0x0f;

    // program -> 0x8ab2, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY2)");

    assert_eq!(cpu.gp_registers[0xa], 0x0);
}

#[test]
fn instruction_0x8xy3() {
    // instruction == 0x8XY3
    // set VX = VX ^ VY
    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xb5, 0x01, 0x11];
    cpu.gp_registers[0xa] = 0xff;
    cpu.gp_registers[0xb] = 0x0f;

    // program -> 0x8ab3, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY3)");

    assert_eq!(cpu.gp_registers[0xa], 0xf0);
}

#[test]
fn instruction_0x8xy4() {
    // instruction == 0x8XY4
    // set VX = VX + VY. set VF = 0x01 if carry occurs, otherwise set VF = 0x00

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xb4, 0x01, 0x11];

    // for no carry
    cpu.gp_registers[0xa] = 0xa;
    cpu.gp_registers[0xb] = 0xb;

    // program -> 0x8ab4, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY4)");

    assert_eq!(cpu.gp_registers[0xa], 0x15);
    assert_eq!(cpu.gp_registers[0xf], 0x0);

    cpu.reset();
    cpu.add_program(&program)
        .expect("should be able to add the program");
    // for carry
    cpu.gp_registers[0xa] = 0xfa;
    cpu.gp_registers[0xb] = 0xfa;

    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY4)");
    assert_eq!(cpu.gp_registers[0xa], 0xf5);
    assert_eq!(cpu.gp_registers[0xf], 0x1);
}

#[test]
fn instruction_0x8xy5() {
    // instruction == 0x8XY5
    // set VX = VX - VY. set VF = 0x00 if borrow occurs, otherwise set VF = 0x01

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xb5, 0x01, 0x11];

    // for no borrow
    cpu.gp_registers[0xa] = 0xa;
    cpu.gp_registers[0xb] = 0x8;

    // program -> 0x8ab5, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY5)");

    assert_eq!(cpu.gp_registers[0xa], 0x2);
    assert_eq!(cpu.gp_registers[0xf], 0x01);

    cpu.reset();
    cpu.add_program(&program)
        .expect("should be able to add the program");
    // for borrow
    cpu.gp_registers[0xa] = 0x8;
    cpu.gp_registers[0xb] = 0xb;

    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY5)");
    assert_eq!(cpu.gp_registers[0xa], 0xfd);
    assert_eq!(cpu.gp_registers[0xf], 0x00);
}

#[test]
fn instruction_0x8xy6() {
    // instruction == 0x8XY6
    // set VX = VY >> 1, set VF to the least significant bit of VY before shift. VY is unchanged

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xb6, 0x01, 0x11];

    cpu.gp_registers[0xb] = 0x2;

    // program -> 0x8ab6, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY6)");

    assert_eq!(cpu.gp_registers[0xf], 0x0);
    assert_eq!(cpu.gp_registers[0xa], 0x1);
    assert_eq!(cpu.gp_registers[0xb], 0x2);
}

#[test]
fn instruction_0x8xy7() {
    // instruction == 0x8XY7
    // set VX = VY - VX. set VF = 0x00 if borrow occcurs, otherwise set VF = 0x01

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xb7, 0x01, 0x11];

    // for no borrow
    cpu.gp_registers[0xa] = 0x8;
    cpu.gp_registers[0xb] = 0xa;

    // program -> 0x8ab7, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY7)");

    assert_eq!(cpu.gp_registers[0xa], 0x2);
    assert_eq!(cpu.gp_registers[0xf], 0x01);

    cpu.reset();
    cpu.add_program(&program)
        .expect("should be able to add the program");
    // for borrow
    cpu.gp_registers[0xa] = 0x2;
    cpu.gp_registers[0xb] = 0x1;

    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XY7)");
    assert_eq!(cpu.gp_registers[0xa], 0xff);
    assert_eq!(cpu.gp_registers[0xf], 0x00);
}

#[test]
fn instruction_8xye() {
    // instruction == 0x8XYE
    // set VX = VY << 1, set VF to the most significant bit of VY before shift. VY is unchanged

    let mut cpu = Cpu::init();
    let program: Vec<u8> = vec![0x8a, 0xbe, 0x01, 0x11];

    cpu.gp_registers[0xb] = 0x2;

    // program -> 0x8abe, 0x0111
    //              ^       ^
    // address -> 0x0200, 0x0202
    cpu.add_program(&program)
        .expect("should be able to add the program");
    cpu.step(&KEY_PRESSED)
        .expect("should execute instruction (0x8XYE)");

    assert_eq!(cpu.gp_registers[0xf], 0x0);
    assert_eq!(cpu.gp_registers[0xa], 0x4);
    assert_eq!(cpu.gp_registers[0xb], 0x2);
}
