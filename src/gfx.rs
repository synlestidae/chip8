#[macro_use]
use glium;

use vm::GFX;
use glium::backend::glutin_backend::GlutinFacade;
use glium::DisplayBuild;
use glium::Surface;

#[derive(Copy, Clone)]
struct Vertex {
	position: [f32; 2]
}

implement_vertex!(Vertex, position);

type Shape = Vec<Vertex>;

pub struct Chip8GFX {
	program: glium::program::Program,
	display: GlutinFacade
}

impl Chip8GFX {
	pub fn new() -> Chip8GFX {
		let display = glium::glutin::WindowBuilder::new()
	        .with_dimensions(1024, 768)
	        .with_title(format!("CHIP8"))
	        .build_glium()
	        .unwrap();

	    let program = glium::Program::from_source(&display, 
			VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
			.unwrap();   

	    let mut frame = display.draw();
	    frame.clear_color(0.0, 0.0, 0.0, 1.0);
	    frame.finish().unwrap();

		Chip8GFX {
			program: program,
			display: display
		}
	}

	pub fn get_display<'a>(&'a mut self) -> &'a mut GlutinFacade {
		&mut self.display
	}
 
	pub fn update_graphics(&mut self, gfx: GFX) {
		let pixel_shapes = self._generate_pixels(gfx);
		let mut target = self.display.draw();

		let test_shape = &pixel_shapes[0];
		let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
		let vertex_buffer = glium::VertexBuffer::new(&self.display, &test_shape).unwrap();
		target.draw(&vertex_buffer, &indices, &self.program, &glium::uniforms::EmptyUniforms,
            &Default::default()).unwrap();

		target.finish();

	}

	fn _generate_pixels(&self, gfx: GFX) -> Vec<Shape> {
		let mut out = Vec::new();
		for i in 0..gfx.len() {
			let x = (i % 64) as f64;
			let y = (i / 32) as f64;
			out.push(vec![
				Vertex { position: [0.0, 0.0]},
				Vertex { position: [0.0, 1.0]},
				Vertex { position: [1.0, 0.0]},
			]);
		}

		out
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