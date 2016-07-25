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
		println!("Updating graphics...");
		let pixel_shapes = self._generate_pixels(gfx);
		let mut target = self.display.draw();

		for shape in pixel_shapes {
			let test_shape = shape;
			let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
			let vertex_buffer = glium::VertexBuffer::new(&self.display, &test_shape).unwrap();
			target.draw(&vertex_buffer, &indices, &self.program, &glium::uniforms::EmptyUniforms,
	            &Default::default()).unwrap();
		}

		target.finish().unwrap();

	}

	fn _generate_pixels(&self, gfx: GFX) -> Vec<Shape> {
		const PIXEL_DISPLAY_SIZE : f32 = 0.01;
		let mut out = Vec::new();
		let mut out_string = String::new();
        for xi in 0..63 {
            for yi in 0..31 {
                let x = xi as f32 * PIXEL_DISPLAY_SIZE;
                let y = yi as f32 * PIXEL_DISPLAY_SIZE;
                let pixel = gfx[yi][xi];
                if pixel != 0 {
                    out.push(vec![
					    Vertex { position: [x, y]},
					    Vertex { position: [x + PIXEL_DISPLAY_SIZE, y]},
					    Vertex { position: [x, y + PIXEL_DISPLAY_SIZE]}
				    ]);
				    out.push(vec![
					    Vertex { position: [x + PIXEL_DISPLAY_SIZE, y]},
					    Vertex { position: [x + PIXEL_DISPLAY_SIZE, y + PIXEL_DISPLAY_SIZE]},
					    Vertex { position: [x, y + PIXEL_DISPLAY_SIZE]}
				    ]);
				    out_string.push_str("0");
                } else {
				    out_string.push_str(" ");
                }
            }
            out_string.push_str("\n");
        }
		println!("--------------\n{}\n--------------", out_string);
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
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;
