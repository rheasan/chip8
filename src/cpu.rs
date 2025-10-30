use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::keyboard::KeyBoard;

// 0x2000 - 0x200 = 0x1EB4 = 7860 bytes
pub const MAX_PROGRAM_SIZE: usize = 7860usize;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const ZERO: &[u8] = &[0xf0, 0x90, 0x90, 0x90, 0xf0];
const ONE: &[u8] = &[0x20, 0x60, 0x20, 0x20, 0x70];
const TWO: &[u8] = &[0xf0, 0x10, 0xf0, 0x80, 0xf0];
const THREE: &[u8] = &[0xf0, 0x10, 0xf0, 0x10, 0xf0];
const FOUR: &[u8] = &[0xf0, 0x80, 0xf0, 0x10, 0xf0];
const FIVE: &[u8] = &[0xf0, 0x80, 0xf0, 0x90, 0xf0];
const SIX: &[u8] = &[0xf0, 0x80, 0xf0, 0x90, 0xf0];
const SEVEN: &[u8] = &[0xf0, 0x10, 0x20, 0x40, 0x40];
const EIGHT: &[u8] = &[0xf0, 0x90, 0xf0, 0x10, 0xf0];
const NINE: &[u8] = &[0xf0, 0x90, 0xf0, 0x10, 0xf0];
const A: &[u8] = &[0xf0, 0x90, 0xf0, 0x90, 0x90];
const B: &[u8] = &[0xe0, 0x90, 0xe0, 0x90, 0xe0];
const C: &[u8] = &[0xf0, 0x80, 0x80, 0x80, 0xf0];
const D: &[u8] = &[0xe0, 0x90, 0x90, 0x90, 0xe0];
const E: &[u8] = &[0xf0, 0x80, 0xf0, 0x80, 0xf0];
const F: &[u8] = &[0xf0, 0x80, 0xf0, 0x80, 0x80];
const HEX_SPRITE_SIZE: u16 = 0x5;

pub struct Cpu {
    pub mem: Vec<u8>,
    pub d_buffer: Rc<RefCell<Vec<u8>>>,
    // general purpose registers V0 to VF, 8bits wide
    pub gp_registers: [u8; 16],
    // address register 'I', 16bit wide but addresses are only 12bit wide
    // only addresses in the range 0x200 - 0xE8F are available for programs
    // first 0x200 bytes are reserved for the interpreter, and final 352 bytes are reserved for
    // "variables and display refresh"
    pub i: u16,
    pub pc: usize,
    pub sp: u8,
    pub stack: Vec<usize>,
    pub delay_timer: (u8, Instant),
    pub sound_timer: (u8, Instant),
    pub program_end_addr: usize,
    pub debug: bool,
}
impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("gp_registers", &self.gp_registers)
            .field("i", &self.i)
            .field("pc", &self.pc)
            .field("sp", &self.sp)
            .field("stack", &self.stack)
            .field("delay_timer", &self.delay_timer)
            .field("sound_timer", &self.sound_timer)
            .field("program_end_addr", &self.program_end_addr)
            .finish()
    }
}

#[derive(Debug)]
pub enum ExecuteError {
    FailedToReadInstruction,
    BadInstruction(u16),
    MaxCallDepthReached(u16),
    BadReturn(u16),
    BadJumpAddr(u16),
}
impl ToString for ExecuteError {
    fn to_string(&self) -> String {
        match self {
            ExecuteError::FailedToReadInstruction => String::from("Failed to read instructions"),
            ExecuteError::BadInstruction(i) => format!("Bad instruction: {:#04}", i),
            ExecuteError::BadJumpAddr(addr) => format!("Bad Jump Address: {:#04}", addr),
            ExecuteError::BadReturn(addr) => format!("Bad return Address: {:#04}", addr),
            ExecuteError::MaxCallDepthReached(i) => format!("Max call depth reached: {:#04}", i),
        }
    }
}

