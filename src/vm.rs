use rand::distributions::{IndependentSample, Range};
use rand;

pub struct CPU {
	pub delay_timer: u8,
	pub sound_timer: u8,
	pub gfx: GFX,
	registers: [u8; 8],
	pc: u16,
	index: u16,
	sp: u8,
	keypad: [u8; 16],
	draw_flag: bool,
	stack: Vec<u16>, 
	ram: RAM
}

type Keypad = [u8; 16];
type RAM = [u8; 4096];
pub type GFX = [u8; 64 * 32];

pub struct Chip8 {
	cpu: CPU
}

impl Default for CPU {
	fn default() -> CPU {
		CPU {
			gfx: [0; 2048],
			..Default::default()
		}
	}
}

impl CPU {
	pub fn write_memory(&mut self, bytes: &[u8], address: u16) {
		for i in 0..bytes.len() as usize {
			self.ram[address as usize + i] = bytes[i]
		}
	}

	pub fn emulate_cycle(&mut self) {
			let register_x: usize;
			let register_y: usize;

			let instruction = self._fetch() as usize;
			if 0xF000 & instruction == 0x1000 {
				self.pc = instruction as u16 & 0x0FFF;
			}
			else if instruction == 0x00E0 { //Clears the screen.
				self.gfx = [0; 2048];
			}
			else if instruction == 0x00EE { //Returns from a subroutine.
				self.pc = self.stack.pop().unwrap();
			}
			else if instruction & 0xF000 == 0x1000 {
				let address = instruction & 0x0FFF;
				self.pc = address as u16;
			}
			else if instruction & 0xF000 == 0x2000 { //Calls subroutine at NNN.
				let sub = instruction - 0x2000;
				self.stack.push(self.pc);
				self.pc = sub as u16;
			} 
			else if instruction & 0xF000 == 0x3000 {// 3XNN	//Skips the next instruction if VX equals NN.
				let register = (instruction & 0x0F00) >> 8;
				if self.registers[register] == 0x00FF & instruction as u8 {
					self.pc += 2;
				}
			}
			else if instruction & 0xF000 == 0x4000 {
				//  4XNN	Skips the next instruction if VX doesn't equal NN.
				let register = (instruction & 0x0F00) >> 8;
				if self.registers[register] != 0x00FF & instruction as u8 {
					self.pc += 2;
				}
			}
			else if instruction & 0xF00F == 0x5000 { 
				// 5XY0 //Skips the next instruction if VX equals VY.
				register_x = (0x0F00 & instruction) >> 8;
				register_y = (0x0F0 & instruction) >> 4;
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
				self.registers[register_x] = self.registers[register_x] + (instruction & 0x00FF) as u8; 
			}
			else if instruction & 0xF00F == 0x8000 { 
				//8XY0	Sets VX to the value of VY. 
				register_x = (instruction & 0x0F00) >> 8;
				register_y = (instruction & 0x0F00) >> 4;
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
				self.registers[register_x] = self.registers[register_x] & self.registers[register_y];
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
			} 
			else if instruction & 0xF000 == 0xF000 {
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
			} 
			else if instruction & 0xF0FF == 0xF033 {
				panic!("Load/store instructions not implemented");
			} // FX33	Stores the Binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
			else if instruction & 0xF0FF == 0xF055 {
				panic!("Load/store instructions not implemented");
			} // FX55	Stores V0 to VX (including VX) in memory starting at address I.[4]
			else if instruction & 0xF0FF == 0xF065 {
				panic!("Load/store instructions not implemented");
			} // FX65	Fills V0 to VX (including VX) with values from memory starting at address I.[4]
			else {
				panic!("Unknown instruction: {}", instruction);
			}
		}

	fn _fetch(&mut self) -> u16 {
		let i1 = self.pc;
		let i2 = self.pc + 1;
		let opcode = (i1 as u16) << 8 | (i2 as u16);
		return opcode;
	}
}

impl Chip8 {
	pub fn new() -> Chip8 {
		Chip8 {
			cpu: CPU {
				//gfx: [0; 2048],
				//draw_flag: false,
				//keypad: [0; 16],
				..Default::default()
			}
		}
	}

	pub fn boot(&mut self, cartridge: &[u8]) {
		self._initialise_memory();
		self.cpu.write_memory(cartridge, 0x512);
		self.cpu.pc = 512;
	}

	fn _initialise_memory(&mut self) {
		//TODO make this actually do something
	}
}