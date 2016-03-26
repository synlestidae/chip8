extern crate rand;
extern crate glutin;

mod vm;

use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc;
use std::thread;
use glutin::Event;

use vm::{Chip8, Key};

pub fn main() {
	let args_vec : Vec<_> = env::args().collect();
	if args_vec.len() != 2 {
		println!("Usage:\n{}: GAME_PATH", args_vec[0]);
	}
	println!("Loading game at {}", args_vec[1]);
	let (key_tx, key_rx) = mpsc::channel();
	let game_path = &args_vec[1];

	let mut f = File::open(game_path).unwrap();
	let mut data = Vec::new();
	let mut chip8 = Chip8::new(key_rx);

	//load the actual cartridge
	f.read_to_end(&mut data);
	chip8.load(&data);
	
	println!("Game cartridge loaded. Setting up your session...");
	thread::spawn(move || {
		loop {
			let window = glutin::Window::new().unwrap();
			unsafe { window.make_current() };
			for event in window.wait_events() {
				if let Event::KeyboardInput(Released, num, _) = event {
					println!("Key num {}", num);
					match num {
						11 => key_tx.send(Key::K0), 
						2 => key_tx.send(Key::K1),
						3 => key_tx.send(Key::K2),
						4 => key_tx.send(Key::K3),
						5 => key_tx.send(Key::K4),
						6 => key_tx.send(Key::K5),
						7 => key_tx.send(Key::K6),
						8 => key_tx.send(Key::K7),
						9 => key_tx.send(Key::K8),
						10 => key_tx.send(Key::K9),
						30 => key_tx.send(Key::A),
						48 => key_tx.send(Key::B),
						46 => key_tx.send(Key::C),
						32 => key_tx.send(Key::D),
						18 => key_tx.send(Key::E),
						33 => key_tx.send(Key::F),
						_ => (Ok(()))
					};
				}
			}
		}
	});
	println!("Running");
	loop{};
	//chip8.run();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
