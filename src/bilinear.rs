use crate::smoke_data::Pair;

// Gets horizontal velocity (U) from the left face of cell(x, y)
pub fn get_u(x: usize, y: usize, map: &[Pair], width: usize, height: usize) -> f32 {
  let x = x.clamp(0, width);
  let y = y.clamp(0, height - 1);
  let vi = x + y * (width + 1);
  return map[vi].left;
}

// Gets vertical velocity (V) from the top face of cell(x, y)
pub fn get_v(x: usize, y: usize, map: &[Pair], width: usize, height: usize) -> f32 {
    let x = x.clamp(0, width - 1);
    let y = y.clamp(0, height);
    
    let vi = x + y * (width + 1);
    
    map[vi].top
}

pub fn sample_u(px: f32, py: f32, map: &[Pair], width: usize, height: usize) -> f32 {
    // U velocities are centered vertically on the face, so we shift Y by 0.5
    let sample_y = py - 0.5;

    // Find the 4 neighboring grid points
    let x0 = px.floor() as usize;
    let y0 = sample_y.floor() as usize;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    // Fractional distances for lerping
    let tx = px - x0 as f32;
    let ty = sample_y - y0 as f32;

    // Fetch the 4 surrounding U velocities
    let u00 = get_u(x0, y0, map, width, height);
    let u10 = get_u(x1, y0, map, width, height);
    let u01 = get_u(x0, y1, map, width, height);
    let u11 = get_u(x1, y1, map, width, height);

    // Bilinear interpolation
    let u0 = lerp(u00, u10, tx); // Bottom edge
    let u1 = lerp(u01, u11, tx); // Top edge
    lerp(u0, u1, ty)             // Vertical blend
}

pub fn sample_v(px: f32, py: f32, map: &[Pair], width: usize, height: usize) -> f32 {
  // V velocities are centered horizontally on the face, so we shift X by 0.5
  let sampleX = px - 0.5;

  let x0 = sampleX.floor() as usize;
  let y0 = py.floor() as usize;
  let x1 = x0 + 1;
  let y1 = y0 + 1;

  let tx = sampleX - x0 as f32;
  let ty = py - y0 as f32;

  let v00 = get_v(x0, y0, map, width, height);
  let v10 = get_v(x1, y0, map, width, height);
  let v01 = get_v(x0, y1, map, width, height);
  let v11 = get_v(x1, y1, map, width, height);

  let v0 = lerp(v00, v10, tx);
  let v1 = lerp(v01, v11, tx);
  lerp(v0, v1, ty)
}
  
pub fn clamp01(num: f32)-> f32 {
  num.clamp(0.0,1.0)
}

pub fn lerp(start: f32, stop: f32, amt: f32) -> f32 {
  start + (stop - start) * amt
}

pub fn sampleSmoke(map: &[f32], px: f32, py: f32, width: usize, height: usize) -> f32{
  // move point so it's always in the top left quadrant
  let px = px - 0.5;
	let py = py - 0.5;
	
  let x = px.floor() as usize;
  let y = py.floor() as usize;
  let xFrac = clamp01(px - x as f32);
  let yFrac = clamp01(py - y as f32);
  
  let x0 = x.clamp(0, width - 1);
  let x1 = (x + 1).clamp(0, width - 1);
  let y0 = y.clamp(0, height - 1);
  let y1 = (y + 1).clamp(0, height - 1);
  
  let bottomLeft  = map[x0 + y0 * width];
  let bottomRight = map[x1 + y0 * width];
  let topLeft     = map[x0 + y1 * width];
  let topRight    = map[x1 + y1 * width];

  let interpolatedTop = lerp(topLeft, topRight, xFrac);
	let interpolatedBottom = lerp(bottomLeft, bottomRight, xFrac);
  lerp(interpolatedBottom, interpolatedTop, yFrac)
}