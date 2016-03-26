extern crate rand;

mod vm;

use std::env;
use std::fs::File;
use std::io::Read;

use vm::Chip8;

pub fn main() {
	let args_vec : Vec<_> = env::args().collect();
	if args_vec.len() != 2 {
		println!("Usage:\n{}: GAME_PATH", args_vec[0]);
	}
	println!("Loading game at {}", args_vec[1]);
	let game_path = &args_vec[1];
	let mut f = File::open(game_path).unwrap();
	let mut data = Vec::new();
	f.read_to_end(&mut data);
	let mut chip8 = Chip8::new();
	chip8.load(&data);
	chip8.run();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
