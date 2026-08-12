
use crate::{Context};

pub trait Updater {
  fn update(&mut self, ctx: &Context);
  fn on_enable(&mut self ) {}
  fn on_disable(&mut self) {}
}
