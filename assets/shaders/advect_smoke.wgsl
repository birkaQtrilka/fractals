struct GridUniforms {
  width: i32,
  height: i32,
  cell_size: f32,
  time_step: f32,
  ady: f32,
  k: f32,
  parity: i32,
  _padding: i32,
}

@group(0) @binding(0) var<uniform> uniforms: GridUniforms;
@group(0) @binding(1) var<storage, read> solid_map: array<u32>;
@group(0) @binding(2) var<storage, read> velocities: array<vec2<f32>>; // The UPDATED velocities
@group(0) @binding(3) var<storage, read> old_smoke: array<f32>;
@group(0) @binding(4) var<storage, read_write> new_smoke: array<f32>;

fn is_solid(idx: i32) -> bool {
  if (idx < 0 || idx >= uniforms.width * uniforms.height) { return true; }
  return solid_map[idx] != 0u;
}

// Sample velocity at any point (world space)
fn sample_vel(x: f32, y: f32) -> vec2<f32> {
  let px = x / uniforms.cell_size;
  let py = y / uniforms.cell_size;
  
  // Average horizontal velocity (Pair.y) to cell center
  let u = (velocities[i32(px) + i32(py) * (uniforms.width + 1)].y + 
           velocities[i32(px + 1.0) + i32(py) * (uniforms.width + 1)].y) * 0.5;
           
  // Average vertical velocity (Pair.x) to cell center
  let v = (velocities[i32(px) + i32(py) * (uniforms.width + 1)].x + 
           velocities[i32(px) + i32(py + 1.0) * (uniforms.width + 1)].x) * 0.5;
           
  return vec2<f32>(u, v);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let p = vec2<i32>(global_id.xy);
  if (p.x >= uniforms.width || p.y >= uniforms.height) { return; }
  
  let i = p.y * uniforms.width + p.x;
  if (is_solid(i)) {
    new_smoke[i] = 0.0;
    return;
  }

  // Backtrace from cell center
  let center_x = (f32(p.x) + 0.5) * uniforms.cell_size;
  let center_y = (f32(p.y) + 0.5) * uniforms.cell_size;
  
  let vel = sample_vel(center_x, center_y);
  
  let prev_x = center_x - uniforms.time_step * vel.x;
  let prev_y = center_y - uniforms.time_step * vel.y;
  
  // Bilinear sample the old smoke
  let px = clamp(prev_x / uniforms.cell_size - 0.5, 0.0, f32(uniforms.width - 1));
  let py = clamp(prev_y / uniforms.cell_size - 0.5, 0.0, f32(uniforms.height - 1));
  
  let x0 = i32(floor(px));
  let y0 = i32(floor(py));
  let x1 = min(x0 + 1, uniforms.width - 1);
  let y1 = min(y0 + 1, uniforms.height - 1);
  
  let tx = px - f32(x0);
  let ty = py - f32(y0);
  
  let s00 = old_smoke[y0 * uniforms.width + x0];
  let s10 = old_smoke[y0 * uniforms.width + x1];
  let s01 = old_smoke[y1 * uniforms.width + x0];
  let s11 = old_smoke[y1 * uniforms.width + x1];
  
  new_smoke[i] = mix(mix(s00, s10, tx), mix(s01, s11, tx), ty) * 0.99; // 0.99 for slight dissipation
}