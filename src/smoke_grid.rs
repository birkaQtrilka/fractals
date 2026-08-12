use crate::bilinear;
use crate::smoke_data::{Pair, Cell, SolverData};
const INVALID: f32 = -100_000_000.0;

pub struct Grid {
  pub pressures: Vec<f32>,
  pub width: usize,
  pub height: usize,
  pub smoke: Vec<f32>,
  pub solid_map: Vec<bool>,
  velocities: Vec<Pair>,
  temp_velocities: Vec<Pair>,
  temp_smoke: Vec<f32>,
  
  cycle_data: Vec<SolverData>,
  density: f32,
  time_step: f32,
  pub cell_size: f32
}

impl Grid {
  pub fn new(
    width: usize,
    height: usize,
    density: f32,
    time_step: f32,
    cell_size: f32,
  )-> Grid {
    let size = width * height;
    let pressures = vec![0.0_f32; size];
    let mut velocities = vec![Pair::new(0.0, 0.0); (width+1) * (height+1)];
    let temp_velocities = vec![Pair::new(0.0, 0.0); (width+1) * (height+1)];
    let smoke = vec![0.0_f32; size];
    let temp_smoke = vec![0.0_f32; size];

    let solid_map = vec![false; size];
    let cycle_data = vec![SolverData { rhs: 0.0, inv_total: 0.0 }; size];

    let len = pressures.len();
    for i in 0..len {
      let x = i % width;
      let y = i / width;
      let vi = i + y;

      velocities[vi] = Pair::new(0.0, 0.0);
      if x == width-1 {
        velocities[vi + 1] = Pair::new(INVALID, 0.0);
      }
      if y == height - 1 {
        velocities[vi + width + 1] = Pair::new(0.0, INVALID);
      }
    }
    velocities[size + height + width] = Pair::new(INVALID, INVALID); 

    let mut grid = Grid {
      width,
      height,
      density,
      time_step,
      cell_size,
      pressures,
      velocities,
      temp_velocities,
      smoke,
      temp_smoke,
      solid_map,
      cycle_data,
    };
    grid.init_solid_map();
    grid
  }
  
  fn init_solid_map(&mut self) {
    for i in 0..self.width {
        self.solid_map[i] = true;
        self.solid_map[i + self.width * (self.height - 1)] = true;
    }
    
    for i in (0..(self.width * self.height)).step_by(self.width) {
        self.solid_map[i] = true;
        self.solid_map[i + self.width - 1] = true;
    }
  }
  
  fn sample_bilinear(&self, world_x: f32, world_y: f32, map: &[Pair])-> Pair {
    // Convert world space directly to grid space
    let px = world_x / self.cell_size;
    let py = world_y / self.cell_size;

    // Sample fields independently
    let vx = bilinear::sample_u(px, py, map, self.width, self.height);
    let vy = bilinear::sample_v(px, py, map, self.width, self.height);

    // Pair constructor is Pair(top, left), which translates to Pair(V, U).
    // Be careful with this order!
    Pair::new(vy, vx)
  }

  fn _get_divergence(&self, c: Cell) -> f32 {
    let gradient_x = (c.r - c.l) / self.cell_size ;
    let gradient_y = (c.t - c.b) / self.cell_size ;

    gradient_x + gradient_y
  }

  fn is_solid(&self, cell_index: usize) -> bool {
    self.solid_map[cell_index]
  }

  pub fn iterate_pressure_updates(&mut self) {
    self.prepare_cycle_data();

    for _ in 0..30 {
      self.update_pressures();
    }
  }

  fn prepare_cycle_data(&mut self) {
    // scale is constant, compute it once
    let ady = self.density * self.cell_size / self.time_step;

    for i in 0..self.pressures.len() {
      if self.is_solid(i) {
        self.cycle_data[i] = SolverData { rhs: 0.0, inv_total: 0.0 };
        continue;
      }
      
      let x = i % self.width;
      let y = i / self.width;
      
      let mut total = 0.0;
      if y > 0 && !self.is_solid(i - self.width) { total += 1.0; }
      if x > 0 && !self.is_solid(i - 1) { total += 1.0; }
      if x + 1 < self.width && !self.is_solid(i + 1) { total += 1.0; }
      if y + 1 < self.height && !self.is_solid(i + self.width) { total += 1.0; }
      
      // Calculate divergence just once
      let v = self.get_velocities(i);
      let div = v.r - v.l + v.b - v.t; 
      
      self.cycle_data[i] = SolverData {
        rhs: -ady * div,
        inv_total: if total > 0.0 { 1.0 / total } else { 0.0 }
      };
    }
  }

  fn update_pressures(&mut self) {
    for i in 0..self.pressures.len() {
      let d = self.cycle_data[i];
      if d.inv_total == 0.0 {
        self.pressures[i] = 0.0;
        continue;
      }
      
      // No bounds checks needed because the borders are solid!
      let p_sum =  self.pressures[i - self.width] 
                      + self.pressures[i - 1] 
                      + self.pressures[i + 1] 
                      + self.pressures[i + self.width];

      // Multiplication by inv_total is much faster than division by total
      self.pressures[i] = (p_sum + d.rhs) * d.inv_total; 
    }
  }

  pub fn density(&self) -> f32 {
    self.density
  }

