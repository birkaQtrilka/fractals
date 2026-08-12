// pressure_solver_gpu.rs
use ogl33::*;
use crate::compute_ext::{ComputeExt, GL_SHADER_STORAGE_BUFFER, GL_SHADER_STORAGE_BARRIER_BIT};
use crate::learn_opengl::{ComputeProgram};

pub struct PressureSolverGpu {
  width: i32,
  height: i32,
  prepare_prog: ComputeProgram,
  solve_prog: ComputeProgram,
  solid_map_ssbo: GLuint,
  divergence_ssbo: GLuint,
  rhs_ssbo: GLuint,
  inv_total_ssbo: GLuint,
  pressures_ssbo: GLuint,
  group_x: u32,
  group_y: u32,
}

fn make_ssbo(byte_len: usize) -> GLuint {
  let mut id = 0;
  unsafe {
    glGenBuffers(1, &mut id);
    glBindBuffer(GL_SHADER_STORAGE_BUFFER, id);
    glBufferData(GL_SHADER_STORAGE_BUFFER, byte_len as isize, std::ptr::null(), GL_DYNAMIC_DRAW);
  }
  id
}

impl PressureSolverGpu {
  pub fn new(width: usize, height: usize) -> Self {
    let size = width * height;
    let prepare_prog = ComputeProgram::from_source(
      include_str!("../assets/shaders/prepare_cycle_data.comp")
    ).expect("prepare shader failed");
    let solve_prog = ComputeProgram::from_source(
      include_str!("../assets/shaders/red_black_pressure.comp")
    ).expect("solve shader failed");

    PressureSolverGpu {
      width: width as i32,
      height: height as i32,
      prepare_prog,
      solve_prog,
      solid_map_ssbo: make_ssbo(size * 4),
      divergence_ssbo: make_ssbo(size * 4),
      rhs_ssbo: make_ssbo(size * 4),
      inv_total_ssbo: make_ssbo(size * 4),
      pressures_ssbo: make_ssbo(size * 4),
      group_x: (width as u32 + 7) / 8,
      group_y: (height as u32 + 7) / 8,
    }
  }

  pub fn upload_solid_map(&self, solid_map: &[bool]) {
    let as_u32: Vec<u32> = solid_map.iter().map(|&b| b as u32).collect();
    self.upload(self.solid_map_ssbo, &as_u32);
  }

  pub fn upload_divergence(&self, divergence: &[f32]) {
    self.upload(self.divergence_ssbo, divergence);
  }

  // pressures persist frame to frame (used as the solve's initial guess), so
  // only upload once at startup, or after they're reset elsewhere
  pub fn upload_pressures(&self, pressures: &[f32]) {
    self.upload(self.pressures_ssbo, pressures);
  }

  fn upload<T>(&self, ssbo: GLuint, data: &[T]) {
    unsafe {
      glBindBuffer(GL_SHADER_STORAGE_BUFFER, ssbo);
      glBufferSubData(
        GL_SHADER_STORAGE_BUFFER, 0,
        (data.len() * std::mem::size_of::<T>()) as isize,
        data.as_ptr().cast(),
      );
    }
  }

  pub fn solve(&self, ext: &ComputeExt, density: f32, cell_size: f32, time_step: f32, iterations: u32) {
    unsafe {
      glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0, self.solid_map_ssbo);
      glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 1, self.divergence_ssbo);
      glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 2, self.rhs_ssbo);
      glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 3, self.inv_total_ssbo);

      glUseProgram(self.prepare_prog.0);
      let ady = density * cell_size / time_step;
      glUniform1i(self.prepare_prog.get_unif_location("width"), self.width);
      glUniform1i(self.prepare_prog.get_unif_location("height"), self.height);
      glUniform1f(self.prepare_prog.get_unif_location("ady"), ady);
      (ext.dispatch_compute)(self.group_x, self.group_y, 1);
      (ext.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);

      // rebind for solve pass (bindings 1..3 reused, add pressures at 1 instead of divergence)
      glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 1, self.pressures_ssbo);

      glUseProgram(self.solve_prog.0);
      let w_loc = self.solve_prog.get_unif_location("width");
      let h_loc = self.solve_prog.get_unif_location("height");
      let parity_loc = self.solve_prog.get_unif_location("parity");
      glUniform1i(w_loc, self.width);
      glUniform1i(h_loc, self.height);

      for _ in 0..iterations {
        for &parity in &[0, 1] {
          glUniform1i(parity_loc, parity);
          (ext.dispatch_compute)(self.group_x, self.group_y, 1);
          (ext.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);
        }
      }
    }
  }

  pub fn download_pressures(&self, out: &mut [f32]) {
    unsafe {
      glBindBuffer(GL_SHADER_STORAGE_BUFFER, self.pressures_ssbo);
      glGetBufferSubData(
        GL_SHADER_STORAGE_BUFFER, 0,
        (out.len() * 4) as isize,
        out.as_mut_ptr().cast(),
      );
    }
  }
}