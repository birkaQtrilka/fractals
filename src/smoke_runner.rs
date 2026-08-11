use std::{cell::RefCell, fs, rc::Rc};

use ogl33::*;
use crate::{Context, learn_opengl::ShaderProgram, program::Updater, smoke_grid::Grid};

pub struct Smoke {
  pub width: f32,
  // last_time: f32,
  // accumulator: f32,
  grid: Rc<RefCell<Grid>>,
  drawer: SmokeDrawer,
}

impl Smoke {
  pub fn start(
    width: f32,
    density: f32,
    time_step: f32,
  ) -> Smoke {
    let w = 40_usize;
    let grid = Rc::new(RefCell::new(Grid::new(
      w, 
      w,
      density, 
      time_step, 
      (width / w as f32, width / w as f32)
    )));
    let drawer = SmokeDrawer::new(Rc::clone(&grid), width);

    Smoke {
      width,
      // last_time: 0.0,
      // accumulator: 0.0,
      drawer,
      grid,
    }
  }


}

impl Updater for Smoke {
  fn update(&mut self, ctx: &Context) {
    let mut grid_mut = self.grid.borrow_mut();
    
    // if last_time == 0 {
      // last_time = 10;
    // }

    // let delta_time = current_time - last_time;
    // last_time = current_time;

    // if delta_time > 250 {
      // delta_time = 250; 
    // }

    // accumulator += delta_time;

    // let simulation_updated = false;

    // while accumulator >= fixed_time_step_ms {
      grid_mut.iterate_pressure_updates();

      // self.interactor?.applyVelocities();
      // self.interactor?.applySmoke();
      grid_mut.update_velocities();
      grid_mut.advect_velocities();
      grid_mut.advect_smoke();
      // accumulator -= fixed_time_step_ms;
      // simulation_updated = true;
    // }
    drop(grid_mut);
    // if simulation_updated {
      self.drawer.update(ctx);
    // }
  }
}

struct SmokeDrawer {
  grid: Rc<RefCell<Grid>>,
  pixel_size: f32,

  u_cell_size: i32,
  u_size: i32,
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
      pixel_size
    }
  }
}

impl Updater for SmokeDrawer {
  fn update(&mut self, ctx: &Context) {
    let grid_ref = self.grid.borrow();

    unsafe {
      glUniform1f(self.u_size, self.pixel_size );
      glUniform1f(self.u_cell_size, self.pixel_size / grid_ref.width as f32);

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
        grid_ref.pressures.as_ptr() as *const _
      );
    }
  }
}