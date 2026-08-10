use rand::Rng;

use crate::bilinear;
use crate::smoke_data::{Pair};
const INVALID: f32 = -100_000_000.0;

#[derive(Clone, Copy)]
struct Cell{
  t: f32,
  l: f32,
  b: f32,
  r: f32,
}

impl Cell {
  fn new(  t: f32, l: f32, b: f32, r: f32) -> Cell {
    Cell {
      t, l, b, r
    }
  }
}

struct CellData {
  solid: bool,
  x: usize,
  y: usize,
  total: f32,
  flowt: f32,
  flowl: f32,
  flowb: f32,
  flowr: f32,
  K: f32,
  velocities: Cell
}

impl CellData {
  fn new(
    solid: bool,
    x: usize,
    y: usize,
    total: f32,
    flowt: f32,
    flowl: f32,
    flowb: f32,
    flowr: f32,
    K: f32,
    velocities: Cell
  ) -> CellData {
    CellData {
      solid,
      x,
      y,
      total,
      flowt,
      flowl,
      flowb,
      flowr,
      K,
      velocities
    }
  }
}

struct Grid {
  pressures: Vec<f32>,
  solid_map: Vec<bool>,
  velocities: Vec<Pair>, //to do: flatten it later
  temp_velocities: Vec<Pair>,
  smoke: Vec<f32>,
  temp_smoke: Vec<f32>,
  
  cycle_data: Vec<CellData>,
  width: usize,
  height: usize,
  density: f32,
  time_step: f32,
  cell_size: (f32, f32)
}

impl Grid {
  fn new(
    width: usize,
    height: usize,
    density: f32,
    time_step: f32,
    cell_size: (f32, f32),
  )-> Grid {
    let size = width * height;
    let pressures = vec![0.0_f32; size];
    let mut velocities = vec![Pair::new( 0.0, 0.0 ); (width+1) * (height+1)];
    let temp_velocities = vec![Pair::new( 0.0, 0.0 ); (width+1) * (height+1)];
    let smoke = vec![0.0_f32; size];
    let temp_smoke = vec![0.0_f32; size];

    let solid_map = vec![false; size];
    let cycle_data = Vec::with_capacity(size);

    let l = pressures.len();
    for i in 0..l {
      let x = i % width;
      let y = i / width;
      let vi = i + y;

      velocities[vi] = Pair::new( 0.0, 0.0 );
      if x == width-1 {
        velocities[vi + 1] = Pair::new( INVALID, 0.0 );
      }
      if y == height - 1 {
        velocities[vi + width + 1] = Pair::new( 0.0, INVALID );
      }
    }
    velocities[size + height + width] = Pair::new( INVALID, INVALID ); 

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
      self.solid_map[i + self.width*(self.height - 1)] = true;      
    }
    
