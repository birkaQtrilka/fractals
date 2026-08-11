use std::{cell::RefCell, rc::Rc};
use queues::{IsQueue, Queue};

use crate::{input_handling::{InputHandler, MouseButton, MouseEventData, MouseSubscription}, smoke_grid::Grid};

#[derive(Clone)]
pub struct VelocityData {
  pub i: usize,
  pub t: Option<f32>,
  pub l: Option<f32>,
  pub b: Option<f32>,
  pub r: Option<f32>,
}

#[derive(Clone)]
pub struct SmokeData {
  pub i: usize,
  pub t: f32,
}

pub struct GridInteractor {
  is_dragging: bool,
  pub velocity_q: Queue<VelocityData>,
  pub smoke_q: Queue<SmokeData>,
  // Radius of the brush (in scaled pixels)
  brush_radius: f32,
  velocity_force: f32,
  mouse_pressed_btn: MouseButton,
  grid: Rc<RefCell<Grid>>,
  mouse_up_sub: Option<MouseSubscription>,
  mouse_down_sub: Option<MouseSubscription>,
  mouse_move_sub: Option<MouseSubscription>,
}

impl GridInteractor {
  pub fn new(grid: Rc<RefCell<Grid>>, brush_radius: f32, velocity_force: f32) -> GridInteractor {
    GridInteractor {
      is_dragging: false,
      velocity_q: Queue::new(),
      smoke_q: Queue::new(),
      brush_radius,
      velocity_force,
      mouse_pressed_btn: MouseButton::Left,
      grid,
      mouse_down_sub: None,
      mouse_move_sub: None,
      mouse_up_sub: None
    }
  }

  pub fn attach(self_rc: &Rc<RefCell<Self>>, input: &mut InputHandler) {
    let mut borrow = self_rc.borrow_mut();
    let h = Rc::clone(self_rc);
    borrow.mouse_down_sub = Some(input.on_mouse_down(move |e| {
      h.borrow_mut().on_mouse_down(e);
    }));
    let h = Rc::clone(self_rc);
    borrow.mouse_up_sub = Some(input.on_mouse_up(move |_e| {
      h.borrow_mut().on_mouse_up();
    }));
    let h = Rc::clone(self_rc);
    borrow.mouse_move_sub = Some(input.on_mouse_move(move |e| {
      h.borrow_mut().on_mouse_move(e);
    }));
  }

  pub fn detach(self_rc: &Rc<RefCell<Self>>, input: &mut InputHandler) {
    let (sub_up, sub_down, sub_move) = {
        let mut borrow = self_rc.borrow_mut();
        (
            std::mem::take(&mut borrow.mouse_up_sub),
            std::mem::take(&mut borrow.mouse_down_sub),
            std::mem::take(&mut borrow.mouse_move_sub),
        )
    };
    
    if let Some(s) = sub_up {
        input.unsubscribe_mouse(s);
    }
    if let Some(s) = sub_down {
        input.unsubscribe_mouse(s);
    }
    if let Some(s) = sub_move {
        input.unsubscribe_mouse(s);
    }
}

  fn on_mouse_down(&mut self, e: &MouseEventData) {
    self.is_dragging = true;
    // println!("{:?}", e.button.unwrap());
    self.mouse_pressed_btn = e.button.unwrap();
  }

  fn on_mouse_up(&mut self) {
    self.is_dragging = false;
  }

  fn on_mouse_move(&mut self, e: &MouseEventData) {
    if !self.is_dragging {
      return;
    }

    let pos = (e.x as f32, e.y as f32);

    // Prevent unnecessary loops if the mouse didn't actually move
    if e.delta_x == 0 && e.delta_y == 0 {
      return;
    }

    let grid = self.grid.borrow();

    let cw = grid.cell_size.0;
    let ch = grid.cell_size.1;

    // Calculate a bounding box of cells to avoid checking the entire grid
    let min_x = ((pos.0 - self.brush_radius).floor() / cw).max(0.0).floor() as usize;
    let max_x = (((pos.0 + self.brush_radius).floor() / cw))
      .min((grid.width - 1) as f32)
      .floor() as usize;
    let min_y = (((pos.1 - self.brush_radius).floor() / ch)).max(0.0).floor() as usize;
    let max_y = (((pos.1 + self.brush_radius).floor() / ch))
      .min((grid.height - 1) as f32)
      .floor() as usize;

    if self.mouse_pressed_btn == MouseButton::Right {
      for cy in min_y..max_y {
        for cx in min_x..max_x {
          let p_index = cy * grid.width + cx;
          self.smoke_q
            .add(SmokeData { i: p_index, t: 1.0 })
            .expect("Cannot add to smoke action queue");
        }
      }
      return;
    }

    let force_x = e.delta_x as f32 * self.velocity_force;
    let force_y = e.delta_y as f32 * self.velocity_force;

    for cy in min_y..max_y {
      for cx in min_x..max_x {
        let p_index = cy * grid.width + cx;
        let cell_x_px = cx as f32 * cw;
        let cell_y_px = cy as f32 * ch;

        // Define center points for the 4 edges of the current cell
        let edges = (
          (cell_x_px + cw / 2.0, cell_y_px), // t
          (cell_x_px, cell_y_px + ch / 2.0), // l
          (cell_x_px + cw / 2.0, cell_y_px + ch), // b
          (cell_x_px + cw, cell_y_px + ch / 2.0), // r
        );
        let (et, el, eb, er) = edges;

        let velocities = grid.get_velocities(p_index);
        let mut t = None;
        let mut l = None;
        let mut b = None;
        let mut r = None;
        let mut changed = false;

        // Linear falloff helper: 1 at the center of the brush, scaling down to 0 at the edge
        let get_fall_off = |edge_x: f32, edge_y: f32| {
          let dist = ((pos.0 - edge_x).powi(2) + (pos.1 - edge_y).powi(2)).sqrt();
          if dist < self.brush_radius {
            1.0 - (dist / self.brush_radius)
          } else {
            0.0
          }
        };

        // Y-axis Velocities
        let falloff_t = get_fall_off(et.0, et.1);
        if falloff_t > 0.0 {
          t = Some(velocities.t + force_y * falloff_t);
          changed = true;
        }

        let falloff_b = get_fall_off(eb.0, eb.1);
        if falloff_b > 0.0 {
          b = Some(velocities.b + force_y * falloff_b);
          changed = true;
        }

        // X-axis Velocities
        let falloff_l = get_fall_off(el.0, el.1);
        if falloff_l > 0.0 {
          l = Some(velocities.l + force_x * falloff_l);
          changed = true;
        }

        let falloff_r = get_fall_off(er.0, er.1);
        if falloff_r > 0.0 {
          r = Some(velocities.r + force_x * falloff_r);
          changed = true;
        }

        if !changed {
          continue;
        }

        self.velocity_q
          .add(VelocityData {
            i: p_index,
            t,
            l,
            b,
            r,
          })
          .expect("Cannot add to velocity action queue");
      }
    }
  }
}