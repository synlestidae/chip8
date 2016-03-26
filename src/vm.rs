use rand::distributions::{IndependentSample, Range};
use rand;
use std::sync::mpsc::Receiver;

pub struct CPU {
	pub delay_timer: u8,
	pub sound_timer: u8,
	pub gfx: GFX,
	registers: [u8; 16],
	pc: u16,
	index: u16,
	sp: u8,
	keypad: [u8; 16],
	draw_flag: bool,
	stack: Vec<u16>, 
	ram: RAM,
	key_input: Receiver<Key>
}

type Keypad = [u8; 16];
type RAM = [u8; 4096];
pub type GFX = [u8; 64 * 32];

pub struct Chip8 {
	cpu: CPU
}

impl CPU {
	pub fn new(key_input: Receiver<Key>) -> CPU {
		CPU {
			delay_timer: 0,
			sound_timer: 0,
			gfx: [0; 2048],
			registers: [0; 16],
			pc: 0,
			index: 0,
			sp: 0,
			keypad: [0; 16],
			draw_flag: false,
			stack: Vec::new(), 
			ram: [0; 4096],
			key_input: key_input
		}
	}

	pub fn write_memory(&mut self, bytes: &[u8], address: u16) {
		for i in 0..bytes.len() as usize {
			self.ram[address as usize + i] = bytes[i];
			//println!("B: {} {}", (address as usize + i), bytes[i]);
		}
	}

