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
@group(0) @binding(2) var<storage, read> pressures: array<f32>;
@group(0) @binding(3) var<storage, read_write> velocities: array<vec2<f32>>;

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
  let vi = p.x + p.y * (uniforms.width + 1);

  if (is_solid(i)) {
    velocities[vi] = vec2<f32>(0.0, 0.0);
    return;
  }

  var v_top = velocities[vi].x;
  var v_left = velocities[vi].y;

  if (!is_solid(i - uniforms.width)) {
    v_top -= uniforms.k * (pressures[i] - pressures[i - uniforms.width]);
  } else {
    v_top = 0.0;
  }

  if (!is_solid(i - 1)) {
    v_left -= uniforms.k * (pressures[i] - pressures[i - 1]);
  } else {
    v_left = 0.0;
  }

  velocities[vi] = vec2<f32>(v_top, v_left);
}