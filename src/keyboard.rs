use std::{thread::sleep, time::Duration};

use minifb::Key;


pub struct KeyBoard {
	key_pressed: Option<u8>
}

impl KeyBoard {
	pub fn new() -> KeyBoard {
		KeyBoard { key_pressed: None }
	}

	pub fn get_current_key(&self) -> Option<u8> {
		self.key_pressed
	}

	pub fn block_until_keypress(&self) -> u8 {
		while self.key_pressed.is_none() {
			sleep(Duration::from_millis(20));
		}
		self.key_pressed.unwrap()
	}

    pub fn set_key_pressed(&mut self, key: Option<&Key>) {
		todo!("Map Key to hex keycoded")
    }
}