use vm::{Chip8, GFX, Key};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

fn make_chip8() -> (Chip8, Sender<(Key, bool)>, Receiver<GFX>) {
	let key_channel = mpsc::channel(); 
	let gfx_channel = mpsc::channel(); 
	(Chip8::new(key_channel.1, gfx_channel.0), key_channel.0, gfx_channel.1)
}

#[test]
fn test_jump_instruction_0x2nnn() {
	let mut chip8 = make_chip8().0;
	chip8.load(&[0x20, 0x00]);
	let mut cpu = chip8.cpu;
	cpu.emulate_cycle();
	assert_eq!(cpu.pc, 0);
}

#[test]
fn test_binary_coded_0xfx33() {
	let mut chip8 = make_chip8().0;
	chip8.load(&[0xF0, 0x33]);
	let mut cpu = chip8.cpu;
	cpu.registers[0] = 123;
	cpu.index = 1024;
	cpu.emulate_cycle();
	assert_eq!(vec![1,2,3], cpu.read_memory(1024, 1024 + 3).iter().map(|&c| c).collect::<Vec<u8>>());
}


#[test]
fn test_fill_instruction_0xfx55() {
	let mut chip8 = make_chip8().0;
	let r = (0..16).collect::<Vec<_>>();
	chip8.load(&[0xFF, 0x65]);
	let mut cpu = chip8.cpu;
	cpu.write_memory(&r, 0x820);
	cpu.index = 0x820;
	cpu.emulate_cycle();
	assert_eq!(r.iter().collect::<Vec<_>>(), cpu.registers[0..16].iter().collect::<Vec<_>>());
}


#[test]
fn test_stores_instruction_0xfx55() {
	let mut chip8 = make_chip8().0;
	let r = (0..16).collect::<Vec<_>>();
	chip8.load(&[0xFF, 0x55]);
	let mut cpu = chip8.cpu;
	cpu.registers = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
	cpu.index = 0x820;
	cpu.emulate_cycle();
	assert_eq!(r.iter().collect::<Vec<_>>(), cpu.read_memory(0x820, 0x820 + 16).iter().collect::<Vec<_>>());
}

#[test]
fn test_adds_vx_to_index_0xfx1e() {
	let mut chip8 = make_chip8().0;
	chip8.load(&[
		0xF0, 0x1E
		]
	);
	let mut cpu = chip8.cpu;
	cpu.registers[0] = 12;
	cpu.index = 600;
	cpu.emulate_cycle();
	assert_eq!(600 + 12, cpu.index);
}

#[test]
fn test_loads_hex_char_sprite_0xf029() {
	let mut chip8 = make_chip8().0;
	chip8.load(&[
		0xF0, 0x29, //sets I to the sprite for "0"
		0xF5, 0x65 //loads the sprite into I..I+4 inclusive
		]
	);
	let mut cpu = chip8.cpu;
	cpu.emulate_cycle();
	cpu.emulate_cycle();
	assert_eq!(vec![
			//0
			0xF0,
			0x90,
			0x90,
			0x90,
			0xF0
	], cpu.registers[0..5].iter().map(|&x| x).collect::<Vec<u8>>());
}

#[test]
fn test_sets_sound_timer_to_vx_0xfx18() {
	let mut chip8 = make_chip8().0;
	chip8.load(&[
		0xf4, 0x18
		]
	);
	let mut cpu = chip8.cpu;
	cpu.registers[0x4] = 4;
	cpu.emulate_cycle();
	assert_eq!(4, cpu.sound_timer);
}

#[test]
fn test_renders_inbuilt_sprite_0() {
	let mut chip8 = make_chip8().0;
	chip8.load(&[
		0xf0, 0x29,
		0xD0, 0x05
		]
	);
	let mut cpu = &mut chip8.cpu;
	cpu.emulate_cycle();
	cpu.emulate_cycle();
	assert_eq!(&[0,0,0,0, 0,0,0,0, 1,1,1,1, 0,0,0,0], &cpu.gfx[0..16]);

}