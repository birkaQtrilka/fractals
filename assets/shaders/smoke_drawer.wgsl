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
@group(1) @binding(0) var grid_texture: texture_2d<f32>;
// Sampler removed!

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
  var out: VertexOutput;
  let x = f32((vertex_index & 1u) << 2u);
  let y = f32((vertex_index & 2u) << 1u);
  
  // Standard UVs: (0,0) is top-left
  out.uv = vec2<f32>(x * 0.5, 1.0 - (y * 0.5));
  out.position = vec4<f32>(x - 1.0, y - 1.0, 0.0, 1.0);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // Use the uniform to prevent stripping
  let cell = uniforms.cell_size;

  // Convert UVs (0.0 to 1.0) to integer pixel coordinates (0 to width/height)
  let dims = vec2<f32>(textureDimensions(grid_texture));
  let coords = vec2<i32>(in.uv * dims);
  
  // textureLoad does not require a sampler and works with R32Float perfectly
  let smoke_val = textureLoad(grid_texture, coords, 0).r;
  
  return vec4<f32>(smoke_val, smoke_val, smoke_val, 1.0);
}