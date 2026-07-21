use std::{ffi::CString, fs::{self, File, OpenOptions}, io::{BufRead, BufReader, Seek, SeekFrom, Write}};

use beryllium::events::{SDLK_a, SDLK_j, SDLK_k, SDLK_o, SDLK_p, SDLK_q, SDLK_s, SDLK_w};
use ogl33::*;

use crate::{Context, input_handling::PressState, learn_opengl::ShaderProgram, mover::MoveData};

pub struct JuliaSet {
  base: Mandelbrot,

  julia_const_location: i32,
  julia_const: (f32, f32),
  julia_const_speed: f32,

  saved_julia_consts: Vec<(f32, f32)>,
  saved_julia_const_index: usize,
  save_file: File,
}

impl JuliaSet {
  pub fn new(
    mover: MoveData,
    zoom_name: &str,
    pos_name: &str,
    frag_path: &str,
    julia_const_name: &str,
    julia_const_speed: f32,
    save_file_path: &str,
    max_iterations: u32,
  ) -> JuliaSet {
    let julia_const_name = CString::new(julia_const_name).unwrap();
    let mandelbrot = Mandelbrot::new(mover, zoom_name, pos_name, frag_path, max_iterations);
    
    let save_file = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .open(save_file_path)
      .expect("file couldn't be created or opened");

    JuliaSet {
      julia_const_location:  unsafe { glGetUniformLocation(mandelbrot.program.0, julia_const_name.as_ptr()) },
      julia_const: (0.70176, 0.3842),
      julia_const_speed,
      saved_julia_consts: Vec::new(),
      saved_julia_const_index: 0,
      base: mandelbrot,
      save_file
     }
  }

  pub fn save(&self, mut file: &File) {
    file.seek(SeekFrom::End(0)).expect("could not move cursor");
    let data = format!("{},{}\n", self.julia_const.0, self.julia_const.1);
    file.write_all( data.as_bytes()).expect("could not write to file");
  }

  pub fn read_save(mut file: &File) -> Vec<(f32, f32)> {
    let reader = BufReader::new(file);
    file.seek(SeekFrom::Start(0)).expect("could not move cursor");

    let data_list: Vec<(f32, f32)> = reader
      .lines()
      .filter_map(|line| line.ok()) // Handle errors, skip bad lines
      .filter(|line| !line.trim().is_empty())
      .filter_map(|line| {
          let parts: Vec<&str> = line.split(',').collect();
          if parts.len() == 2 {
              let x = parts[0].trim().parse::<f32>().ok()?;
              let y = parts[1].trim().parse::<f32>().ok()?;
              Some((x, y))
          } else {
              None
          }
      })
      .collect();
    data_list
  }

  pub fn check_for_save(&mut self, ctx: &Context) {
    if ctx.input_handler.get_key(SDLK_o).state == PressState::Down {
      self.saved_julia_consts = JuliaSet::read_save(&self.save_file);
      self.saved_julia_const_index = self.saved_julia_const_index.clamp(0, self.saved_julia_consts.len()-1); 
    }
    if self.saved_julia_consts.len() == 0 {
      return;
    }
    
    if ctx.input_handler.get_key(SDLK_j).state == PressState::Down && self.saved_julia_const_index > 0{
      self.saved_julia_const_index -= 1;
      self.saved_julia_const_index = self.saved_julia_const_index.clamp(0, self.saved_julia_consts.len()-1);
      self.julia_const = self.saved_julia_consts.get(self.saved_julia_const_index).copied().unwrap();
    }
    if ctx.input_handler.get_key(SDLK_k).state == PressState::Down {
      self.saved_julia_const_index += 1;
      self.saved_julia_const_index = self.saved_julia_const_index.clamp(0, self.saved_julia_consts.len()-1);    
      self.julia_const = self.saved_julia_consts.get(self.saved_julia_const_index).copied().unwrap();
    }
  }

  pub fn update(&mut self, ctx: &Context) {
    let input = &ctx.input_handler;

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
    self.base.update(ctx);
    unsafe {
      glUniform2f(self.julia_const_location, self.julia_const.0, self.julia_const.1);
    }

    self.check_for_save(&ctx);
    
    if ctx.input_handler.get_key(SDLK_p).state == PressState::Down {
      self.save( &self.save_file);
    }
  }

}

pub struct Mandelbrot {
  mover: MoveData,
  zoom_location: i32,
  pos_location: i32,
  max_iterations_location: i32,
  program: ShaderProgram,
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
