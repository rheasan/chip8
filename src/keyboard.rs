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
            sleep(Duration::from_millis(100));
        }
        self.key_pressed.unwrap()
    }

    pub fn set_key_pressed(&mut self, key: Option<&Key>) {
        if key.is_none() {
            return;
        }
        // https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference#keypad-input
        let key_value: u8 = match key.unwrap() {
            Key::Key1 => 0x1,
            Key::Key2 => 0x2,
            Key::Key3 => 0x3,
            Key::Key4 => 0xc,
            Key::Q => 0x4,
            Key::W => 0x5,
            Key::E => 0x6,
            Key::R => 0xd,
            Key::A => 0x7,
            Key::S => 0x8,
            Key::D => 0x9,
            Key::F => 0xe,
            Key::Z => 0xa,
            Key::X => 0x0,
            Key::C => 0xb,
            Key::V => 0xf,
            _ => 0xff
        };

        if key_value == 0xff {
            self.key_pressed = None;
        } else {
            println!("Key pressed: {:?}", key);
            self.key_pressed = Some(key_value);
        }
    }
}