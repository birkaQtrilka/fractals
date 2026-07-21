use std::{ffi::CString, fs};

use ogl33::*;
use crate::{Context, learn_opengl::ShaderProgram, mover::MoveData};


pub struct Mandelbrot {
  mover: MoveData,
  zoom_location: i32,
  pos_location: i32,
  max_iterations_location: i32,
  pub program: ShaderProgram,
  max_iterations: u32,
}

impl Mandelbrot {
  pub fn new(
    mover: MoveData,
    zoom_name: &str,
    pos_name: &str,
    frag_path: &str,
    max_iterations: u32,
  ) -> Mandelbrot 
    {
    let vert_shader = fs::read_to_string("assets/shaders/screen_uv.vs")
      .expect("Failed to read vertex shader file");
    let frag_shader = fs::read_to_string(frag_path)
      .expect("Failed to read fragment shader file");
    let program = ShaderProgram::from_vert_frag(&vert_shader, &frag_shader).expect("couldn't create program");
    program.use_program();

    let zoom_name = CString::new(zoom_name).unwrap();
    let pos_name = CString::new(pos_name).unwrap();
    let max_iterations_name = CString::new("max_iterations").unwrap();
    
    Mandelbrot {
      mover,
      zoom_location: unsafe { glGetUniformLocation(program.0, zoom_name.as_ptr()) },
      pos_location:  unsafe { glGetUniformLocation(program.0, pos_name.as_ptr()) },
      max_iterations_location: unsafe { glGetUniformLocation(program.0, max_iterations_name.as_ptr()) },
      max_iterations,
      program,
    }
  }
  
  pub fn update(&mut self, ctx: &Context) {
    // Process all currently pressed keys
    self.mover.update(ctx);

    fn to_emulated_double(num: f64) -> (f32, f32) {
      let center_x_hi = num as f32; 
      let center_x_lo: f32 = (num - center_x_hi as f64) as f32;
      (center_x_hi, center_x_lo)
    }

    unsafe {
      let split_zoom = to_emulated_double(self.mover.zoom);
      let split_x = to_emulated_double(self.mover.pos.0);
      let split_y = to_emulated_double(self.mover.pos.1);

      glUniform2f(self.zoom_location, split_zoom.0, split_zoom.1);
      glUniform4f(self.pos_location, split_x.0, split_x.1,split_y.0, split_y.1 );
      glUniform1ui(self.max_iterations_location, self.max_iterations);
    }
  }
}
