use vm::GFX;

use glium;
use glium::backend::glutin_backend::GlutinFacade;
use glium::DisplayBuild;

struct Vertex {
	vertices: [f32; 4]
}

pub struct Chip8GFX {
	display: GlutinFacade
}

impl Chip8GFX {
	pub fn new() -> Chip8GFX {
		let display = glium::glutin::WindowBuilder::new()
	        .with_dimensions(1024, 768)
	        .with_title(format!("CHIP8"))
	        .build_glium()
	        .unwrap();

		Chip8GFX {
			display: display
		}
	}

	pub fn get_display<'a>(&'a mut self) -> &'a mut GlutinFacade {
		&mut self.display
	}
 
	pub fn update_graphics(&mut self, gfx: GFX) {
		panic!("Not implemented {:?}", &gfx[0..32]);
	}
}

const VERTEX_SHADER_SRC: &'static str = r#"
    #version 140

    in vec2 position;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
    #version 140

    out vec4 color;

    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;