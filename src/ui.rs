use std::sync::mpsc::{Sender, Receiver};
use vm::{GFX, Key};

use glutin;
use glutin::{Event};

pub struct Chip8UI {
	key_sender: Sender<(Key, bool)>,
	gfx_receiver: Receiver<GFX>
}

impl Chip8UI {
	pub fn new(key_sender: Sender<(Key, bool)>, 
		gfx_receiver: Receiver<GFX>) -> Chip8UI {
		Chip8UI {
			key_sender: key_sender,
			gfx_receiver: gfx_receiver
		}
	}

	fn _acquire_window(&mut self) -> glutin::Window {
		let window = glutin::WindowBuilder::new()
							.with_dimensions(64 * 4, 32 * 4)
							.build()
							.unwrap();

		if let Err(e) = unsafe { window.make_current() } {
			println!("Making the window current failed: {:?}", e);
		}

		window
	}

	fn _update_graphics(&mut self, gfx: GFX) {
		//TODO! Make this work
		println!("GFX: {:?}", &gfx[0..2048]);
	}

	pub fn start_session(mut self) {
		let window = self._acquire_window();

		loop {
			//handle keyboard input if any
			for event in window.poll_events() {
				if let Event::KeyboardInput(state, num, _) = event {
					let up = state == glutin::ElementState::Pressed;
					let send_result = match num {
						11 => self.key_sender.send((Key::K0, up)), 
						2 => self.key_sender.send((Key::K1, up)),
						3 => self.key_sender.send((Key::K2, up)),
						4 => self.key_sender.send((Key::K3, up)),
						5 => self.key_sender.send((Key::K4, up)),
						6 => self.key_sender.send((Key::K5, up)),
						7 => self.key_sender.send((Key::K6, up)),
						8 => self.key_sender.send((Key::K7, up)),
						9 => self.key_sender.send((Key::K8, up)),
						10 => self.key_sender.send((Key::K9, up)),
						30 => self.key_sender.send((Key::A, up)),
						48 => self.key_sender.send((Key::B, up)),
						46 => self.key_sender.send((Key::C, up)),
						32 => self.key_sender.send((Key::D, up)),
						18 => self.key_sender.send((Key::E, up)),
						33 => self.key_sender.send((Key::F, up)),
						_ => (Ok(()))
					};
					if !send_result.is_ok() {
						println!("Failed to send event: {:?}", event);
					}
				}
			}

			match self.gfx_receiver.try_recv() {
				Ok(graphics_update) => self._update_graphics(graphics_update),
				_ => ()
			}
		}
	}
}