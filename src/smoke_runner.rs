use std::{cell::{RefCell, RefMut}, rc::Rc};

use queues::IsQueue;

use crate::{Context, compute_ext::ComputeExt, input_handling::InputHandler, grid_gpu::GridGpu, program::Updater, smoke_drawer::SmokeDrawer, smoke_grid::Grid, smoke_interactor::GridInteractor};

pub struct Smoke {
  pub time_step: f32,
  pub accumulator: f32,
  grid: Rc<RefCell<GridGpu>>,
  drawer: SmokeDrawer,
  interactor: Rc<RefCell<GridInteractor>>,
  input: Rc<RefCell<InputHandler>>,
  compute_ext: Rc<ComputeExt>,
  pressure_iterations: u32,
}

impl Smoke {
  pub fn start(
    width: f32,
    height: f32,
    cell_count_x: usize,
    cell_count_y: usize,
    density: f32,
    time_step: f32,
    input: Rc<RefCell<InputHandler>>,
    compute_ext: Rc<ComputeExt>,
  ) -> Smoke {
    let cell_size = (width / cell_count_x as f32).min(height / cell_count_y as f32);

    let grid = Rc::new(RefCell::new(GridGpu::new(
      cell_count_x,
      cell_count_y,
      density, 
      time_step, 
      cell_size
    )));
    {
      Self::init_solid_map(&mut grid.borrow_mut());
    }

    let drawer = SmokeDrawer::new(Rc::clone(&grid), height);
    let interactor = Rc::new(RefCell::new(
      GridInteractor::new(Rc::clone(&grid), 30.0, 10.0)
    ));
    
    Smoke {
      time_step,
      accumulator: 0.0,
      drawer,
      grid,
      interactor,
      input,
      compute_ext,
      pressure_iterations: 30,
    }
  }

  pub fn apply_velocities(&self, grid: &mut RefMut<'_, GridGpu>, interactor: &mut RefMut<'_, GridInteractor>){
    while interactor.velocity_q.size() > 0 {
      let data = interactor.velocity_q.remove().expect("couldn't dequeue");
      grid.set_velocities(data.i,data.t, data.l, data.b, data.r);
    }

    let w = grid.width;
    let h = grid.height;
    let bar_height = 10;
    let bar_center_offset = 0;
    let bar_start = (w * h) / 2 - (w * bar_height / 2) - (w * bar_center_offset) + 5;
    for i in 0..bar_height {
      grid.set_velocities( bar_start + w * i,None, Some(600.0), None, None);
      grid.set_smoke(bar_start + w * i + 1, 1.0);
    }

  }

  fn init_solid_map(grid: &mut RefMut<'_, GridGpu>) {
    let radius = 10;
    let w = grid.width;
    let h = grid.height;
    let center_x = w / 2;
    let center_y = h / 2;
    
    for y in 0..h {
      for x in 0..w {
        let dx = x as i32 - center_x as i32;
        let dy = y as i32 - center_y as i32;
        let distance_squared = dx * dx + dy * dy;
        
        if distance_squared <= (radius * radius) {
          grid.solid_map[y * w + x] = true;
        }
      }
    }

    grid.upload_solid_map();
  }
  
  pub fn apply_smoke(&self, grid: &mut RefMut<'_, GridGpu>, interactor: &mut RefMut<'_, GridInteractor>){
    while interactor.smoke_q.size() > 0 {
      let data = interactor.smoke_q.remove().expect("couldn't dequeue");
      grid.set_smoke(data.i, data.t);
    }
    
  }
}

impl Updater for Smoke {
  fn on_enable(&mut self) {
    GridInteractor::attach(&self.interactor, &mut self.input.borrow_mut());
  }

  fn on_disable(&mut self) {
    GridInteractor::detach(&self.interactor, &mut self.input.borrow_mut());
  }

  fn update(&mut self, ctx: &Context) {
    self.accumulator += ctx.delta_time;

    while self.accumulator >= self.time_step {
      {
        let mut grid_mut = self.grid.borrow_mut();
        let mut interactor_mut = self.interactor.borrow_mut();

        self.apply_velocities(&mut grid_mut, &mut interactor_mut);
        self.apply_smoke(&mut grid_mut, &mut interactor_mut);

        // GridGpu now encapsulates the entire compute pass seamlessly.
        grid_mut.step(&self.compute_ext, self.pressure_iterations);
      }
      self.accumulator -= self.time_step;
    }
    
    self.drawer.update(ctx);
  }
}
