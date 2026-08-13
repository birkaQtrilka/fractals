struct DrawerUniforms {
  cell_size: f32,
  pixel_size: f32,
  grid_width: f32,
  grid_height: f32,
  res_x: f32,
  res_y: f32,
  _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: DrawerUniforms;
@group(1) @binding(0) var<storage, read> smoke_data: array<f32>;  

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
  var out: VertexOutput;
  let x = f32((vertex_index & 1u) << 2u);
  let y = f32((vertex_index & 2u) << 1u);
  out.uv = vec2<f32>(x * 0.5, 1.0 - (y * 0.5)); // Corrected Y for wgpu
  out.position = vec4<f32>(x - 1.0, y - 1.0, 0.0, 1.0);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let grid_w = i32(uniforms.grid_width / uniforms.cell_size);
  let grid_h = i32(uniforms.grid_height / uniforms.cell_size);
  
  let coords = vec2<i32>(in.uv * vec2<f32>(f32(grid_w), f32(grid_h)));
  let index = coords.y * grid_w + coords.x;
  
  // Safety check for bounds
  if (index < 0 || index >= grid_w * grid_h) { return vec4<f32>(0.0); }

  let val = smoke_data[index];
  return vec4<f32>(val, val, val, 1.0);
}