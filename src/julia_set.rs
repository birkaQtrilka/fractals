use std::{ffi::CString, fs::{File, OpenOptions}, io::{BufRead, BufReader, Seek, SeekFrom, Write}};

use beryllium::events::{SDLK_a, SDLK_j, SDLK_k, SDLK_o, SDLK_p, SDLK_q, SDLK_s, SDLK_w};
use ogl33::{glGetUniformLocation, glUniform2f};

use crate::{Context, input_handling::PressState, mover::MoveData, program::Mandelbrot};


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