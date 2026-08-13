use std::{cell::RefCell, rc::Rc};
use crate::{Context, grid_gpu::GridGpu, updater::Updater};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawerUniforms {
  cell_size: f32,
  pixel_size: f32,
  grid_width: f32,
  grid_height: f32,
  res_x: f32,
  res_y: f32,
  _padding: [f32; 2],
}

pub struct SmokeDrawer {
  grid: Rc<RefCell<GridGpu>>,
  pixel_size: f32,

  render_pipeline: wgpu::RenderPipeline,
  uniform_buffer: wgpu::Buffer,
  uniform_bind_group: wgpu::BindGroup,
  // Note: We no longer store a texture_bind_group here because it 
  // needs to be recreated (or swapped) per frame in update().
}

impl SmokeDrawer {
  pub fn new(
    grid: Rc<RefCell<GridGpu>>, 
    pixel_size: f32,
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
  ) -> SmokeDrawer {
    // 1. Uniform Buffer setup
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("SmokeDrawer Uniform Buffer"),
      size: std::mem::size_of::<DrawerUniforms>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    // 2. Shader Module
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("SmokeDrawer Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/smoke_drawer.wgsl").into()),
    });

    // 3. Render Pipeline
    // WGPU v30 will automatically derive the layout from the WGSL.
    // Group 0: Uniforms
    // Group 1: Storage Buffer (smoke_data)
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("SmokeDrawer Render Pipeline"),
      layout: None, 
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
          format: surface_format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview_mask: None,
      cache: None,
    });

    // 4. Uniform Bind Group (Group 0)
    // This one is static, so we can create it once.
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("SmokeDrawer Uniform Bind Group"),
      layout: &render_pipeline.get_bind_group_layout(0),
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: uniform_buffer.as_entire_binding(),
        }
      ],
    });

    SmokeDrawer {
      grid,
      pixel_size,
      render_pipeline,
      uniform_buffer,
      uniform_bind_group,
    }
  }
}

impl Updater for SmokeDrawer {
  fn update(&mut self, ctx: &mut Context) {
    let grid_ref = self.grid.borrow();
    
    // 1. Update Uniform Data
    let grid_w = grid_ref.width as f32 * grid_ref.cell_size;
    let grid_h = grid_ref.height as f32 * grid_ref.cell_size;

    let uniforms = DrawerUniforms {
      cell_size: grid_ref.cell_size,
      pixel_size: self.pixel_size,
      grid_width: grid_w,
      grid_height: grid_h,
      res_x: ctx.window_w as f32,
      res_y: ctx.window_h as f32,
      _padding: [0.0; 2],
    };
    ctx.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

    // 2. Dynamic Bind Group for the current Active Smoke Buffer (Group 1)
    // This connects the simulation output directly to the renderer.
    let smoke_buffer_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Dynamic Smoke Buffer Bind Group"),
        layout: &self.render_pipeline.get_bind_group_layout(1),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_ref.get_current_smoke_buffer().as_entire_binding(),
            }
        ],
    });

    // 3. Render Pass
    let mut rpass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Smoke Drawer Render Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: ctx.view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.1, b: 0.2, a: 1.0 }),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
      multiview_mask: None,
    });

    rpass.set_pipeline(&self.render_pipeline);
    rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
    rpass.set_bind_group(1, &smoke_buffer_bg, &[]);
    
    // Full screen triangle
    rpass.draw(0..3, 0..1); 
  }
}