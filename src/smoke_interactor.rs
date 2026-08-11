use std::{cell::RefCell, rc::Rc};
use queues::{IsQueue, Queue};

use crate::smoke_grid::Grid;

struct MouseEvent{
  button: u8,
}

#[derive(Clone)]
struct VelocityData { 
  i: usize, t: Option<f32>, l: Option<f32>, b: Option<f32>, r: Option<f32>
}

#[derive(Clone)]
struct SmokeData {
  i: usize, t: f32
}

struct GridInteractor {
  is_dragging: bool,
  last_mouse_pos: Option<(f32, f32)>,
  velocity_q: Queue<VelocityData>,
  smoke_q: Queue<SmokeData>,
  // Radius of the brush (in scaled pixels)
  brushRadius: f32, // Increased radius to act as a proper brush
  mousePressedBtn: u8,
  grid: Rc<RefCell<Grid>>,
}

impl GridInteractor {

  pub fn new(
    grid: Rc<RefCell<Grid>>,
    brushRadius: f32,
  ) -> GridInteractor {

    GridInteractor {
      last_mouse_pos: None,
      is_dragging: false,
      velocity_q: Queue::new(),
      smoke_q: Queue::new(),
      brushRadius,
      mousePressedBtn: 0,
      grid,
    }
  }

  // fn attach() {
  //   this.canvas.addEventListener('mousedown', this.onMouseDown);
  //   window.addEventListener('mousemove', this.onMouseMove);
  //   window.addEventListener('mouseup', this.onMouseUp);
  // }

  // detach() {
  //   this.canvas.removeEventListener('mousedown', this.onMouseDown);
  //   window.removeEventListener('mousemove', this.onMouseMove);
  //   window.removeEventListener('mouseup', this.onMouseUp);
  // }

  fn get_adjusted_pos(&self, e: MouseEvent)-> (f32,f32){
    (0.0,0.0)
  }

  fn onMouseDown(&mut self, e: MouseEvent) {
    self.is_dragging = true;
    self.mousePressedBtn = e.button;
    self.last_mouse_pos = Some(self.get_adjusted_pos(e));
  }

  fn onMouseUp(&mut self) {
    self.is_dragging = false;
    self.last_mouse_pos = None;
  }

  fn onMouseMove(&mut self, e: MouseEvent) {

    if !self.is_dragging || self.last_mouse_pos.is_none() {return;}
    let last_mouse_pos = self.last_mouse_pos.unwrap();
    // todo: get mouse pos
    let pos = (0.0_f32, 0.0_f32);//(e.clientX, e.clientY);
    let deltaX = pos.0 - last_mouse_pos.0;
    let deltaY = pos.1 - last_mouse_pos.1;
    self.last_mouse_pos = Some(pos);
    
    // Prevent unnecessary loops if the mouse didn't actually move
    if deltaX == 0.0 && deltaY == 0.0 {return;} 
    let grid = self.grid.borrow();

    let cw = grid.cell_size.0;
    let ch = grid.cell_size.1;

    // Calculate a bounding box of cells to avoid checking the entire grid
    let minX = ((pos.0 - self.brushRadius).floor() / cw).max(0.0).floor() as usize;
    let maxX = (((pos.0 + self.brushRadius).floor() / cw)).min((grid.width - 1) as f32).floor() as usize;;
    let minY = (((pos.1 - self.brushRadius).floor() / ch)).max(0.0).floor() as usize;;
    let maxY = (((pos.1 + self.brushRadius).floor() / ch)).min((grid.height - 1) as f32).floor() as usize;;

    if self.mousePressedBtn == 0 {
      for cy in minY..maxY {
        for cx in minX..maxX {
          let pIndex = cy * grid.width + cx;
          self.smoke_q.add(SmokeData {i: pIndex, t: 1.0}).expect("Cannot add to smoke action queue");
        }
      }
      return;
    }

    let forceX = deltaX * 30.0;
    let forceY = deltaY * 30.0;

    for cy in minY..maxY {
      for cx in minX..maxX {
        let pIndex = cy * grid.width + cx;
        let cellX_px = cx as f32 * cw;
        let cellY_px = cy as f32 * ch;

        // Define center points for the 4 edges of the current cell
        let edges = (
          (cellX_px + cw / 2.0, cellY_px),           // t
          (cellX_px, cellY_px + ch / 2.0),           // l
          (cellX_px + cw / 2.0, cellY_px + ch),      // b
          (cellX_px + cw, cellY_px + ch / 2.0),      // r
        );
        let (et, el, eb, er) = edges;

        let velocities = grid.get_velocities(pIndex);
        let mut t = Option::<f32>::None; 
        let mut l = Option::<f32>::None; 
        let mut b = Option::<f32>::None; 
        let mut r = Option::<f32>::None; 
        let mut changed = false;

        // Linear falloff helper: 1 at the center of the brush, scaling down to 0 at the edge
        let get_fall_off = |edge_x: f32, edge_y: f32| {
          let dist = ((pos.0 - edge_x).powi(2) + (pos.1 - edge_y).powi(2)).sqrt();
          if dist < self.brushRadius  {1.0 - (dist / self.brushRadius)} else  {0.0}
        };

        // Y-axis Velocities
        let falloffT = get_fall_off(et.0, et.1);
        if falloffT > 0.0 {
          t = Some(velocities.t + forceY * falloffT);
          changed = true;
        }

        let falloffB = get_fall_off(eb.0, eb.1);
        if falloffB > 0.0 {
          b = Some(velocities.b + forceY * falloffB);
          changed = true;
        }

        // X-axis Velocities
        let falloffL = get_fall_off(el.0, el.1);
        if falloffL > 0.0 {
          l = Some(velocities.l + forceX * falloffL);
          changed = true;
        }

        let falloffR = get_fall_off(er.0, er.1);
        if falloffR > 0.0 {
          r = Some(velocities.r + forceX * falloffR);
          changed = true;
        }

        // Update the grid cell only if it was affected by the brush
        if changed {
          self.velocity_q.add(VelocityData {i: pIndex, t, l, b, r}).expect("Cannot add to velocity action queue");
        }
      }
    }
  }
}