	pub fn emulate_cycle(&mut self) {
			let register_x: usize;
			let register_y: usize;

			let instruction = self._fetch() as usize;
			if 0xF000 & instruction == 0x1000 {
				self.pc = instruction as u16 & 0x0FFF;
				return;
			}
			else if instruction == 0x00E0 { //Clears the screen.
				self.gfx = [0; 2048];
			}
			else if instruction == 0x00EE { //Returns from a subroutine.
				self.pc = self.stack.pop().unwrap();
				return;
			}
			else if instruction & 0xF000 == 0x1000 {
				let address = instruction & 0x0FFF;
				self.pc = address as u16;
				return;
			}
			else if instruction & 0xF000 == 0x2000 { //Calls subroutine at NNN.
				let sub = instruction - 0x2000;
				self.stack.push(self.pc);
				self.pc = sub as u16;
				return;
			} 
			else if instruction & 0xF000 == 0x3000 {
				// 3XNN	//Skips the next instruction if VX equals NN.
				let register = (instruction & 0x0F00) >> 8;
				let n = (0x00FF & instruction) as u8;
				println!("Reg: {} {} {}", register, self.registers[register], n);
				if self.registers[register] == n {
					self.pc += 2;
				}
			}
			else if instruction & 0xF000 == 0x4000 {
				//  4XNN	Skips the next instruction if VX doesn't equal NN.
				let register = (instruction & 0x0F00) >> 8;
				let n = (0x00FF & instruction) as u8;
				println!("Reg: {} {} {}", register, self.registers[register], n);
				if self.registers[register] != 0x00FF & instruction as u8 {
					self.pc += 2;
				}
			}
			else if instruction & 0xF00F == 0x5000 { 
				// 5XY0 //Skips the next instruction if VX equals VY.
				register_x = (0x0F00 & instruction) >> 8;
				register_y = (0x00F0 & instruction) >> 4;
				if self.registers[register_x] == self.registers[register_y] {
					self.pc += 2;
				}
			}	
			else if instruction & 0xF000 == 0x6000 {
				//6XNN	Sets VX to NN.
				register_x = instruction & 0x0F00 >> 8;
				self.registers[register_x] = (instruction & 0x00FF) as u8;
			}
			else if instruction & 0xF000 == 0x7000 {
				//7XNN	Adds NN to VX.
				register_x = (instruction & 0x0F00) >> 8;
				let n = (instruction & 0x00FF) as u8;
				self.registers[register_x] = self.registers[register_x].overflowing_add(n).0; 
			}
			else if instruction & 0xF00F == 0x8000 { 
				//8XY0	Sets VX to the value of VY. 
				register_x = (instruction & 0x0F00) >> 8;
				register_y = (instruction & 0x00F0) >> 4;
				self.registers[register_x] = self.registers[register_y];
			}
			else if instruction & 0xF00F == 0x8001 { 
				//8XY1	Sets VX to VX or VY.
				register_x = (instruction & 0x0F00) >> 8;
				register_y = (instruction & 0x0F00) >> 4;
				self.registers[register_x] = self.registers[register_x] | self.registers[register_y];
			}
			else if instruction & 0xF00F == 0x8002 {
				//8XY2	Sets VX to VX and VY.
				register_x = (instruction & 0x0F00) >> 8;
				register_y = (instruction & 0x0F00) >> 4;
				self.registers[register_x] = self.registers[register_x] & self.registers[register_y];
			}
			else if instruction & 0xF00F == 0x8003 {
				//8XY3	Sets VX to VX xor VY.
				register_x = (instruction & 0x0F00) >> 8;
				register_y = (instruction & 0x0F00) >> 4;
				self.registers[register_x] = self.registers[register_x] & self.registers[register_y];
			}
			else if instruction & 0xF00F == 0x8004 {
				//8XY4	Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
				register_x = (instruction & 0x0F00) >> 8;
				register_y = (instruction & 0x0F00) >> 4;
				let result = self.registers[register_x].overflowing_add(self.registers[register_y]);
				self.registers[register_x] =  result.0;
				self.ram[0xF] = match result.1 {
					 true => 1,
					 false => 0
				};
			}
			else if instruction & 0xF00F == 0x8005 {
				/*8XY5 VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.*/
				let register_x = instruction & 0x0F00 >> 8;
				let register_y = instruction & 0x00F0 >> 4;
				let val = self.registers[register_x];
				let result = val - self.registers[register_y];
				self.registers[register_x] = result;
				if result < val {
					self.registers[0xF] = 0;
				}
				else {
					self.registers[0xF] = 1;
				}
			}
			else if instruction & 0xF00F == 0x8006 {
				/* 8XY6 Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift.[2]*/
				register_x = (instruction & 0x0F00) >> 8;
				self.registers[register_x] = self.registers[register_x] >> 1;
			}
			else if instruction & 0xF00F == 0x8007 {
				/*8XY7	Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.*/
				let register_x = instruction & 0x0F00 >> 8;
				let register_y = instruction & 0x00F0 >> 4;
				let val = self.registers[register_x];
				let result = self.registers[register_y] - val;
				self.registers[register_x] = result;
				if (result < val) {
					self.registers[0xF] = 0;
				}
				else {
					self.registers[0xF] = 1;
				}
			}
			else if instruction & 0xF00F == 0x800E {
				/*8XYE	Shifts VX left by one. VF is set to the 
				value of the most significant bit of VX before the shift.[2]*/
				register_x = instruction & 0x0F00 >> 8;
				let val = self.registers[register_x];
				self.registers[0xF] = val & 129 >> 7;
				self.registers[register_x] = val << 1;
			}
			else if instruction & 0xF00F == 0x9000 {
				/*0x9XY0	Skips the next instruction if VX doesn't equal VY.*/
				register_x = (instruction & 0x0F00) >> 8;
				register_y = (instruction & 0x0F0) >> 4;
				if self.registers[register_x] != self.registers[register_y] {
					self.pc += 2;
				}
			}
			else if instruction & 0xF000 == 0xA000 {/*ANNN	Sets I to the address NNN.*/
				self.index = (instruction & 0x0FFF) as u16;
			}
			else if instruction & 0xF000 == 0xB000 {
				//BNNN	Jumps to the address NNN plus V0.
				self.pc = 0x0FFF & (instruction as u16) + (self.registers[0] as u16);
				return;
			} 
			else if instruction & 0xF000 == 0xC000 {
				//CXNN	Sets VX to the result of a bitwise and operation on a random number and NN.
				let between = Range::new(0, 0xFF);
				let mut rng = rand::thread_rng();
				let n = (instruction & 0x0F00) >> 4 as u8;
				register_x = (instruction & 0x0F00) >> 8;
				self.registers[register_x] = (n & between.ind_sample(&mut rng)) as u8;
			} 
			else if instruction & 0xF000 == 0xD000 {
				// DXYN	Sprites stored in memory at location in index register (I), 
				//8bits wide. Wraps around the screen. If when drawn, clears a 
				//pixel, register VF is set to 1 otherwise it is zero. All drawing 
				//is XOR drawing (i.e. it toggles the screen pixels). Sprites are 
				//drawn starting at position VX, VY. N is the number of 8bit rows 
				//that need to be drawn. If N is greater than 1, second line 
				//continues at position VX, VY+1, and so on.
				let sprite_height = (instruction & 0x000F) as u16;
				register_x = instruction & 0x0F00 >> 8;
				register_y = instruction & 0x00F0 >> 8;
				let px = self.registers[register_x];
				let py = self.registers[register_y];
				println!("Screen position {:?}", (px, py));
				for i in 0..sprite_height {
					let mut row = self.ram[(self.index + i) as usize];
					let mut x : i8 = 7;
					while x > 0 {
						let mut pixel = &mut self.gfx[(py as usize) * 32 + (px as usize)];
						let ghost_pixel = *pixel;
						*pixel = match (*pixel != 0) ^ ((row & (1 << x)) >> x == 1) {
							true => 1,
							false => 0
						};
						//If this causes any pixels to be erased, VF is set to 1 
						//otherwise it is set to 0
						if (*pixel == 0 && ghost_pixel == 1) {
							self.registers[0xF] = 1;
						}
						else {
							self.registers[0xF] = 0;
						}
						x = x - 1;
					}
				}
			} 
			else if instruction & 0xF0FF == 0xE09E {
				//EX9E	Skips the next instruction if the key stored in VX is pressed.
				panic!("Key presses are not implemented");
			} 
			else if instruction & 0xF0FF == 0xE0A1 {
				//EXA1	Skips the next instruction if the key stored in VX isn't pressed.
				panic!("Key presses are not implemented");
			} 
			else if instruction & 0xF0FF == 0xF007 {
				//FX07 Sets VX to the value of the delay timer.
				register_x = (instruction & 0x0F00) >> 8;
				self.registers[register_x] = self.delay_timer;
			}	
			else if instruction & 0xF0FF == 0xF00A {
				// FX0A	A key press is awaited, and then stored in VX.
				panic!("Key presses are not implemented");
			} 
			else if instruction & 0xF0FF == 0xF015 {
				//  FX15	Sets the delay timer to VX.
				register_x = instruction & 0x0F00 >> 8;
				self.delay_timer = self.registers[register_x] as u8;
			} 
			else if instruction & 0xF0FF == 0xF018 {
				// FX18	Sets the sound timer to VX.
				register_x = instruction & 0x0F00 >> 8;
				self.sound_timer = self.registers[register_x] as u8;
			} 
			else if instruction & 0xF0FF == 0xF01E {
				//FX1E	Adds VX to I.[3]
				register_x = (0x0F00 & instruction) >> 8; 
				self.index = self.index + self.registers[register_x] as u16;
			} 
			else if instruction & 0xF0FF == 0xF029 {
				// FX29	Sets I to the location of the sprite for the character in VX. 
				//haracters 0-F (in hexadecimal) are represented by a 4x5 font.
				panic!("Sprite instruction FX29 not implemented");
			} 
			else if instruction & 0xF0FF == 0xF033 {
				// FX33	Stores the Binary-coded decimal representation of VX, 
				//with the most significant of three digits at the address in 
				//I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
				panic!("Load/store instructions not implemented");
			} 
			else if instruction & 0xF0FF == 0xF055 {
				// FX55	Stores V0 to VX (including VX) in memory starting at address I.[4]
				register_x = (instruction & 0x0F00) >> 8;
				for j in 0..register_x {
					self.ram[self.index as usize + j] = self.registers[j];
				}
			}
			else if instruction & 0xF0FF == 0xF065 {
				// FX65	Fills V0 to VX (including VX) with values from memory starting at address I.[4]
				let k = instruction & (0x0F00 >> 8) as usize;
				for j in 0..k {
					self.registers[j] = self.ram[self.index as usize + j as usize]
				}
			} 
			else {
				panic!("Unknown instruction: {}", instruction);
			}
			self.pc += 2;
		}		