  /// Same math as prepare_cycle_data's rhs computation, minus the ady scale
  /// (the solver applies that itself so it doesn't need a Grid reference).
  pub fn compute_divergence(&self) -> Vec<f32> {
    let mut divergence = vec![0.0_f32; self.width * self.height];
    for i in 0..self.pressures.len() {
      if self.is_solid(i) { continue; }
      let v = self.get_velocities(i);
      divergence[i] = v.r - v.l + v.b - v.t;
    }
    divergence
  }

  pub fn set_pressures(&mut self, pressures: &[f32]) {
    self.pressures.copy_from_slice(pressures);
  }

  pub fn update_velocities(&mut self) { 
    let k = self.time_step / (self.cell_size * self.density);
    
    for i in 0..self.pressures.len() {
      let y = i / self.width;
      let vi = i + y;

      if self.is_solid(i) {
        self.velocities[vi] = Pair::new(0.0, 0.0);
        continue;
      }

      let mut v_top = self.velocities[vi].top;
      let mut v_left = self.velocities[vi].left;

      if !self.is_solid(i - self.width) {
        v_top -= k * (self.pressures[i] - self.pressures[i - self.width]);
      } else {
        v_top = 0.0; 
      }

      if !self.is_solid(i - 1) {
        v_left -= k * (self.pressures[i] - self.pressures[i - 1]);
      } else {
        v_left = 0.0;
      }

      self.velocities[vi] = Pair::new(v_top, v_left);
    }
  }

  pub fn advect_velocities(&mut self) {
    for i in 0..self.pressures.len() {
      let y = i / self.width;

      let vi = i + y;

      if self.is_solid(i) {
        self.velocities[vi] = Pair::new(0.0, 0.0);
        continue;
      }

      let is_top_solid =  self.is_solid(i - self.width);
      let is_left_solid = self.is_solid(i - 1);

      let mut new_left_vel = 0.0;
      let x = i % self.width;

      if !is_left_solid {
        let face_u_x = (x as f32) * self.cell_size;
        let face_u_y = ((y as f32) + 0.5) * self.cell_size;

        let vel_at_face_u = self.sample_bilinear(face_u_x, face_u_y, &self.velocities);
        
        let prev_x = face_u_x - vel_at_face_u.left * self.time_step;
        let prev_y = face_u_y - vel_at_face_u.top * self.time_step;

        // sampleU expects grid coordinates, so we divide by cell_size
        new_left_vel = bilinear::sample_u(
          prev_x / self.cell_size, 
          prev_y / self.cell_size,
          &self.velocities,
          self.width, 
          self.height
        );
      }

      let mut new_top_vel = 0.0;
      if !is_top_solid {
        let face_v_x = ((x as f32) + 0.5) * self.cell_size;
        let face_v_y = (y as f32) * self.cell_size;

        let vel_at_face_v = self.sample_bilinear(face_v_x, face_v_y, &self.velocities);

        let prev_x = face_v_x - vel_at_face_v.left * self.time_step;
        let prev_y = face_v_y - vel_at_face_v.top * self.time_step;

        new_top_vel = bilinear::sample_v(
          prev_x / self.cell_size,
          prev_y / self.cell_size,
          &self.velocities,
          self.width,
          self.height
        );
      }

      self.temp_velocities[vi] = Pair::new(new_top_vel, new_left_vel);
    }

    self.velocities.copy_from_slice(&self.temp_velocities);
  }

  pub fn advect_smoke(&mut self) {
    for i in 0..self.pressures.len() {
      let x = i % self.width;
      let y = i / self.width;

      if self.is_solid(i) {
        self.temp_smoke[i] = 0.0;
        continue;
      }

      let center_x = ((x as f32) + 0.5) * self.cell_size;
      let center_y = ((y as f32) + 0.5) * self.cell_size;
      let vel_at_center = self.sample_bilinear(center_x, center_y, &self.velocities);
      
      let prev_x = center_x - vel_at_center.left * self.time_step;
      let prev_y = center_y - vel_at_center.top * self.time_step;
      // expects grid coordinates, so we divide by cell_size
      let advected_smoke = bilinear::sample_smoke(
        &self.smoke,
        prev_x / self.cell_size,
        prev_y / self.cell_size,
        self.width,
        self.height
      );
      self.temp_smoke[i] = advected_smoke;
    }

    self.smoke.copy_from_slice(&self.temp_smoke);
  }

  pub fn set_velocities(
    &mut self,
    pressure_index: usize,
    t: Option<f32>,
    l: Option<f32>,
    b: Option<f32>,
    r: Option<f32>,
) {
    let y = pressure_index / self.width;
    let vi = pressure_index + y;

    let vel = self.velocities[vi];
    self.velocities[vi] = Pair::new(t.unwrap_or(vel.top), l.unwrap_or(vel.left));

    let vel = self.velocities[vi + 1];
    self.velocities[vi + 1] = Pair::new(vel.top, r.unwrap_or(vel.left));

    let vel = self.velocities[vi + self.width + 1];
    self.velocities[vi + self.width + 1] = Pair::new(b.unwrap_or(vel.top), vel.left);
}

  pub fn get_velocities(&self, pressure_index: usize) -> Cell {
    let y = pressure_index / self.width;
    self.get_velocities_y(pressure_index, y)
  }

  fn get_velocities_y(&self, pressure_index: usize, y: usize) -> Cell {
    let vi = pressure_index + y;
    let t_l = self.velocities[vi];

    let t = t_l.top;
    let l = t_l.left;
    let b = self.velocities[vi + self.width + 1].top;
    let r = self.velocities[vi + 1].left;
    return Cell::new(t,l,b,r);
  }
}