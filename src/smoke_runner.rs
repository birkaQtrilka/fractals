use std::{cell::RefCell, rc::Rc};

use crate::{Context, program::Updater, smoke_drawer::SmokeDrawer, smoke_grid::Grid};

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
    // let mut grid_mut = grid.borrow_mut();
    // grid_mut.smoke[2*w-5] = 1.0;
    // grid_mut.smoke[w*w/ 2 + w/2] = 1.0;
    // drop(grid_mut);
    let drawer = SmokeDrawer::new(Rc::clone(&grid), width);

    Smoke {
      width,
      // last_time: 0.0,
      // accumulator: 0.0,
      drawer,
      grid,
    }
  }

  // pub fn applyVelocities(){
  //   while(!this.velocity_q.isEmpty()) {
  //     const data = this.velocity_q.dequeue();
  //     if(!data) continue;
  //     this.grid.setVelocities(data.i,data.t, data.l, data.b, data.r);
  //   }
  // }
  
  // pub fn applySmoke(){
  //   while(!this.smoke_q.isEmpty()) {
  //     const data = this.smoke_q.dequeue();
  //     if(!data) continue;
  //     this.grid.smoke[data.i] = data.t
  //   }
  // }
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
    let w = grid_mut.width;
      grid_mut.set_velocities(w*w/ 2 + w/2 - 2, None, None, None, Some(600.0));
    // grid_mut.smoke[] = 1.0;
    grid_mut.smoke[w*w/ 2 + w/2] = 1.0;
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