	pub fn deal_with_input(&mut self) {
		//TODO: Make this do stuff
	}

	fn _fetch(&mut self) -> u16 {
		let i1 = self.ram[self.pc as usize] as u16;
		let i2 = self.ram[self.pc as usize + 1] as u16;
		let opcode = (i1 << 8) | i2;
		//println!("OP: {}", format!("{:X} {:X}", self.pc, opcode));
		return opcode;
	}
}

impl Chip8 {
	pub fn new(key_input: Receiver<Key>) -> Chip8 {
		Chip8 {
			cpu: CPU::new(key_input)
		}
	}

	pub fn load(&mut self, cartridge: &[u8]) {
		self._initialise_memory();
		self.cpu.write_memory(cartridge, 512);
		self.cpu.pc = 512;
	}

	pub fn run(&mut self) {
		loop {
			self.cpu.emulate_cycle();
			self.cpu.deal_with_input();
		}
	}

	fn _initialise_memory(&mut self) {
		//TODO make this actually do something
	}
}

#[derive(Clone, Copy)]
pub enum Key {
	K0,K1,K2,K3,K4,K5,K6,K7,K8,K9,
	A,B,C,D,E,F
}

impl Key {
	pub fn to_byte(&self) -> u8 {
		match self {
			&Key::K0 => 0x0,
			&Key::K1 => 0x1,
			&Key::K2 => 0x2,
			&Key::K3 => 0x3,
			&Key::K4 => 0x4,
			&Key::K5 => 0x5,
			&Key::K6 => 0x6,
			&Key::K7 => 0x7,
			&Key::K8 => 0x8,
			&Key::K9 => 0x9,
			&Key::A => 0xA,
			&Key::B => 0xB,
			&Key::C => 0xC,
			&Key::D => 0xD,
			&Key::E => 0xE,
			&Key::F => 0xF
		}
	}
}