    for i in (0..(self.width * self.height)).step_by(self.width) { 
      self.solid_map[i] = true;
      self.solid_map[i + self.width - 1] = true;      
    }
    
  }
  
  fn to_index(&self, x: usize, y: usize)-> usize {
    return y * self.width + x;
  }

  fn sample_bilinear(&self, worldX: f32, worldY: f32, map: &[Pair])-> Pair {
    // Convert world space directly to grid space
    let px = worldX / self.cell_size.0;
    let py = worldY / self.cell_size.1;

    // Sample fields independently
    let vx = bilinear::sample_u(px, py, map, self.width, self.height);
    let vy = bilinear::sample_v(px, py, map, self.width, self.height);

    // Pair constructor is Pair(top, left), which translates to Pair(V, U).
    // Be careful with this order!
    Pair::new(vy, vx)
  }

  fn get_divergence(&self, c: Cell) -> f32 {
    let gradientX = (c.r - c.l) / self.cell_size.0 ;
    let gradientY = (c.t - c.b) / self.cell_size.1 ;

    gradientX + gradientY
  }

  fn is_solid(&self, cellIndex: usize) -> bool {
    self.solid_map[cellIndex]
  }

  fn get_pressures(&self, p_index: usize, x: usize, y: usize) -> Cell {
    
    let t = if y > 0 { 
      self.get_pressure(p_index - self.width) 
    } else { 
      self.get_pressure(p_index) 
    };
    
    let l = if x > 0 { 
      self.get_pressure(p_index - 1) 
    } else { 
      0.0
    };
    
    let r = if x + 1 < self.width { 
      self.get_pressure(p_index + 1) 
    } else { 
      self.get_pressure(p_index) 
    };
    
    let b = if y + 1 < self.height { 
      self.get_pressure(p_index + self.width) 
    } else { 
      self.get_pressure(p_index) 
    };
    
    Cell::new(t, l, b, r)
  }

  fn get_pressure(&self, pIndx: usize) -> f32 {
    self.pressures[pIndx]
  }

  fn iterate_pressure_updates(&mut self) {
    self.prepare_cycle_data();

    for _ in 0..30 {
      self.update_pressures();
    }
  }

  fn prepare_cycle_data(&mut self) {
    let l = self.pressures.len();
    
    let k = self.time_step / (self.cell_size.0 * self.density);

    self.cycle_data.clear();

    for i in 0..l {
      let x = i % self.width;
      let y = i / self.width;
      
      if self.is_solid(i) {
        self.cycle_data.push(CellData::new(
          true, x, y, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, Cell::new(0.0, 0.0, 0.0, 0.0)
        ));
        continue;
      }
      
      let f_t = if y > 0 && !self.is_solid(i - self.width) { 1.0 } else { 0.0 };
      let f_l = if x > 0 && !self.is_solid(i - 1) { 1.0 } else { 0.0 };
      
      let f_r = if x + 1 < self.width && !self.is_solid(i + 1) { 1.0 } else { 0.0 };
      let f_b = if y + 1 < self.height && !self.is_solid(i + self.width) { 1.0 } else { 0.0 };
      
      let total = f_t + f_l + f_b + f_r;
      
      self.cycle_data.push(CellData::new(
        false, x, y, total, f_t, f_l, f_b, f_r, k, self.get_velocities(i)
      ));
    }
  }

  fn update_pressures(&mut self) {
    let l = self.pressures.len();
    for i in 0..l {
      let d = &self.cycle_data[i];
      if d.solid || d.total == 0.0 {
        self.pressures[i] = 0.0;
        continue;
      }
      
      let v = d.velocities;
      let p = self.get_pressures(i, d.x, d.y);
      let pSum = p.t * d.flowt + p.l * d.flowl + p.r * d.flowr + p.b * d.flowb;
      let deltaVelocitySum = v.r * d.flowr - v.l * d.flowl + v.b * d.flowb - v.t * d.flowt; 

      self.pressures[i] = (pSum - self.density * self.cell_size.0 * deltaVelocitySum / self.time_step) / d.total; 
    }
  }

  fn update_velocities(&mut self) { 
    for i in 0..self.pressures.len() {
      let d = &self.cycle_data[i];
      let vi = i + d.y;

      if d.solid {
        self.velocities[vi] = Pair::new(0.0, 0.0);
        continue;
      }

      let mut vTop = self.velocities[vi].top;
      let mut vLeft = self.velocities[vi].left;

      let isTopSolid  = self.is_solid(i - self.width);
      let isLeftSolid = self.is_solid(i - 1);

      if !isTopSolid {
        let pc = self.pressures[i];
        let pt = self.pressures[i - self.width];
        vTop -= d.K * (pc - pt);
      } else {
        vTop = 0.0; 
      }

      if !isLeftSolid {
        let pc = self.pressures[i];
        let pl = self.pressures[i - 1];
        vLeft -= d.K * (pc - pl);
      } else {
        vLeft = 0.0;
      }

      self.velocities[vi] = Pair::new(vTop, vLeft);
    }
  }

  fn advect_velocities(&mut self) {
    for i in 0..self.pressures.len() {
      let d = &self.cycle_data[i];
      let vi = i + d.y;

      if d.solid {
        self.velocities[vi] = Pair::new(0.0, 0.0);
        continue;
      }

      let isTopSolid =  self.is_solid(i - self.width);
      let isLeftSolid = self.is_solid(i - 1);

      let mut newLeftVel = 0.0;
      if !isLeftSolid {
        let faceUX = (d.x as f32) * self.cell_size.0;
        let faceUY = ((d.y as f32) + 0.5) * self.cell_size.1;

        let velAtFaceU = self.sample_bilinear(faceUX, faceUY, &self.velocities);
        
        let prevX = faceUX - velAtFaceU.left * self.time_step;
        let prevY = faceUY - velAtFaceU.top * self.time_step;

        // sampleU expects grid coordinates, so we divide by cell_size
        newLeftVel = bilinear::sample_u(
          prevX / self.cell_size.0, 
          prevY / self.cell_size.1,
          &self.velocities,
          self.width, 
          self.height
        );
      }

      let mut newTopVel = 0.0;
      if !isTopSolid {
        let faceVX = ((d.x as f32) + 0.5) * self.cell_size.0;
        let faceVY = (d.y as f32) * self.cell_size.1;

        let velAtFaceV = self.sample_bilinear(faceVX, faceVY, &self.velocities);

        let prevX = faceVX - velAtFaceV.left * self.time_step;
        let prevY = faceVY - velAtFaceV.top * self.time_step;

        newTopVel = bilinear::sample_v(
          prevX / self.cell_size.0,
          prevY / self.cell_size.1,
          &self.velocities,
          self.width,
          self.height
        );
      }

      self.temp_velocities[vi] = Pair::new(newTopVel, newLeftVel);
    }

    self.velocities.copy_from_slice(&self.temp_velocities);
  }

  fn advect_smoke(&mut self) {
    for i in 0..self.pressures.len() {
      let x = i % self.width;
      let y = i / self.width;

      if self.is_solid(i) {
        self.temp_smoke[i] = 0.0;
        continue;
      }

      let centerX = ((x as f32) + 0.5) * self.cell_size.0;
      let centerY = ((y as f32) + 0.5) * self.cell_size.1;
      let velAtCenter = self.sample_bilinear(centerX, centerY, &self.velocities);
      
      let prevX = centerX - velAtCenter.left * self.time_step;
      let prevY = centerY - velAtCenter.top * self.time_step;
      // expects grid coordinates, so we divide by cell_size
      let smokeVel = bilinear::sampleSmoke(
        &self.smoke,
        prevX / self.cell_size.0,
        prevY / self.cell_size.1,
        self.width,
        self.height
      );
      self.temp_smoke[i] = smokeVel;
    }

    self.smoke.copy_from_slice(&self.temp_smoke);
  }

  fn set_velocities(
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

  fn get_velocities(&self, pressureIndex: usize) -> Cell {
    let y = pressureIndex / self.width;
    self.get_velocities_y(pressureIndex, y)
  }

  fn get_velocities_y(&self, pressureIndex: usize, y: usize) -> Cell {
    let vi = pressureIndex + y;
    let t_l = self.velocities[vi];

    let t = t_l.top;
    let l = t_l.left;
    let b = self.velocities[vi + self.width + 1].top;
    let r = self.velocities[vi + 1].left;
    return Cell::new(t,l,b,r);
  }
}