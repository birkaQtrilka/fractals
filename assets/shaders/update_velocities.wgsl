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

fn is_solid(x: i32, y: i32) -> bool {
    if (x < 0 || x >= uniforms.width || y < 0 || y >= uniforms.height) { return true; }
    return solid_map[y * uniforms.width + x] != 0u;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let p = vec2<i32>(global_id.xy);
    if (p.x >= uniforms.width || p.y >= uniforms.height) { return; }

    let i = p.y * uniforms.width + p.x;
    let vi = p.x + p.y * (uniforms.width + 1);

    // Subtract pressure gradient from velocities
    // Pair.y is Left, Pair.x is Top
    
    // Update Left face
    if (p.x > 0 && !is_solid(p.x, p.y) && !is_solid(p.x - 1, p.y)) {
        velocities[vi].y -= uniforms.k * (pressures[i] - pressures[i - 1]);
    }

    // Update Top face
    if (p.y > 0 && !is_solid(p.x, p.y) && !is_solid(p.x, p.y - 1)) {
        velocities[vi].x -= uniforms.k * (pressures[i] - pressures[i - uniforms.width]);
    }
}