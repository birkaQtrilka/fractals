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
  texture: wgpu::Texture,
  uniform_buffer: wgpu::Buffer,
  
  uniform_bind_group: wgpu::BindGroup,
  texture_bind_group: wgpu::BindGroup,
}

impl SmokeDrawer {
  pub fn new(
    grid: Rc<RefCell<GridGpu>>, 
    pixel_size: f32,
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
  ) -> SmokeDrawer {
    let grid_ref = grid.borrow();
    
    // 1. Uniform Buffer
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("SmokeDrawer Uniform Buffer"),
      size: std::mem::size_of::<DrawerUniforms>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    // 2. Texture & Sampler
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("SmokeDrawer Texture"),
      size: wgpu::Extent3d {
        width: grid_ref.width as u32,
        height: grid_ref.height as u32,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::R32Float, // Replaces GL_R32F / GL_RED
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("SmokeDrawer Sampler"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Nearest,
      min_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });
    drop(grid_ref);
    
    // 3. Render Pipeline Setup
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("SmokeDrawer Shader"),
      // Combine screen_uv.vs and smoke.fs into this one wgsl file
      source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/smoke_drawer.wgsl").into()),
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("SmokeDrawer Render Pipeline"),
      layout: None,
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[], // Vertices generated internally in WGSL
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

    // 4. Bind Groups
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

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("SmokeDrawer Texture Bind Group"),
      layout: &render_pipeline.get_bind_group_layout(1),
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&texture_view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&sampler),
        }
      ],
    });

    SmokeDrawer {
      grid,
      pixel_size,
      render_pipeline,
      texture,
      uniform_buffer,
      uniform_bind_group,
      texture_bind_group,
    }
  }
}

impl Updater for SmokeDrawer {
  fn update(&mut self, ctx: &mut Context) {
    let grid_ref = self.grid.borrow();
    
    let cell_size = grid_ref.cell_size;
    let grid_w = grid_ref.width as f32 * cell_size;
    let grid_h = grid_ref.height as f32 * cell_size;

    // 1. Update Uniforms
    let uniforms = DrawerUniforms {
      cell_size,
      pixel_size: self.pixel_size,
      grid_width: grid_w,
      grid_height: grid_h,
      res_x: ctx.window_w as f32,
      res_y: ctx.window_h as f32,
      _padding: [0.0; 2],
    };
    ctx.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

    // 2. Upload CPU Smoke data to GPU Texture
    ctx.queue.write_texture(
      wgpu::TexelCopyTextureInfo {
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      bytemuck::cast_slice(&grid_ref.smoke),
      wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(grid_ref.width as u32 * 4), // 1 float (4 bytes) per pixel
        rows_per_image: Some(grid_ref.height as u32),
      },
      wgpu::Extent3d {
        width: grid_ref.width as u32,
        height: grid_ref.height as u32,
        depth_or_array_layers: 1,
      },
    );

    // 3. Render Pass directly to context view
    let mut rpass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Smoke Drawer Render Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: ctx.view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
    rpass.set_bind_group(1, &self.texture_bind_group, &[]);
    
    // Draw 3 vertices (will form a full screen triangle in WGSL without needing vertex buffers)
    rpass.draw(0..3, 0..1); 
  }
}