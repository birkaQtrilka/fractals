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

// Declared to match the Rust BindGroup layout, even if unused in this pass
@group(0) @binding(1) var<storage, read> solid_map: array<u32>;

@group(0) @binding(2) var<storage, read_write> pressures: array<f32>;
@group(0) @binding(3) var<storage, read> rhs: array<f32>;
@group(0) @binding(4) var<storage, read> inv_total: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let p = vec2<i32>(global_id.xy);
  
  if (p.x >= uniforms.width || p.y >= uniforms.height) {
    return;
  }
  
  if ((p.x + p.y) % 2 != uniforms.parity) {
    return; // only this pass's color
  }

  let i = p.y * uniforms.width + p.x;
  let it = inv_total[i];
  
  // Boundary cells have inv_total == 0.0 (set in prepare step). 
  // This early return cleanly prevents out-of-bounds indexing for i - width etc.
  if (it == 0.0) { 
    pressures[i] = 0.0; 
    return; 
  }

  let p_sum = pressures[i - uniforms.width] 
            + pressures[i - 1]
            + pressures[i + 1]     
            + pressures[i + uniforms.width];

  pressures[i] = (p_sum + rhs[i]) * it;
}