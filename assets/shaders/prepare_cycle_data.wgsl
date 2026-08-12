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
@group(0) @binding(2) var<storage, read> velocities: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> rhs: array<f32>;
@group(0) @binding(4) var<storage, read_write> inv_total: array<f32>;

fn is_solid(idx: i32) -> bool {
  return solid_map[idx] != 0u;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let p = vec2<i32>(global_id.xy);
  
  if (p.x >= uniforms.width || p.y >= uniforms.height) {
    return;
  }
  
  let i = p.y * uniforms.width + p.x;

  // Solid cells don't have divergence or pressure
  if (is_solid(i)) {
    rhs[i] = 0.0;
    inv_total[i] = 0.0;
    return;
  }

  var total: f32 = 0.0;
  if (p.y > 0 && !is_solid(i - uniforms.width)) { total += 1.0; }
  if (p.x > 0 && !is_solid(i - 1)) { total += 1.0; }
  if (p.x + 1 < uniforms.width && !is_solid(i + 1)) { total += 1.0; }
  if (p.y + 1 < uniforms.height && !is_solid(i + uniforms.width)) { total += 1.0; }

  // .x maps to `top`, .y maps to `left` in our Rust `Pair` struct
  let vi = p.x + p.y * (uniforms.width + 1);
  let top = velocities[vi].x;
  let left = velocities[vi].y;
  let bottom = velocities[vi + uniforms.width + 1].x;
  let right = velocities[vi + 1].y;
  
  let div = right - left + bottom - top;

  rhs[i] = -uniforms.ady * div;
  
  if (total > 0.0) {
    inv_total[i] = 1.0 / total;
  } else {
    inv_total[i] = 0.0;
  }
}