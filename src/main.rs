extern crate rand;
extern crate glutin;

mod vm;
mod ui;

use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

use vm::{Chip8};
use ui::{Chip8UI};

pub fn main() {
	let args_vec : Vec<_> = env::args().collect();
	if args_vec.len() != 2 {
		println!("Usage:\n{}: GAME_PATH", args_vec[0]);
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
	
	println!("Program data loaded. Setting up your session...");
	let session = Chip8UI::new(key_tx, gfx_rx);
	session.start_session();

	println!("Running emulator");
	thread::spawn(move || chip8.run());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
