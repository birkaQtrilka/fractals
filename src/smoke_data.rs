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

#[derive(Clone, Copy)]
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

pub struct CellData {
  pub solid: bool,
  pub x: usize,
  pub y: usize,
  pub total: f32,
  pub flowt: f32,
  pub flowl: f32,
  pub flowb: f32,
  pub flowr: f32,
  pub k: f32,
  pub velocities: Cell
}

impl CellData {
  pub fn new(
    solid: bool,
    x: usize,
    y: usize,
    total: f32,
    flowt: f32,
    flowl: f32,
    flowb: f32,
    flowr: f32,
    k: f32,
    velocities: Cell
  ) -> CellData {
    CellData {
      solid,
      x,
      y,
      total,
      flowt,
      flowl,
      flowb,
      flowr,
      k,
      velocities
    }
  }
}
