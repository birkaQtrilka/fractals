use std::{cell::{RefCell, RefMut}, rc::Rc};

use queues::IsQueue;

use crate::{Context, input_handling::InputHandler, program::Updater, smoke_drawer::SmokeDrawer, smoke_grid::Grid, smoke_interactor::GridInteractor};

pub struct Smoke {
  pub width: f32,
  // last_time: f32,
  // accumulator: f32,
  grid: Rc<RefCell<Grid>>,
  drawer: SmokeDrawer,
  interactor: Rc<RefCell<GridInteractor>>,
}

impl Smoke {
  pub fn start(
    width: f32,
    density: f32,
    time_step: f32,
    input: &mut InputHandler,
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
    let interactor = Rc::new(RefCell::new(
      GridInteractor::new(Rc::clone(&grid), 30.0, 100.0)
    ));
    GridInteractor::attach(&interactor, input);
    Smoke {
      width,
      drawer,
      grid,
      interactor
    }
  }

  pub fn apply_velocities(&self, grid: &mut RefMut<'_, Grid>, interactor: &mut RefMut<'_, GridInteractor>){
    while interactor.velocity_q.size() > 0 {
      let data = interactor.velocity_q.remove().expect("couldn't dequeue");
      grid.set_velocities(data.i,data.t, data.l, data.b, data.r);
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
  fn update(&mut self, ctx: &Context) {
    let mut grid_mut = self.grid.borrow_mut();
    let mut interactor_mut = self.interactor.borrow_mut();

    self.apply_velocities(&mut grid_mut, &mut interactor_mut);
    self.apply_smoke(&mut grid_mut, &mut interactor_mut);
    
    grid_mut.iterate_pressure_updates();
    grid_mut.update_velocities();
    grid_mut.advect_velocities();
    grid_mut.advect_smoke();
    
    drop(grid_mut);
    
    self.drawer.update(ctx);
  }
}
