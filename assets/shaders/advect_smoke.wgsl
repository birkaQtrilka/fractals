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
@group(0) @binding(3) var<storage, read> smoke: array<f32>;
@group(0) @binding(4) var<storage, read_write> temp_smoke: array<f32>;

fn is_solid(idx: i32) -> bool {
  return solid_map[idx] != 0u;
}

fn get_u(x_in: i32, y_in: i32) -> f32 {
  let x = clamp(x_in, 0, uniforms.width);
  let y = clamp(y_in, 0, uniforms.height - 1);
  return velocities[x + y * (uniforms.width + 1)].y;
}

fn get_v(x_in: i32, y_in: i32) -> f32 {
  let x = clamp(x_in, 0, uniforms.width - 1);
  let y = clamp(y_in, 0, uniforms.height);
  return velocities[x + y * (uniforms.width + 1)].x;
}

fn sample_u(px: f32, py: f32) -> f32 {
  let sample_y = py - 0.5;
  let x0 = i32(floor(px));
  let y0 = i32(floor(sample_y));
  let x1 = x0 + 1;
  let y1 = y0 + 1;
  let tx = px - f32(x0);
  let ty = sample_y - f32(y0);
  
  return mix(
    mix(get_u(x0, y0), get_u(x1, y0), tx),
    mix(get_u(x0, y1), get_u(x1, y1), tx),
    ty
  );
}

fn sample_v(px: f32, py: f32) -> f32 {
  let sample_x = px - 0.5;
  let x0 = i32(floor(sample_x));
  let y0 = i32(floor(py));
  let x1 = x0 + 1;
  let y1 = y0 + 1;
  let tx = sample_x - f32(x0);
  let ty = py - f32(y0);
  
  return mix(
    mix(get_v(x0, y0), get_v(x1, y0), tx),
    mix(get_v(x0, y1), get_v(x1, y1), tx),
    ty
  );
}

fn sample_bilinear(world_x: f32, world_y: f32) -> vec2<f32> {
  let px = world_x / uniforms.cell_size;
  let py = world_y / uniforms.cell_size;
  return vec2<f32>(sample_v(px, py), sample_u(px, py));
}

fn sample_smoke(px_in: f32, py_in: f32) -> f32 {
  let px = px_in - 0.5;
  let py = py_in - 0.5;
  
  let x = i32(floor(px));
  let y = i32(floor(py));
  
  let x_frac = clamp(px - f32(x), 0.0, 1.0);
  let y_frac = clamp(py - f32(y), 0.0, 1.0);

  let x0 = clamp(x, 0, uniforms.width - 1);
  let x1 = clamp(x + 1, 0, uniforms.width - 1);
  let y0 = clamp(y, 0, uniforms.height - 1);
  let y1 = clamp(y + 1, 0, uniforms.height - 1);

  let bottom_left  = smoke[x0 + y0 * uniforms.width];
  let bottom_right = smoke[x1 + y0 * uniforms.width];
  let top_left     = smoke[x0 + y1 * uniforms.width];
  let top_right    = smoke[x1 + y1 * uniforms.width];

  let interpolated_bottom = mix(bottom_left, bottom_right, x_frac);
  let interpolated_top = mix(top_left, top_right, x_frac);
  
  return mix(interpolated_bottom, interpolated_top, y_frac);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let p = vec2<i32>(global_id.xy);
  
  if (p.x >= uniforms.width || p.y >= uniforms.height) {
    return;
  }
  
  let i = p.y * uniforms.width + p.x;

  if (is_solid(i)) {
    temp_smoke[i] = 0.0;
    return;
  }

  let center_x = (f32(p.x) + 0.5) * uniforms.cell_size;
  let center_y = (f32(p.y) + 0.5) * uniforms.cell_size;
  
  let vel_at_center = sample_bilinear(center_x, center_y);
  
  let prev_x = center_x - vel_at_center.y * uniforms.time_step; // .y is left/U
  let prev_y = center_y - vel_at_center.x * uniforms.time_step; // .x is top/V

  temp_smoke[i] = sample_smoke(prev_x / uniforms.cell_size, prev_y / uniforms.cell_size);
}