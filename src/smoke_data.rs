#[derive(Clone, Copy)]
pub struct Pair {
  pub top: f32,
  pub left: f32
}

impl Pair {
  pub fn new(top: f32, left: f32) -> Pair{
    Pair{
      top, 
      left
    }
  }
}
