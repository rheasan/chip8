// 0xE8F - 0x200 = 0xC8F = 3215 bytes
const MAX_PROGRAM_SIZE : usize = 3215usize;

pub struct Cpu<'a> {
    mem: Vec<u8>,
    d_buffer: Option<&'a mut Vec<u8>>,
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
}


impl<'a> Cpu<'a> {
    pub fn init() -> Self {
        return Cpu {
            mem: vec![0u8; 4096],
            d_buffer: None,
            gp_registers: [0u8; 16],
			i: 0,
			pc: 0x200,
			sp: 0,
			stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
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

		Ok(())
	}
	pub fn get_mem(self: &'a Self) -> &'a Vec<u8> {
		&self.mem
	}
}