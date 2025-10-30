use std::{
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};

use minifb::{Window, WindowOptions};

use crate::{
    cpu::{self, HEIGHT, WIDTH},
    ext::ToARGB,
    keyboard,
};

const SCALE: usize = 8;

pub struct Chip8 {
    cpu: cpu::Cpu,
    io: IO,
    timing: bool,
    timing_data: Timing,
    scaled_buffer: Vec<u32>,
}

struct Timing {
    avg: f64,
    instructions: u32,
    last_time: Instant,
}

pub struct IO {
    window: Window,
    keyboard: keyboard::KeyBoard,
}

impl Chip8 {
    pub fn new(debug: bool, timing: bool) -> Chip8 {
        let window = Window::new("CHIP8", WIDTH * 8, HEIGHT * 8, WindowOptions::default())
            .expect("failed to create a window");
        Chip8 {
            cpu: cpu::Cpu::init(debug),
            io: IO {
                window,
                keyboard: keyboard::KeyBoard::new(),
            },
            timing,
            timing_data: Timing {
                avg: 0f64,
                instructions: 0,
                last_time: Instant::now(),
            },
            scaled_buffer: vec![0u32; WIDTH * HEIGHT * 64],
        }
    }

    pub fn run(&mut self) {
        loop {
            let res = self.cpu.step(&self.io.keyboard);

            if self.timing {
                let now = Instant::now();
                let elapsed = now.duration_since(self.timing_data.last_time).as_micros() as f64;
                self.timing_data.last_time = now;
                println!("time: {} microsecs", elapsed);
                self.timing_data.instructions += 1;
                self.timing_data.avg =
                    (self.timing_data.avg * (self.timing_data.instructions - 1) as f64 + elapsed)
                        / (self.timing_data.instructions as f64);
            }
            self.reset_keys();
            match res {
                Ok(did_draw) => {
                    self.check_keypresses();
                    if did_draw {
                        self.update_window();
                    }
                    sleep(Duration::from_millis(1000 / 60));
                }
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    return;
                }
            }
        }
    }

    pub fn add_program(&mut self, program: &[u8]) -> Result<(), std::io::Error> {
        self.cpu.add_program(program)?;
        Ok(())
    }

    fn scale_d_buffer(&mut self) {
        for (y, row) in self.cpu.d_buffer.borrow().chunks(WIDTH).enumerate() {
            let base_y = y * SCALE;
            for (x, &val) in row.iter().enumerate() {
                let color = val.to_argb();
                let base_x = x * SCALE;

                // fill 8x8 block directly
                for dy in 0..SCALE {
                    let row_start = (base_y + dy) * WIDTH * SCALE + base_x;
                    self.scaled_buffer[row_start..row_start + SCALE].fill(color);
                }
            }
        }
    }

    fn update_window(&mut self) {
        self.scale_d_buffer();
        self.io
            .window
            .update_with_buffer(&self.scaled_buffer, WIDTH * 8, HEIGHT * 8)
            .expect("Failed to draw window");
    }

    fn check_keypresses(&mut self) {
        let pressed = self.io.window.get_keys_pressed(minifb::KeyRepeat::Yes);
        match pressed.last() {
            Some(k) => match *k {
                minifb::Key::NumPad1 => {
                    self.cpu.dump(true, 0);
                }
                minifb::Key::NumPad2 => {
                    self.cpu.dump_everything();
                }
                minifb::Key::NumPad3 => {
                    self.die();
                }
                _ => {}
            },
            None => {}
        }
        self.io.keyboard.set_key_pressed(pressed.last());
    }
    fn reset_keys(&mut self) {
        self.io.keyboard.key_pressed = None;
    }

    fn die(&self) {
        if self.timing {
            println!("Avg time for instruction: {} micros", self.timing_data.avg);
        }
        exit(0);
    }
}
