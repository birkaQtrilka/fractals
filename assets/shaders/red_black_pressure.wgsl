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
@group(0) @binding(1) var<storage, read_write> pressures: array<f32>;
@group(0) @binding(2) var<storage, read> rhs: array<f32>;
@group(0) @binding(3) var<storage, read> inv_total: array<f32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let p = vec2<i32>(global_id.xy);
    if (p.x >= uniforms.width || p.y >= uniforms.height) { return; }
    
    // Red-Black parity check: (x + y) % 2
    if ((p.x + p.y) % 2 != uniforms.parity) {
        return;
    }

    let i = p.y * uniforms.width + p.x;
    if (inv_total[i] == 0.0) { return; }

    var sum: f32 = 0.0;
    // Sample neighbors
    if (p.y > 0) { sum += pressures[i - uniforms.width]; }
    if (p.x > 0) { sum += pressures[i - 1]; }
    if (p.x + 1 < uniforms.width) { sum += pressures[i + 1]; }
    if (p.y + 1 < uniforms.height) { sum += pressures[i + uniforms.width]; }

    // Gauss-Seidel iteration
    pressures[i] = (rhs[i] + sum) * inv_total[i];
}