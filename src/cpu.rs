use std::{cell::RefCell, rc::Rc, thread::sleep, time::Duration};

// 0xE8F - 0x200 = 0xC8F = 3215 bytes
const MAX_PROGRAM_SIZE : usize = 3215usize;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Cpu {
    mem: Vec<u8>,
    d_buffer: Rc<RefCell<Vec<u8>>>,
    // general purpose registers V0 to VF, 8bits wide
    gp_registers: [u8; 16],
    // address register 'I', 16bit wide but addresses are only 12bit wide
	// only addresses in the range 0x200 - 0xE8F are available for programs
	// first 0x200 bytes are reserved for the interpreter, and final 352 bytes are reserved for 
	// "variables and display refresh"
	i: u16,
	pc: usize,
	sp: u8,
	stack: Vec<usize>,
    delay_timer: u8,
    sound_timer: u8,
	program_end_addr: usize,

	// FIXME: this state should not be owned by the CPU
	is_key_pressed: bool,
	last_key_pressed: u8
}
pub enum ExecuteError{
	FailedToReadInstruction,
	BadInstruction,
	MaxCallDepthReached,
	BadReturn,
	BadJumpAddr,
	InvalidSprite
}

impl Cpu {
    pub fn init() -> Self {
        return Cpu {
            mem: vec![0u8; 4096],
            d_buffer: Rc::new(RefCell::new(vec![0u8; WIDTH*HEIGHT])),
            gp_registers: [0u8; 16],
			i: 0,
			pc: 0x200,
			sp: 0,
			stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
			program_end_addr: 0,
			is_key_pressed: false,
			last_key_pressed: 0x0
        };
    }
	pub fn add_program(&mut self, program: &Vec<u8>) -> Result<(), std::io::Error> {
		if program.len() > MAX_PROGRAM_SIZE {
			return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, 
				format!("Max supported program size if 1183 bytes. Received {} bytes", program.len())));
		}

		for i in 0..program.len() {
			self.mem[200 + i] = program[i];
		}
		self.program_end_addr = 200 + program.len();
		
		Ok(())
	}
	pub fn get_mem(self: &Self) -> &Vec<u8> {
		&self.mem
	}
	pub fn step(&mut self) -> Result<(), ExecuteError> {
		let instruction = self.get_next_instruction()?;
		let nnn = (instruction & 0x0fff) as usize;
		// x only contains 4 bits so can access gp_resgisters without bound check
		let x = (instruction & 0x0f00 >> 8) as usize;
		let y = (instruction & 0x00f0 >> 4) as usize;
		let nn = (instruction & 0x00ff) as u8;
		let n = (instruction & 0x000f) as u8;

		match instruction & 0xf000 {
			0x0000 => {
				if ((instruction & 0x0f00) >> 8) != 0 {
					// instruction == 0x0NNN
					// execute machine language subroutine at addr NNN
					// this instruction is only on RCA COSMAC VIP (the original implementation of chip8)
					// so ignore this instruction
					()
				}
				else {
					// instruction == 0x00E(0|E)
					match instruction & 0x00ff {
						0xe0 => {
							// instruction == 0x00E0
							// clear the screen
							self.d_buffer.borrow_mut().fill(0);
						}
						0xee => {
							// instruction == 0x00EE
							// return from a subroutine
							if self.stack.len() == 0 {
								return Err(ExecuteError::BadReturn);
							}
							let addr = self.stack.pop().unwrap();
							if !self.is_valid_program_addr(addr) {
								return Err(ExecuteError::BadJumpAddr);
							}
							self.pc = addr;
						}
						_ => {
							return Err(ExecuteError::BadInstruction);
						}
					}
				}
			}
			0x1000 => {
				// instruction == 0x1NNN
				// jump to address NNN
				if !self.is_valid_program_addr(nnn) {
					return Err(ExecuteError::BadJumpAddr);
				}
				self.pc = nnn;
			}
			0x2000 => {
				// instruction == 0x2NNN
				// execute subroutine starting at address NNN
				if self.stack.len() == 16 {
					return Err(ExecuteError::MaxCallDepthReached);
				}
				self.stack.push(self.pc);
				self.pc = nnn;
			}
			0x3000 => {
				// instruction == 0x3XNN
				// skip the following instruction if the value of VX == NN
				if self.gp_registers[x] == nn {
					self.pc += 2;
				}
			}
			0x4000 => {
				// instruction == 0x4XNN
				// skip the following instruction if the value of VX != NN
				if self.gp_registers[x] != nn {
					self.pc += 2;
				}
			}
			0x5000 => {
				if instruction & 0x000f != 0 {
					return Err(ExecuteError::BadInstruction);
				}
				// instruction === 0x5XY0
				// skip the following instruction if the value of VX == VY
				if self.gp_registers[x] == self.gp_registers[y] {
					self.pc += 2;
				}
			}
			0x6000 => {
				// instruction == 0x6XNN
				// store number nn in register VX
				self.gp_registers[x] = nn;
			}
			0x7000 => {
				// instruction == 0x7XNN
				// add value NN to register VX
				self.gp_registers[x] = self.gp_registers[x].wrapping_add(nn);
			}
			0x8000 => {
				match n {
					0x0 => {
						// instruction == 0x8XY0
						// store value of VY in VX
						self.gp_registers[x] = self.gp_registers[y];
					}
					0x1 => {
						// instruction == 0x8XY1
						// set VX = VX | VY
						self.gp_registers[x] |= self.gp_registers[y];
					}
					0x2 => {
						// instruction == 0x8XY2
						// set VX = VX & VY
						self.gp_registers[x] &= self.gp_registers[y];
					}
					0x3 => {
						// instruction == 0x8XY3
						// set VX = VX ^ VY
						self.gp_registers[x] ^= self.gp_registers[y];
					}
					0x4 => {
						// instruction == 0x8XY4
						// set VX = VX + VY. set VF = 0x01 if carry occurs, otherwise set VF = 0x00
						let t = self.gp_registers[x].checked_add(self.gp_registers[y]);
						match t {
							Some(val) => {
								self.gp_registers[x] = val;
								self.gp_registers[0xf] = 0;
							}
							None => {
								// addition overflowed
								self.gp_registers[x] = (self.gp_registers[x] as u16 + self.gp_registers[y] as u16 - 0xffu16) as u8;
								self.gp_registers[0xf] = 1;
							}
						}
					}
					0x5 => {
						// instruction == 0x8XY5
						// set VX = VX - VY. set VF = 0x00 if borrow occurs, otherwise set VF = 0x01
						let t = self.gp_registers[x].checked_sub(self.gp_registers[y]);
						match t {
							Some(val) => {
								self.gp_registers[x] = val;
								self.gp_registers[0xf] = 1;
							}
							None => {
								// subtraction underflowed
								self.gp_registers[x] = 0xff - (self.gp_registers[y] - self.gp_registers[x]) + 0x1;
								self.gp_registers[0xf] = 0;
							}
						}
					}
					0x6 => {
						// instruction == 0x8XY6
						// set VX = VY >> 1, set VF to the least significant bit of VY before shift. VY is unchanged
						self.gp_registers[x] = self.gp_registers[y] >> 1;
						self.gp_registers[0xf] = self.gp_registers[y] & 0x1;
					}
					0x7 => {
						// instruction == 0x8XY7
						// set VX = VY - VX. set VF = 0x00 if borrow occcurs, otherwise set VF = 0x01
						let t = self.gp_registers[y].checked_sub(self.gp_registers[x]);
						match t {
							Some(val) => {
								self.gp_registers[x] = val;
								self.gp_registers[0xf] = 1;
							}
							None => {
								// subtraction underflowed
								self.gp_registers[x] = 0xff - (self.gp_registers[x] - self.gp_registers[y]) + 0x1;
								self.gp_registers[0xf] = 0;
							}
						}
					}
					0xE => {
						// instruction == 0x8XYE
						// set VX = VY << 1, set VF to the most significant bit of VY before shift. VY is unchanged
						self.gp_registers[x] = self.gp_registers[y] << 1;
						self.gp_registers[0xf] = self.gp_registers[y] & 0x80;
					}
					_ => {
						return Err(ExecuteError::BadInstruction);
					}
				}
			}
			0x9000 => {
				if (instruction & 0x000f) != 0 {
					return Err(ExecuteError::BadInstruction);
				}
				// instruction == 0x9XY0
				// skip the following instruction if VX != VY
				if self.gp_registers[x] != self.gp_registers[y] {
					self.pc += 2;
				}
			}
			0xa000 => {
				// instruction == 0xANNN
				// store memory address NNN in I
				self.i = nnn as u16;
			}
			0xb000 => {
				// instruction == 0xBNNN
				// jump to address V0 + NNN
				self.pc = self.gp_registers[0] as usize + nnn;
			}
			0xc000 => {
				// instruction == 0xCXNN
				// set VX to random number with the mask NN
				let random = rand::random::<u8>();
				self.gp_registers[x] = random & nn;
			}
			0xd000 => {
				// instruction == 0xDXYN
				// draw a sprite at position VX and VY with N bytes of sprite data starting at
				// address stored in I.
				// Set VF = 0x01 if any set pixels are changed to unset, otherwise set VF = 0x00.
				
				if self.draw_sprite(n, self.gp_registers[x], self.gp_registers[y])? {
					self.gp_registers[0xf] = 0x01;
				}
				else {
					self.gp_registers[0xf] = 0x00;
				}
			}
			0xe000 => {
				match instruction & 0xff {
					0x9e => {
						// instruction == 0xEX9E
						// skip the following instruction if the key corresponding to the hex value in VX
						// is pressed. do not wait for input
						if self.is_key_pressed && self.last_key_pressed == self.gp_registers[x] {
							self.pc += 2;
						}
					}
					0xa1 => {
						// instruction == 0xEXA1
						// skip the following instruction if the key corresponding to the hex value in VX
						// is not pressed. do not wait for input
						if !self.is_key_pressed || 
							(self.is_key_pressed && self.last_key_pressed != self.gp_registers[x]) 
						{
							self.pc += 2;
						}
					}
					_ => {
						return Err(ExecuteError::BadInstruction);
					}
				}
			}
			0xf000 => {
				match instruction & 0xff {
					0x07 => {
						// instruction == 0xFX07
						// store current value of delay timer in VX
						self.gp_registers[x] = self.delay_timer;
					}
					0x0A => {
						// instruction == 0xFX0A
						// wait for keypress and store the value of key in VX
						while !self.is_key_pressed {
							self.gp_registers[x] = self.last_key_pressed;
							sleep(Duration::from_millis(60));
						}
					}
					0x15 => {
						// instruction == 0xFX15
						// set the delay timer to the value of VX
						self.delay_timer = self.gp_registers[x];
					}
					0x18 => {
						// instruction == 0xFX18
						// set the sound timer to the value of VX
						self.sound_timer = self.gp_registers[x];
					}
					0x1e => {
						// instruction == 0xFX1E
						// Add the value stored in VX to I
						self.i += self.gp_registers[x] as u16;
					}
					0x29 => {
						// instruction == 0xFX29
						// set I to memory address of sprite data corresponding to the digit stored in register VX
						todo!("Implement preset sprite data for 0x00 to 0x0F");
					}
					0x33 => {
						// instruction == 0xFX33
						// store the binary coded decimal equivalent of value in VX at addr I, I+1, I+2
						// https://en.wikipedia.org/wiki/Binary-coded_decimal
						let vx = self.gp_registers[x];
						// TODO: bound check
						let addr = self.i as usize;
						self.mem[addr] = vx / 100;
						self.mem[addr + 1] = (vx % 100) / 10;
						self.mem[addr + 2] = vx % 10;
					}
					0x55 => {
						// instruction == 0xFX55
						// store the values of registers V0 to VX inclusive to memory starting at address I.
						// set I = I + X + 1 after saving.
						let addr = self.i as usize;
						self.mem[addr..=addr + x].copy_from_slice(&self.gp_registers[0..=x]);
						self.i += (x + 1) as u16;
					}
					0x65 => {
						// instruction == 0xFX65
						// fill V0 to VX inclusive with values stored at memory starting at address I.
						// set I = I + X + 1 after filling.
						let addr = self.i as usize;
						self.gp_registers[0..=x].copy_from_slice(&self.mem[addr..=addr+x]);
						self.i += (x + 1) as u16;
					}
					_ => {
						return Err(ExecuteError::BadInstruction);
					}
				}
			}
			_ => {
				return Err(ExecuteError::BadInstruction);
			}
		}
		self.pc += 2;
		Ok(())
	}
	#[inline]
	fn is_valid_program_addr(&self, addr: usize) -> bool {
		return addr >= 0x200 && addr <= self.program_end_addr;
	}
	fn get_next_instruction(&self) -> Result<u16, ExecuteError> {
		let byte_1 = self.mem.get(self.pc as usize);
		let byte_2 = self.mem.get(self.pc as usize + 1);

		if byte_1.is_none() || byte_2.is_none() {
			return Err(ExecuteError::FailedToReadInstruction);
		}
		let instruction = (*byte_1.unwrap() as u16) << 8 | *byte_2.unwrap() as u16;
		return Ok(instruction);
	}
	
	fn draw_sprite(&mut self, n: u8, x: u8, y: u8) -> Result<bool, ExecuteError> {
		// flag is set if is any set pixels are set to unset
		let mut should_set_flag = false;
		let sprite_start = self.i as usize;
		let sprite_end = sprite_start + n as usize;
		// FIXME: this is not the correct way to stop execution of sprites
		if sprite_start <= self.program_end_addr ||  sprite_end <= self.program_end_addr {
			return Err(ExecuteError::InvalidSprite);
		}

		let mut coord_x = x as usize;
		let mut coord_y = y as usize;

		let sprite = Vec::from(&self.mem[sprite_start..sprite_end]);


		// each byte in the display buffer corresponds to a pixel and a bit in the sprite
		// each sprite is always 1 byte wide and 1 to 15 pixels tall
		let mut d_buffer = self.d_buffer.borrow_mut();
		for byte in sprite {
			let mut b = byte;
			for _ in 0..8 {
				coord_x %= WIDTH;
				coord_y %= HEIGHT;
				let index = coord_x + coord_y * WIDTH;
				let prev_value = d_buffer[index];
				// the sprite is drawn by xoring with the current value not by setting a new value
				d_buffer[index] ^= (b & 0x80) >> 7;

				if prev_value == 1 && d_buffer[index] == 0 {
					should_set_flag = true;
				}
				coord_x += 1;
				b <<= 1;
			}
			coord_y += 1;
		}

		Ok(should_set_flag)
	}
}