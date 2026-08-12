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
@group(0) @binding(3) var<storage, read_write> temp_velocities: array<vec2<f32>>;

fn is_solid(idx: i32) -> bool {
  return solid_map[idx] != 0u;
}

fn get_u(x_in: i32, y_in: i32) -> f32 {
  let x = clamp(x_in, 0, uniforms.width);
  let y = clamp(y_in, 0, uniforms.height - 1);
  let vi = x + y * (uniforms.width + 1);
  return velocities[vi].y; // y matches Pair.left
}

fn get_v(x_in: i32, y_in: i32) -> f32 {
  let x = clamp(x_in, 0, uniforms.width - 1);
  let y = clamp(y_in, 0, uniforms.height);
  let vi = x + y * (uniforms.width + 1);
  return velocities[vi].x; // x matches Pair.top
}

fn sample_u(px: f32, py: f32) -> f32 {
  let sample_y = py - 0.5;
  let x0 = i32(floor(px));
  let y0 = i32(floor(sample_y));
  let x1 = x0 + 1;
  let y1 = y0 + 1;
  
  let tx = px - f32(x0);
  let ty = sample_y - f32(y0);

  let u0 = mix(get_u(x0, y0), get_u(x1, y0), tx);
  let u1 = mix(get_u(x0, y1), get_u(x1, y1), tx);
  return mix(u0, u1, ty);
}

fn sample_v(px: f32, py: f32) -> f32 {
  let sample_x = px - 0.5;
  let x0 = i32(floor(sample_x));
  let y0 = i32(floor(py));
  let x1 = x0 + 1;
  let y1 = y0 + 1;
  
  let tx = sample_x - f32(x0);
  let ty = py - f32(y0);

  let v0 = mix(get_v(x0, y0), get_v(x1, y0), tx);
  let v1 = mix(get_v(x0, y1), get_v(x1, y1), tx);
  return mix(v0, v1, ty);
}

fn sample_bilinear(world_x: f32, world_y: f32) -> vec2<f32> {
  let px = world_x / uniforms.cell_size;
  let py = world_y / uniforms.cell_size;
  let vx = sample_u(px, py);
  let vy = sample_v(px, py);
  return vec2<f32>(vy, vx); // Returning (top, left) equivalents
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let p = vec2<i32>(global_id.xy);
  if (p.x >= uniforms.width || p.y >= uniforms.height) {
    return;
  }
  
  let i = p.y * uniforms.width + p.x;
  let vi = p.x + p.y * (uniforms.width + 1);

  if (is_solid(i)) {
    temp_velocities[vi] = vec2<f32>(0.0, 0.0);
    return;
  }

  let is_top_solid = is_solid(i - uniforms.width);
  let is_left_solid = is_solid(i - 1);

  var new_left_vel: f32 = 0.0;
  if (!is_left_solid) {
    let face_u_x = f32(p.x) * uniforms.cell_size;
    let face_u_y = (f32(p.y) + 0.5) * uniforms.cell_size;

    let vel_at_face_u = sample_bilinear(face_u_x, face_u_y);
    let prev_x = face_u_x - vel_at_face_u.y * uniforms.time_step; // .y is left/U
    let prev_y = face_u_y - vel_at_face_u.x * uniforms.time_step; // .x is top/V

    new_left_vel = sample_u(prev_x / uniforms.cell_size, prev_y / uniforms.cell_size);
  }

  var new_top_vel: f32 = 0.0;
  if (!is_top_solid) {
    let face_v_x = (f32(p.x) + 0.5) * uniforms.cell_size;
    let face_v_y = f32(p.y) * uniforms.cell_size;

    let vel_at_face_v = sample_bilinear(face_v_x, face_v_y);
    let prev_x = face_v_x - vel_at_face_v.y * uniforms.time_step;
    let prev_y = face_v_y - vel_at_face_v.x * uniforms.time_step;

    new_top_vel = sample_v(prev_x / uniforms.cell_size, prev_y / uniforms.cell_size);
  }

  temp_velocities[vi] = vec2<f32>(new_top_vel, new_left_vel);
}