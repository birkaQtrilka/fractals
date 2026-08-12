#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
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

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Cell{
  pub t: f32,
  pub l: f32,
  pub b: f32,
  pub r: f32,
}

impl Cell {
  pub fn new(t: f32, l: f32, b: f32, r: f32) -> Cell {
    Cell {
      t, l, b, r
    }
  }
}

#[derive(Clone, Copy, Default)]
pub struct SolverData {
  pub rhs: f32,
  pub inv_total: f32, // Precomputed 1.0 / total (to avoid slow divisions later)
}