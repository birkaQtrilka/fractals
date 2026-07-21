use beryllium::events::{SDLK_DOWN, SDLK_LEFT, SDLK_RIGHT, SDLK_UP, SDLK_x, SDLK_z};

use crate::Context;

pub struct MoveData {
  pub zoom: f64,
  zoom_speed: f64,
  pub pos: (f64, f64),
  move_speed: f64,
}

impl MoveData {
  pub fn new(zoom_speed: f64, move_speed: f64)-> MoveData {
     MoveData { 
      zoom: 1.0,
      zoom_speed,
      pos: (0.0, 0.0),
      move_speed
    }
  }

  pub fn update(&mut self, ctx: &Context) {
    let relative_speed = self.move_speed * self.zoom;
    let input = &ctx.input_handler;

    if input.is_key_active(SDLK_x) {
      self.zoom *= self.zoom_speed;
    }
    if input.is_key_active(SDLK_z) {
      self.zoom /= self.zoom_speed;
      if self.zoom > 1.0 {self.zoom = 1.0}
    }
    if input.is_key_active(SDLK_DOWN) {
      self.pos.1 -= relative_speed;
    }
    if input.is_key_active(SDLK_UP) {
      self.pos.1 += relative_speed;
    }
    if input.is_key_active(SDLK_RIGHT) {
      self.pos.0 += relative_speed;
    }
    if input.is_key_active(SDLK_LEFT) {
      self.pos.0 -= relative_speed;
    }
  }
}