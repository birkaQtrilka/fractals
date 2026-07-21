use std::{ffi::CString, fs::{self, File}, io::Write};

use beryllium::events::{SDLK_DOWN, SDLK_LEFT, SDLK_RIGHT, SDLK_UP, SDLK_a, SDLK_q, SDLK_s, SDLK_w, SDLK_x, SDLK_z};
use ogl33::*;

use crate::{Context, learn_opengl::ShaderProgram};

pub trait Programm{
  fn update(&mut self,  ctx: &Context);
}

pub struct Mandelbrot {
  zoom: f64,
  zoom_speed: f64,
  pos: (f64, f64),
  move_speed: f64,
  zoom_location: i32,
  pos_location: i32,

  julia_const_location: i32,
  julia_const: (f32, f32),
  julia_const_speed: f32,

}

impl Mandelbrot {
  pub fn new(zoom_speed: f64, move_speed: f64, zoom_name: &str, pos_name: &str,
    julia_const_name: &str,
    julia_const_speed: f32,
    frag_path: &str) -> Mandelbrot {
    let vert_shader = fs::read_to_string("assets/shaders/screen_uv.vs")
      .expect("Failed to read vertex shader file");
    let frag_shader = fs::read_to_string(frag_path)
      .expect("Failed to read fragment shader file");
    let programm = ShaderProgram::from_vert_frag(&vert_shader, &frag_shader).expect("couldn't create programm");
    programm.use_program();

    let zoom_name = CString::new(zoom_name).unwrap();
    let pos_name = CString::new(pos_name).unwrap();
    let julia_const_name = CString::new(julia_const_name).unwrap();

    Mandelbrot {
      zoom: 1.0,
      zoom_speed,
      pos: (0.0, 0.0),
      move_speed,
      zoom_location: unsafe { glGetUniformLocation(programm.0, zoom_name.as_ptr()) },
      pos_location:  unsafe { glGetUniformLocation(programm.0, pos_name.as_ptr()) },
      julia_const_location:  unsafe { glGetUniformLocation(programm.0, julia_const_name.as_ptr()) },
      julia_const: (0.70176, 0.3842),
      julia_const_speed
    }
  }

  pub fn save(&self, mut file: &File) {
    let data = format!("{},{}\n", self.julia_const.0, self.julia_const.1);
    file.write_all( data.as_bytes()).expect("could not write to file");
  }

  pub fn to_emulated_double(num: f64) -> (f32, f32) {
    let center_x_hi = num as f32; 
    let center_x_lo: f32 = (num - center_x_hi as f64) as f32;
    (center_x_hi, center_x_lo)
  }

}

impl Programm for Mandelbrot {
  
  fn update(&mut self, ctx: &Context) {
    // Process all currently pressed keys
    let relative_speed = self.move_speed * self.zoom;
    let input = &ctx.input_handler;

    if input.is_key_active(SDLK_x) {
      self.zoom *= self.zoom_speed;
    }
    if input.is_key_active(SDLK_z) {
      self.zoom /= self.zoom_speed;
      if self.zoom > 1.0 {self.zoom = 1.0}
    }
    if input.is_key_active(SDLK_DOWN) {
      self.pos.1 -= relative_speed;
    }
    if input.is_key_active(SDLK_UP) {
      self.pos.1 += relative_speed;
    }
    if input.is_key_active(SDLK_RIGHT) {
      self.pos.0 += relative_speed;
    }
    if input.is_key_active(SDLK_LEFT) {
      self.pos.0 -= relative_speed;
    }
    if input.is_key_active(SDLK_a) {
      self.julia_const.0 -= self.julia_const_speed;
    }
    if input.is_key_active(SDLK_s) {
      self.julia_const.0 += self.julia_const_speed;
    }
    if input.is_key_active(SDLK_q) {
      self.julia_const.1 -= self.julia_const_speed;
    }
    if input.is_key_active(SDLK_w) {
      self.julia_const.1 += self.julia_const_speed;
    }

    fn to_emulated_double(num: f64) -> (f32, f32) {
      let center_x_hi = num as f32; 
      let center_x_lo: f32 = (num - center_x_hi as f64) as f32;
      (center_x_hi, center_x_lo)
    }

    unsafe {
      let split_zoom = to_emulated_double(self.zoom);
      let split_x = to_emulated_double(self.pos.0);
      let split_y = to_emulated_double(self.pos.1);

      glUniform2f(self.zoom_location, split_zoom.0, split_zoom.1);
      glUniform4f(self.pos_location, split_x.0, split_x.1,split_y.0, split_y.1 );
      glUniform2f(self.julia_const_location, self.julia_const.0, self.julia_const.1);
    }
  }
}
