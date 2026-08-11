use std::{cell::{RefCell, RefMut}, rc::Rc};

use queues::IsQueue;

use crate::{Context, input_handling::InputHandler, program::Updater, smoke_drawer::SmokeDrawer, smoke_grid::Grid, smoke_interactor::GridInteractor};

pub struct Smoke {
  pub time_step: f32,
  pub accumulator: f32,
  grid: Rc<RefCell<Grid>>,
  drawer: SmokeDrawer,
  interactor: Rc<RefCell<GridInteractor>>,
  input: Rc<RefCell<InputHandler>>
}

impl Smoke {
  pub fn start(
    width: f32,
    cell_count: usize,
    density: f32,
    time_step: f32,
    input: Rc<RefCell<InputHandler>>,
  ) -> Smoke {
    let grid = Rc::new(RefCell::new(Grid::new(
      cell_count, 
      cell_count,
      density, 
      time_step, 
      (width / cell_count as f32, width / cell_count as f32)
    )));
    {
      Self::init_solid_map(&mut grid.borrow_mut());
    }
    let drawer = SmokeDrawer::new(Rc::clone(&grid), width);
    let interactor = Rc::new(RefCell::new(
      GridInteractor::new(Rc::clone(&grid), 30.0, 10.0)
    ));
    
    Smoke {
      time_step,
      accumulator: 0.0,
      drawer,
      grid,
      interactor,
      input
    }
  }

  pub fn apply_velocities(&self, grid: &mut RefMut<'_, Grid>, interactor: &mut RefMut<'_, GridInteractor>){
    while interactor.velocity_q.size() > 0 {
      let data = interactor.velocity_q.remove().expect("couldn't dequeue");
      grid.set_velocities(data.i,data.t, data.l, data.b, data.r);
    }

    let w = grid.width;
    let bar_height = 10;
    let bar_center_offset = 0;
    let bar_start = w*w/2 -(w*bar_height/2) - (w*bar_center_offset) + 5;
    for i in 0..bar_height {
      grid.set_velocities( bar_start + w * i,None, Some(600.0), None, None);
      grid.smoke[bar_start + w * i + 1] = 1.0;
    }

  }

  fn init_solid_map(grid: &mut RefMut<'_, Grid>) {
    let radius = 10;
    let w = grid.width;
    let center_x = grid.width / 2;
    
    for y in 0..w {
      for x in 0..w {
        let dx = x as i32 - center_x as i32;
        let dy = y as i32 - center_x as i32;
        let distance_squared = dx * dx + dy * dy;
        
        if distance_squared <= (radius * radius) {
          grid.solid_map[y * w + x] = true;
        }
      }
    }
  }
  
  pub fn apply_smoke(&self, grid: &mut RefMut<'_, Grid>, interactor: &mut RefMut<'_, GridInteractor>){
    while interactor.smoke_q.size() > 0 {
      let data = interactor.smoke_q.remove().expect("couldn't dequeue");
      grid.smoke[data.i] = data.t;
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
      // Scoping the borrows so they drop at the end of each physical step loop
      {
        let mut grid_mut = self.grid.borrow_mut();
        let mut interactor_mut = self.interactor.borrow_mut();

        self.apply_velocities(&mut grid_mut, &mut interactor_mut);
        self.apply_smoke(&mut grid_mut, &mut interactor_mut);
        
        grid_mut.iterate_pressure_updates();
        grid_mut.update_velocities();
        grid_mut.advect_velocities();
        grid_mut.advect_smoke();
      }
      
      self.accumulator -= self.time_step;
    }
    
    self.drawer.update(ctx);
  }
}
