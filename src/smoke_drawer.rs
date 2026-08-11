use std::{cell::RefCell, fs, rc::Rc};

use ogl33::*;

use crate::{Context, learn_opengl::ShaderProgram, program::Updater, smoke_grid::Grid};


pub struct SmokeDrawer {
  grid: Rc<RefCell<Grid>>,
  pixel_size: f32,

  u_cell_size: i32,
  u_size: i32,
  u_resolution: i32,
  program: ShaderProgram,
  texture_id: u32,
}

impl SmokeDrawer {
  pub fn new(grid: Rc<RefCell<Grid>>, pixel_size: f32) -> SmokeDrawer {
    let vert_shader = fs::read_to_string("assets/shaders/screen_uv.vs")
      .expect("Failed to read vertex shader file");
    let frag_shader = fs::read_to_string("assets/shaders/smoke.fs")
      .expect("Failed to read fragment shader file");
    let program = ShaderProgram::from_vert_frag(&vert_shader, &frag_shader).expect("couldn't create program");
    program.use_program();

    let u_cell_size = program.get_unif_location("cellSize");
    let u_size = program.get_unif_location("size");
    let u_grid_texture = program.get_unif_location("gridTexture");
    let u_resolution = program.get_unif_location("resolution");
    unsafe { glUniform1i(u_grid_texture, 0); }

    // Create the OpenGL texture
    let mut texture_id = 0;
    unsafe {
      glGenTextures(1, &mut texture_id);
      glBindTexture(GL_TEXTURE_2D, texture_id);
      
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as i32);
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as i32);
    }

    SmokeDrawer {
      grid,
      u_cell_size,
      u_size,
      program,
      texture_id,
      pixel_size,
      u_resolution
    }
  }
}

impl Updater for SmokeDrawer {
  fn update(&mut self, ctx: &Context) {
    let grid_ref = self.grid.borrow();
    self.program.use_program();

    unsafe {
      glUniform1f(self.u_size, self.pixel_size );
      glUniform1f(self.u_cell_size, self.pixel_size / grid_ref.width as f32);
      glUniform2i(self.u_resolution,ctx.window_w, ctx.window_h);

      glActiveTexture(GL_TEXTURE0);
      glBindTexture(GL_TEXTURE_2D, self.texture_id);

      glTexImage2D(
        GL_TEXTURE_2D,
        0,
        GL_R32F as i32,
        grid_ref.width as i32,
        grid_ref.height as i32,
        0,
        GL_RED,
        GL_FLOAT,
        grid_ref.smoke.as_ptr() as *const _
      );
    }
  }
}