impl Cpu {
    pub fn init(debug: bool) -> Self {
        let sprites = vec![
            ZERO, ONE, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE, A, B, C, D, E, F,
        ]
        .concat();
        let mut cpu = Cpu {
            mem: vec![0; 8192],
            d_buffer: Rc::new(RefCell::new(vec![0u8; WIDTH * HEIGHT])),
            gp_registers: [0u8; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: Vec::new(),
            delay_timer: (0, Instant::now()),
            sound_timer: (0, Instant::now()),
            program_end_addr: 0,
            debug: debug,
        };
        // add sprites to the start of the memory
        cpu.mem[0..sprites.len()].copy_from_slice(&sprites);
        cpu
    }
    pub fn add_program(&mut self, program: &[u8]) -> Result<(), std::io::Error> {
        if program.len() > MAX_PROGRAM_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Max supported program size if {} bytes. Received {} bytes",
                    MAX_PROGRAM_SIZE,
                    program.len()
                ),
            ));
        }
        self.mem[512..(program.len() + 512)].copy_from_slice(&program[..]);
        self.program_end_addr = 0x200 + program.len();

        Ok(())
    }
    pub fn reset(&mut self) {
        self.i = 0;
        self.pc = 0x0200;
        self.sp = 0;
        self.stack.clear();
        self.delay_timer = (0, Instant::now());
        self.sound_timer = (0, Instant::now());
        self.mem[0x200..=self.program_end_addr].fill(0);
    }

    pub fn dump(&self, dump_d_buffer: bool, program_bytes: usize) {
        println!("State: {:?}", self);

        if dump_d_buffer {
            println!("\nDisplay buffer: ");
            let d_buffer = self.d_buffer.borrow();
            for (i, e) in d_buffer.iter().enumerate() {
                print!("{}", e & 1);
                if i != 0 && (i + 1) % WIDTH == 0 {
                    println!();
                }
            }
        }

        if program_bytes > 0 {
            println!(
                "\n Loaded Program: (total program length {} bytes)",
                self.program_end_addr - 0x200
            );
            for i in &self.mem[0x200..program_bytes + 0x200] {
                println!("{:#04x}", i);
            }
        }
    }

    pub fn dump_everything(&self) {
        self.dump(true, self.program_end_addr - 0x200);
    }
    // Returns Ok(true) if d_buffer was updated
    pub fn step(&mut self, keyboard: &KeyBoard) -> Result<bool, ExecuteError> {
        let instruction = self.get_next_instruction()?;
        let nnn = (instruction & 0x0fff) as usize;
        // x only contains 4 bits so can access gp_registers without bound check
        let x = ((instruction & 0x0f00) >> 8) as usize;
        let y = ((instruction & 0x00f0) >> 4) as usize;
        let nn = (instruction & 0x00ff) as u8;
        let n = (instruction & 0x000f) as u8;

        let changed = Cpu::handle_timer(&mut self.sound_timer);

        let _ = Cpu::handle_timer(&mut self.delay_timer);

        match instruction & 0xf000 {
            0x0000 => {
                if ((instruction & 0x0f00) >> 8) != 0 {
                    // instruction == 0x0NNN
                    // execute machine language subroutine at addr NNN
                    // this instruction is only on RCA COSMAC VIP (the original implementation of chip8)
                    // so ignore this instruction
                    self.pc += 2;
                } else {
                    // instruction == 0x00E(0|E)
                    match instruction & 0x00ff {
                        0xe0 => {
                            if self.debug {
                                println!("cls");
                            }
                            // instruction == 0x00E0
                            // clear the screen
                            self.d_buffer.borrow_mut().fill(0);
                            self.pc += 2;
                        }
                        0xee => {
                            if self.debug {
                                println!("ret");
                            }
                            // instruction == 0x00EE
                            // return from a subroutine
                            if self.stack.is_empty() {
                                return Err(ExecuteError::BadReturn(instruction));
                            }
                            let addr = self.stack.pop().unwrap();
                            if !self.is_valid_program_addr(addr) {
                                return Err(ExecuteError::BadJumpAddr(instruction));
                            }
                            // the returned address will be the instruction calling the subroutine so skip it
                            self.pc = addr + 2;
                        }
                        _ => {
                            return Err(ExecuteError::BadInstruction(instruction));
                        }
                    }
                }
            }
            0x1000 => {
                // instruction == 0x1NNN
                // jump to address NNN
                if self.debug {
                    println!("jmp {:x}", nnn);
                }
                if !self.is_valid_program_addr(nnn) {
                    return Err(ExecuteError::BadJumpAddr(instruction));
                }
                self.pc = nnn;
            }
            0x2000 => {
                // instruction == 0x2NNN
                // execute subroutine starting at address NNN
                if self.debug {
                    println!("call {:x}", nnn);
                }
                if self.stack.len() == 16 {
                    return Err(ExecuteError::MaxCallDepthReached(instruction));
                }
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            0x3000 => {
                // instruction == 0x3XNN
                // skip the following instruction if the value of VX == NN
                if self.debug {
                    println!("se v{:x} {:x} (vx = {:x})", x, nn, self.gp_registers[x]);
                }
                if self.gp_registers[x] == nn {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x4000 => {
                // instruction == 0x4XNN
                // skip the following instruction if the value of VX != NN
                if self.debug {
                    println!("sne v{:x} {:x}", x, nn);
                }
                if self.gp_registers[x] != nn {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x5000 => {
                if instruction & 0x000f != 0 {
                    return Err(ExecuteError::BadInstruction(instruction));
                }
                // instruction === 0x5XY0
                // skip the following instruction if the value of VX == VY
                if self.debug {
                    println!(
                        "se v{:x} v{:x} (vx = {:x} vy = {:x})",
                        x, y, self.gp_registers[x], self.gp_registers[y]
                    );
                }
                if self.gp_registers[x] == self.gp_registers[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x6000 => {
                // instruction == 0x6XNN
                // store number nn in register VX
                if self.debug {
                    println!("ld v{:x} {:x}", x, nn);
                }
                self.gp_registers[x] = nn;
                self.pc += 2;
            }
            0x7000 => {
                // instruction == 0x7XNN
                // add value NN to register VX (wrapping addition)
                if self.debug {
                    println!("add v{:x} {:x} (vx = {:x})", x, nn, self.gp_registers[x]);
                }
                self.gp_registers[x] = self.gp_registers[x].wrapping_add(nn);
                self.pc += 2;
            }
            0x8000 => {
                match n {
                    0x0 => {
                        // instruction == 0x8XY0
                        // store value of VY in VX
                        if self.debug {
                            println!("ld v{:x} v{:x}", x, y);
                        }
                        self.gp_registers[x] = self.gp_registers[y];
                        self.pc += 2;
                    }
                    0x1 => {
                        // instruction == 0x8XY1
                        // set VX = VX | VY
                        if self.debug {
                            println!("or v{:x} v{:x}", x, y);
                        }
                        self.gp_registers[x] |= self.gp_registers[y];
                        self.pc += 2;
                    }
                    0x2 => {
                        // instruction == 0x8XY2
                        // set VX = VX & VY
                        if self.debug {
                            println!("and v{:x} v{:x}", x, y);
                        }
                        self.gp_registers[x] &= self.gp_registers[y];
                        self.pc += 2;
                    }
                    0x3 => {
                        // instruction == 0x8XY3
                        // set VX = VX ^ VY
                        if self.debug {
                            println!("xor v{:x} v{:x}", x, y);
                        }
                        self.gp_registers[x] ^= self.gp_registers[y];
                        self.pc += 2;
                    }
                    0x4 => {
                        // instruction == 0x8XY4
                        // set VX = VX + VY. set VF = 0x01 if carry occurs, otherwise set VF = 0x00
                        if self.debug {
                            print!(
                                "add v{:x} v{:x} (vx = {:x} vy = {:x})",
                                x, y, self.gp_registers[x], self.gp_registers[y]
                            );
                        }
                        let t = self.gp_registers[x].checked_add(self.gp_registers[y]);
                        match t {
                            Some(val) => {
                                self.gp_registers[x] = val;
                                self.gp_registers[0xf] = 0;
                                if self.debug {
                                    println!(
                                        "    vf after add {:x} vx after add {:x}",
                                        self.gp_registers[0xf], self.gp_registers[x]
                                    );
                                }
                            }
                            None => {
                                // addition overflowed
                                let sum = self.gp_registers[x] as u16 + self.gp_registers[y] as u16;
                                self.gp_registers[x] = (sum & 0xFF) as u8; // wrap around correctly
                                self.gp_registers[0xf] = 1;
                                if self.debug {
                                    println!(
                                        "    vf after add {:x} vx after add {:x} overflowed",
                                        self.gp_registers[0xf], self.gp_registers[x]
                                    );
                                }
                            }
                        }
                        self.pc += 2;
                    }
                    0x5 => {
                        // instruction == 0x8XY5
                        // set VX = VX - VY. set VF = 0x00 if borrow occurs, otherwise set VF = 0x01
                        if self.debug {
                            print!("sub v{:x} v{:x}", x, y);
                        }

                        if self.gp_registers[y] <= self.gp_registers[x] {
                            self.gp_registers[0xf] = 0x1;
                        } else {
                            self.gp_registers[0xf] = 0x0;
                        }
                        if self.debug {
                            println!("    vf after sub {:x}", self.gp_registers[0xf]);
                        }
                        self.gp_registers[x] =
                            self.gp_registers[x].wrapping_sub(self.gp_registers[y]);

                        self.pc += 2;
                    }
                    0x6 => {
                        // instruction == 0x8XY6
                        // set VX = VY >> 1, set VF to the least significant bit of VY before shift. VY is unchanged
                        if self.debug {
                            println!("shr v{:x} v{:x}", x, y);
                        }
                        self.gp_registers[0xf] = self.gp_registers[y] & 0x1;
                        self.gp_registers[x] = self.gp_registers[y] >> 1;
                        self.pc += 2;
                    }
                    0x7 => {
                        // instruction == 0x8XY7
                        // set VX = VY - VX. set VF = 0x00 if borrow occcurs, otherwise set VF = 0x01
                        if self.debug {
                            print!("subn v{:x} v{:x}", x, y);
                        }
                        self.gp_registers[x] =
                            self.gp_registers[y].wrapping_sub(self.gp_registers[x]);

                        if self.gp_registers[x] <= self.gp_registers[y] {
                            self.gp_registers[0xf] = 0x1;
                        } else {
                            self.gp_registers[0xf] = 0x0;
                        }
                        if self.debug {
                            println!("    vf after subn {:x}", self.gp_registers[0xf]);
                        }
                        self.pc += 2;
                    }
                    0xE => {
                        // instruction == 0x8XYE
                        // set VX = VY << 1, set VF to the most significant bit of VY before shift. VY is unchanged
                        if self.debug {
                            println!(
                                "shl v{:x} v{:x} (vx = {:x}, vy = {:x})",
                                x, y, self.gp_registers[x], self.gp_registers[y]
                            );
                        }
                        self.gp_registers[0xf] = (self.gp_registers[y] & 0x80) >> 7;
                        self.gp_registers[x] = self.gp_registers[y] << 1;
                        self.pc += 2;
                    }
                    _ => {
                        return Err(ExecuteError::BadInstruction(instruction));
                    }
                }
            }
            0x9000 => {
                if (instruction & 0x000f) != 0 {
                    return Err(ExecuteError::BadInstruction(instruction));
                }
                // instruction == 0x9XY0
                // skip the following instruction if VX != VY
                if self.debug {
                    println!("sne v{:x} v{:x}", x, y);
                }
                if self.gp_registers[x] != self.gp_registers[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0xa000 => {
                // instruction == 0xANNN
                // store memory address NNN in I
                if self.debug {
                    println!("ld I, {:x}", nnn);
                }
                self.i = nnn as u16;
                self.pc += 2;
            }
            0xb000 => {
                // instruction == 0xBNNN
                // jump to address V0 + NNN
                if self.debug {
                    println!("jmp V0 {:x}", nnn);
                }
                self.pc = self.gp_registers[0] as usize + nnn;
                self.pc += 2;
            }
            0xc000 => {
                // instruction == 0xCXNN
                // set VX to random number with the mask NN
                if self.debug {
                    println!("rnd v{:x}, {:x}", x, nn);
                }
                let random = rand::random::<u8>();
                self.gp_registers[x] = random & nn;
                self.pc += 2;
            }
            0xd000 => {
                // instruction == 0xDXYN
                // draw a sprite at position VX and VY with N bytes of sprite data starting at
                // address stored in I.
                // Set VF = 0x01 if any set pixels are changed to unset, otherwise set VF = 0x00.
                if self.debug {
                    print!("drw v{:x} v{:x} {:x}", x, y, n);
                }

                if self.draw_sprite(n, self.gp_registers[x], self.gp_registers[y])? {
                    self.gp_registers[0xf] = 0x01;
                } else {
                    self.gp_registers[0xf] = 0x00;
                }
                if self.debug {
                    println!("    vf after drw {:x}", self.gp_registers[0xf]);
                }
                self.pc += 2;
            }
            0xe000 => {
                match instruction & 0xff {
                    0x9e => {
                        // instruction == 0xEX9E
                        // skip the following instruction if the key corresponding to the hex value in VX
                        // is pressed. do not wait for input
                        if self.debug {
                            println!("skp v{:x}", x);
                        }
                        match keyboard.get_current_key() {
                            Some(key) => {
                                if key == self.gp_registers[x] {
                                    self.pc += 4
                                }
                            }
                            None => self.pc += 2,
                        }
                    }
                    0xa1 => {
                        // instruction == 0xEXA1
                        // skip the following instruction if the key corresponding to the hex value in VX
                        // is not pressed. do not wait for input
                        if self.debug {
                            println!("sknp v{:x} (vx = {:x})", x, self.gp_registers[x]);
                        }
                        match keyboard.get_current_key() {
                            Some(key) => {
                                if self.debug {
                                    println!("pressed key = {:x}", key);
                                }
                                if key != self.gp_registers[x] {
                                    self.pc += 4
                                }
                            }
                            None => self.pc += 2,
                        }
                    }
                    _ => {
                        return Err(ExecuteError::BadInstruction(instruction));
                    }
                }
            }
            0xf000 => {
                match instruction & 0xff {
                    0x07 => {
                        // instruction == 0xFX07
                        // store current value of delay timer in VX
                        if self.debug {
                            println!("ld v{:x} dt {:x}", x, self.delay_timer.0);
                        }
                        self.gp_registers[x] = self.delay_timer.0;
                        self.pc += 2;
                    }
                    0x0A => {
                        // instruction == 0xFX0A
                        // wait for keypress and store the value of key in VX
                        if self.debug {
                            println!("ld v{:x} K", x);
                        }
                        // dont increment pc is there is no keypress
                        match keyboard.get_current_key() {
                            Some(k) => {
                                self.gp_registers[x] = k;
                                self.pc += 2;
                            }
                            None => {}
                        }
                    }
                    0x15 => {
                        // instruction == 0xFX15
                        // set the delay timer to the value of VX
                        if self.debug {
                            println!("ld dt v{:x}", x);
                        }
                        self.delay_timer = (self.gp_registers[x], Instant::now());
                        self.pc += 2;
                    }
                    0x18 => {
                        // instruction == 0xFX18
                        // set the sound timer to the value of VX
                        if self.debug {
                            println!("ld st v{:x}", x);
                        }
                        self.sound_timer = (self.gp_registers[x], Instant::now());
                        self.pc += 2;
                    }
                    0x1e => {
                        // instruction == 0xFX1E
                        // Add the value stored in VX to I
                        if self.debug {
                            println!("add I v{:x}", x);
                        }
                        self.i += self.gp_registers[x] as u16;
                        self.pc += 2;
                    }
                    0x29 => {
                        // instruction == 0xFX29
                        // set I to memory address of sprite data corresponding to the digit stored in register VX
                        if self.debug {
                            println!("ld f v{:x}", x);
                        }
                        if self.gp_registers[x] > 0xf {
                            return Err(ExecuteError::BadInstruction(instruction));
                        }
                        self.i = self.gp_registers[x] as u16 * HEX_SPRITE_SIZE;
                        self.pc += 2;
                    }
                    0x33 => {
                        // instruction == 0xFX33
                        // store the binary coded decimal equivalent of value in VX at addr I, I+1, I+2
                        // https://en.wikipedia.org/wiki/Binary-coded_decimal
                        if self.debug {
                            println!("ld b v{:x}", x);
                        }
                        let vx = self.gp_registers[x];
                        // TODO: bound check
                        let addr = self.i as usize;
                        self.mem[addr] = vx / 100;
                        self.mem[addr + 1] = (vx % 100) / 10;
                        self.mem[addr + 2] = vx % 10;
                        self.pc += 2;
                    }
                    0x55 => {
                        // instruction == 0xFX55
                        // store the values of registers V0 to VX inclusive to memory starting at address I.
                        // set I = I + X + 1 after saving.
                        if self.debug {
                            println!("ld [I] v{:x}", x);
                        }
                        let addr = self.i as usize;
                        self.mem[addr..=addr + x].copy_from_slice(&self.gp_registers[0..=x]);
                        self.i += (x + 1) as u16;
                        self.pc += 2;
                    }
                    0x65 => {
                        // instruction == 0xFX65
                        // fill V0 to VX inclusive with values stored at memory starting at address I.
                        // set I = I + X + 1 after filling.
                        if self.debug {
                            println!("ld v{:x} I", x);
                        }
                        let addr = self.i as usize;
                        self.gp_registers[0..=x].copy_from_slice(&self.mem[addr..=addr + x]);
                        self.i += (x + 1) as u16;
                        self.pc += 2;
                    }
                    _ => {
                        return Err(ExecuteError::BadInstruction(instruction));
                    }
                }
            }
            _ => {
                return Err(ExecuteError::BadInstruction(instruction));
            }
        }

        Ok(instruction & 0xd000 == 0xd000)
    }
    #[inline]
    fn is_valid_program_addr(&self, addr: usize) -> bool {
        addr >= 0x200 && addr <= self.program_end_addr
    }
    fn get_next_instruction(&self) -> Result<u16, ExecuteError> {
        let byte_1 = self.mem.get(self.pc);
        let byte_2 = self.mem.get(self.pc + 1);

        if byte_1.is_none() || byte_2.is_none() {
            return Err(ExecuteError::FailedToReadInstruction);
        }
        let instruction = (*byte_1.unwrap() as u16) << 8 | *byte_2.unwrap() as u16;
        Ok(instruction)
    }

    fn handle_timer(timer: &mut (u8, Instant)) -> bool {
        let (ref mut s_timer, ref mut last_op) = timer;
        if *s_timer == 0 {
            return false;
        }

        if Instant::now().duration_since(*last_op) >= Duration::from_micros(1_000_000 / 60) {
            *s_timer -= 1;
            *last_op = Instant::now();
            return true;
        }
        return false;
    }

    fn draw_sprite(&mut self, n: u8, x: u8, y: u8) -> Result<bool, ExecuteError> {
        // flag is set if is any set pixels are set to unset
        let mut should_set_flag = false;
        let sprite_start = self.i as usize;
        let sprite_end = sprite_start + n as usize;

        let mut coord_x = x as usize % WIDTH;
        let mut coord_y = y as usize % HEIGHT;

        // each byte in the display buffer corresponds to a pixel and a bit in the sprite
        // each sprite is always 1 byte wide and 1 to 15 pixels tall
        let mut d_buffer = self.d_buffer.borrow_mut();
        for byte in self.mem[sprite_start..sprite_end].iter_mut() {
            let b = byte;
            for _ in 0..8 {
                let index = coord_x + coord_y * WIDTH;
                let prev_value = d_buffer[index];
                // the sprite is drawn by xoring with the current value not by setting a new value
                d_buffer[index] ^= (*b & 0x80) >> 7;
                should_set_flag |= prev_value == 1 && d_buffer[index] == 0;

                coord_x += 1;
                *b <<= 1;
            }
            coord_y += 1;
            coord_x = x as usize % WIDTH;
        }

        Ok(should_set_flag)
    }
}
