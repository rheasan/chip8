use std::{process::exit, sync::{Arc, Mutex}, thread::{self, sleep}, time::Duration};

use minifb::{Window, WindowOptions};

use crate::{cpu::{self, ExecuteError, HEIGHT, WIDTH}, ext::ToARGB, keyboard};

pub struct Chip8 {
	cpu: cpu::Cpu,
	io: IO
}

pub struct IO {
	window: Window,
	keyboard: keyboard::KeyBoard
}



impl Chip8 {
	pub fn new() -> Chip8 {
		let window = Window::new("CHIP8", WIDTH*8, HEIGHT*8, WindowOptions::default())
		.expect("failed to create a window");
		Chip8 {
			cpu: cpu::Cpu::init(),
			io: IO {
				window,
				keyboard: keyboard::KeyBoard::new()
			}
		}
	}

	pub fn run(&mut self) {
		loop {
			let res = self.cpu.step(&self.io.keyboard);
			match res {
				Ok(()) => {
					self.check_keypresses();
					self.update_window();
					sleep(Duration::from_millis(20));
				}
				Err(e) => {
					eprintln!("{}", e.to_string());
					return
				}
			}
		}
	}

	pub fn add_program(&mut self, program: &[u8]) -> Result<(), std::io::Error> {
		self.cpu.add_program(program)?;
		Ok(())
	}

	fn scale(buffer: &[u8]) -> Vec<u32> {
		let mut res = Vec::with_capacity(buffer.len() * 64);

		for chunk in buffer.chunks(WIDTH) {
			let chunk_i32 = chunk.iter()
				.flat_map(|e| std::iter::repeat(e.to_argb()).take(8))
				.collect::<Vec<_>>();
			for _ in 0..8 {
				res.extend(chunk_i32.clone());
			}
		}
		res
	}
	fn update_window(&mut self) {
		let scaled_buffer = Self::scale(&self.cpu.d_buffer.borrow());
			self
			.io
			.window
			.update_with_buffer(&scaled_buffer, WIDTH*8, HEIGHT*8).expect("Failed to draw window");
	}

	fn check_keypresses(&mut self) {
		let pressed = self.io.window.get_keys_pressed(minifb::KeyRepeat::No);
		self.io.keyboard.set_key_pressed(pressed.last());
	}
}

