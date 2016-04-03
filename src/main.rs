#[macro_use]
extern crate glium;
extern crate rand;
extern crate glutin;

mod vm;
mod ui;
mod gfx;
mod tests;

use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

use vm::{Chip8};
use ui::{Chip8UI};

pub fn main() {
	let args_vec : Vec<_> = env::args().collect();
	if args_vec.len() == 0 {
		println!("Please specify a path to a game file");
		return;
	}
	else if args_vec.len() != 2 {
		println!("Usage: {}: GAME_PATH", args_vec[0]);
		return;
	}
	println!("Loading game at {}...", args_vec[1]);
	let game_path = &args_vec[1];
	let mut f = File::open(game_path).unwrap();
	let mut data = Vec::new();

	//set up the chip8 with channels
	let (key_tx, key_rx) = mpsc::channel();
	let (gfx_tx, gfx_rx) = mpsc::channel();
	let mut chip8 = Chip8::new(key_rx, gfx_tx);

	//load the actual cartridge
	println!("Reading program data...");
	if let Err(e) = f.read_to_end(&mut data) {
		println!("Failed to read data: {}", e);
	}
	chip8.load(&data);
	
	println!("Program data loaded.");

	println!("Starting emulator");
	thread::spawn(move || chip8.run());
	println!("Emulator running.");

	println!("Starting session...");
	let session = Chip8UI::new(key_tx, gfx_rx);
	session.start_session();
}