use std::rc::Rc;
use wgpu::util::DeviceExt;

use crate::Context;
use crate::smoke_data::{Cell, Pair};

const INVALID: f32 = -100_000_000.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GridUniforms {
  width: i32,
  height: i32,
  cell_size: f32,
  time_step: f32,
  ady: f32,
  k: f32,
  parity: i32,
  _padding: i32, // Padding to 32 bytes for strict uniform buffer alignment
}

pub struct GridGpu {
  pub width: usize,
  pub height: usize,
  pub density: f32,
  pub time_step: f32,
  pub cell_size: f32,

  pub smoke: Vec<f32>,
  pub velocities: Vec<Pair>,
  pub solid_map: Vec<bool>,

  velocities_dirty: bool,
  smoke_dirty: bool,

  device: Rc<wgpu::Device>,
  queue: Rc<wgpu::Queue>,

  solid_map_buffer: wgpu::Buffer,
  pressures_buffer: wgpu::Buffer,
  rhs_buffer: wgpu::Buffer,
  inv_total_buffer: wgpu::Buffer,

  velocities_a_buffer: wgpu::Buffer,
  velocities_b_buffer: wgpu::Buffer,
  smoke_a_buffer: wgpu::Buffer,
  smoke_b_buffer: wgpu::Buffer,

  velocities_staging: wgpu::Buffer,
  smoke_staging: wgpu::Buffer,

  uniform_buf_0: wgpu::Buffer,
  uniform_buf_1: wgpu::Buffer,

  prepare_pipeline: wgpu::ComputePipeline,
  solve_pipeline: wgpu::ComputePipeline,
  update_vel_pipeline: wgpu::ComputePipeline,
  advect_vel_pipeline: wgpu::ComputePipeline,
  advect_smoke_pipeline: wgpu::ComputePipeline,

  prepare_bg_a: wgpu::BindGroup,
  prepare_bg_b: wgpu::BindGroup,
  
  solve_bg_0: wgpu::BindGroup,
  solve_bg_1: wgpu::BindGroup,

  update_vel_bg_a: wgpu::BindGroup,
  update_vel_bg_b: wgpu::BindGroup,

  advect_vel_bg_a: wgpu::BindGroup,
  advect_vel_bg_b: wgpu::BindGroup,

  advect_smoke_bg_a: wgpu::BindGroup,
  advect_smoke_bg_b: wgpu::BindGroup,

  group_x: u32,
  group_y: u32,

  is_even_frame: bool,
}

impl GridGpu {
  pub fn new(
    width: usize,
    height: usize,
    density: f32,
    time_step: f32,
    cell_size: f32,
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
  ) -> Self {
    let size = width * height;
    let vel_size = (width + 1) * (height + 1);

    let mut velocities = vec![Pair::new(0.0, 0.0); vel_size];
    let smoke = vec![0.0_f32; size];
    let solid_map = vec![false; size];

    // Init Boundary Condition
    for i in 0..size {
      let x = i % width;
      let y = i / width;
      let vi = i + y; // x + y * (width + 1)

      if x == width - 1 { velocities[vi + 1] = Pair::new(INVALID, 0.0); }
      if y == height - 1 { velocities[vi + width + 1] = Pair::new(0.0, INVALID); }
    }
    velocities[size + height + width] = Pair::new(INVALID, INVALID);

    // 1. Uniform Setup
    let ady = density * cell_size / time_step;
    let k = time_step / (cell_size * density);
    
    let uniforms_0 = GridUniforms {
      width: width as i32,
      height: height as i32,
      cell_size,
      time_step,
      ady,
      k,
      parity: 0,
      _padding: 0,
    };
    let uniforms_1 = GridUniforms { parity: 1, ..uniforms_0 };

    let uniform_buf_0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("uniforms_0"),
      contents: bytemuck::bytes_of(&uniforms_0),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_buf_1 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("uniforms_1"),
      contents: bytemuck::bytes_of(&uniforms_1),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 2. Storage Buffers Setup
    let zeros_bytes = vec![0.0_f32; size];
    let velocities_bytes = bytemuck::cast_slice(&velocities);
    let smoke_bytes = bytemuck::cast_slice(&smoke);

    let build_storage = |label: &str, data: &[u8]| {
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: data,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
      })
    };

    let solid_map_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("solid_map"),
      size: (size * 4) as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let pressures_buffer = build_storage("pressures", bytemuck::cast_slice(&zeros_bytes));
    let rhs_buffer = build_storage("rhs", bytemuck::cast_slice(&zeros_bytes));
    let inv_total_buffer = build_storage("inv_total", bytemuck::cast_slice(&zeros_bytes));
    
    let velocities_a_buffer = build_storage("velocities_a", velocities_bytes);
    let velocities_b_buffer = build_storage("velocities_b", velocities_bytes);
    let smoke_a_buffer = build_storage("smoke_a", smoke_bytes);
    let smoke_b_buffer = build_storage("smoke_b", smoke_bytes);

    // Staging buffers for downloading GPU results back to CPU
    let velocities_staging = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("velocities_staging"),
      size: (vel_size * 8) as u64,
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let smoke_staging = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("smoke_staging"),
      size: (size * 4) as u64,
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    // 3. Shaders and Pipelines (Implicit auto-generated bind group layouts)
    let make_pipeline = |name: &str, source: &str| {
      let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(source.into()),
      });
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(name),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None
      })
    };

    let prepare_pipeline = make_pipeline("prepare", include_str!("../assets/shaders/prepare_cycle_data.wgsl"));
    let solve_pipeline = make_pipeline("solve", include_str!("../assets/shaders/red_black_pressure.wgsl"));
    let update_vel_pipeline = make_pipeline("update_vel", include_str!("../assets/shaders/update_velocities.wgsl"));
    let advect_vel_pipeline = make_pipeline("advect_vel", include_str!("../assets/shaders/advect_velocities.wgsl"));
    let advect_smoke_pipeline = make_pipeline("advect_smoke", include_str!("../assets/shaders/advect_smoke.wgsl"));

    // 4. Double Buffered Bind Groups Creation (eliminates swapping pointers)
    let build_bg = |pipeline: &wgpu::ComputePipeline, label: &str, bufs: &[&wgpu::Buffer]| {
      let entries: Vec<_> = bufs.iter().enumerate().map(|(i, b)| wgpu::BindGroupEntry {
        binding: i as u32,
        resource: b.as_entire_binding(),
      }).collect();

      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
      })
    };

    // Prepare: Uniform, SolidMap, Vel (in), Rhs (out), InvTotal (out)
    let prepare_bg_a = build_bg(&prepare_pipeline, "prepare_a", &[&uniform_buf_0, &solid_map_buffer, &velocities_a_buffer, &rhs_buffer, &inv_total_buffer]);
    let prepare_bg_b = build_bg(&prepare_pipeline, "prepare_b", &[&uniform_buf_0, &solid_map_buffer, &velocities_b_buffer, &rhs_buffer, &inv_total_buffer]);

    // Solve (Red-Black iterations ping-pong uniformly only via Parity, not layout variables)
    // Bindings: Uniform, SolidMap, Pressures (in/out), Rhs (in), InvTotal (in)
    let solve_bg_0 = build_bg(&solve_pipeline, "solve_0", &[&uniform_buf_0, &solid_map_buffer, &pressures_buffer, &rhs_buffer, &inv_total_buffer]);
    let solve_bg_1 = build_bg(&solve_pipeline, "solve_1", &[&uniform_buf_1, &solid_map_buffer, &pressures_buffer, &rhs_buffer, &inv_total_buffer]);

    // Update Velocities: Uniform, SolidMap, Pressures (in), Vel (in/out)
    let update_vel_bg_a = build_bg(&update_vel_pipeline, "update_vel_a", &[&uniform_buf_0, &solid_map_buffer, &pressures_buffer, &velocities_a_buffer]);
    let update_vel_bg_b = build_bg(&update_vel_pipeline, "update_vel_b", &[&uniform_buf_0, &solid_map_buffer, &pressures_buffer, &velocities_b_buffer]);

    // Advect Velocities: Uniform, SolidMap, Vel (in), Vel (out)
    let advect_vel_bg_a = build_bg(&advect_vel_pipeline, "advect_vel_a", &[&uniform_buf_0, &solid_map_buffer, &velocities_a_buffer, &velocities_b_buffer]);
    let advect_vel_bg_b = build_bg(&advect_vel_pipeline, "advect_vel_b", &[&uniform_buf_0, &solid_map_buffer, &velocities_b_buffer, &velocities_a_buffer]);

    // Advect Smoke: Uniform, SolidMap, Vel (in - post-advection!), Smoke (in), Smoke (out)
    let advect_smoke_bg_a = build_bg(&advect_smoke_pipeline, "advect_smoke_a", &[&uniform_buf_0, &solid_map_buffer, &velocities_b_buffer, &smoke_a_buffer, &smoke_b_buffer]);
    let advect_smoke_bg_b = build_bg(&advect_smoke_pipeline, "advect_smoke_b", &[&uniform_buf_0, &solid_map_buffer, &velocities_a_buffer, &smoke_b_buffer, &smoke_a_buffer]);

    let mut grid = GridGpu {
      width, height, density, time_step, cell_size,
      smoke, velocities, solid_map,
      velocities_dirty: true, smoke_dirty: true,
      device, queue,
      
      solid_map_buffer, pressures_buffer, rhs_buffer, inv_total_buffer,
      velocities_a_buffer, velocities_b_buffer, smoke_a_buffer, smoke_b_buffer,
      velocities_staging, smoke_staging,
      uniform_buf_0, uniform_buf_1,
      
      prepare_pipeline, solve_pipeline, update_vel_pipeline, advect_vel_pipeline, advect_smoke_pipeline,
      prepare_bg_a, prepare_bg_b,
      solve_bg_0, solve_bg_1,
      update_vel_bg_a, update_vel_bg_b,
      advect_vel_bg_a, advect_vel_bg_b,
      advect_smoke_bg_a, advect_smoke_bg_b,
      
      group_x: (width as u32 + 7) / 8,
      group_y: (height as u32 + 7) / 8,

      is_even_frame: true,
    };

    grid.init_solid_map();
    grid
  }

  fn init_solid_map(&mut self) {
    for i in 0..self.width {
      self.solid_map[i] = true;
      self.solid_map[i + self.width * (self.height - 1)] = true;
    }
    for i in (0..(self.width * self.height)).step_by(self.width) {
      self.solid_map[i] = true;
      self.solid_map[i + self.width - 1] = true;
    }
    self.upload_solid_map();
  }

  // Call this explicitly if you manually edit the CPU-side `solid_map`
  pub fn upload_solid_map(&self) {
    let as_u32: Vec<u32> = self.solid_map.iter().map(|&b| b as u32).collect();
    self.queue.write_buffer(&self.solid_map_buffer, 0, bytemuck::cast_slice(&as_u32));
  }

  pub fn set_velocities(&mut self, pressure_index: usize, t: Option<f32>, l: Option<f32>, b: Option<f32>, r: Option<f32>) {
    let y = pressure_index / self.width;
    let vi = pressure_index + y;

    let vel = self.velocities[vi];
    self.velocities[vi] = Pair::new(t.unwrap_or(vel.top), l.unwrap_or(vel.left));

    let vel = self.velocities[vi + 1];
    self.velocities[vi + 1] = Pair::new(vel.top, r.unwrap_or(vel.left));

    let vel = self.velocities[vi + self.width + 1];
    self.velocities[vi + self.width + 1] = Pair::new(b.unwrap_or(vel.top), vel.left);
    
    self.velocities_dirty = true;
  }

  pub fn get_velocities(&self, pressure_index: usize) -> Cell {
    let y = pressure_index / self.width;
    let vi = pressure_index + y;
    let t = self.velocities[vi].top;
    let l = self.velocities[vi].left;
    let b = self.velocities[vi + self.width + 1].top;
    let r = self.velocities[vi + 1].left;
    Cell::new(t, l, b, r)
  }

  pub fn set_smoke(&mut self, i: usize, val: f32) {
    self.smoke[i] = val;
    self.smoke_dirty = true;
  }

  pub fn step(&mut self, ctx: &Context, pressure_iterations: u32) {
    // 1. Send CPU overrides down to GPU
    if self.velocities_dirty {
      let current_vel = if self.is_even_frame { &self.velocities_a_buffer } else { &self.velocities_b_buffer };
      ctx.queue.write_buffer(current_vel, 0, bytemuck::cast_slice(&self.velocities));
      self.velocities_dirty = false;
    }
    
    if self.smoke_dirty {
      let current_smoke = if self.is_even_frame { &self.smoke_a_buffer } else { &self.smoke_b_buffer };
      ctx.queue.write_buffer(current_smoke, 0, bytemuck::cast_slice(&self.smoke));
      self.smoke_dirty = false;
    }

    // Creating an isolated ComputeEncoder so we can safely submit midway
    let mut compute_encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Grid Compute Encoder") });

    {
      let mut cpass = compute_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Fluid Compute Pass"),
        timestamp_writes: None,
      });

      // 1. Prepare Cycle Data
      cpass.set_pipeline(&self.prepare_pipeline);
      cpass.set_bind_group(0, if self.is_even_frame { &self.prepare_bg_a } else { &self.prepare_bg_b }, &[]);
      cpass.dispatch_workgroups(self.group_x, self.group_y, 1);

      // 2. Solve Pressure (Red-Black iterations)
      cpass.set_pipeline(&self.solve_pipeline);
      for _ in 0..pressure_iterations {
        cpass.set_bind_group(0, &self.solve_bg_0, &[]);
        cpass.dispatch_workgroups(self.group_x, self.group_y, 1);

        cpass.set_bind_group(0, &self.solve_bg_1, &[]);
        cpass.dispatch_workgroups(self.group_x, self.group_y, 1);
      }

      // 3. Update Velocities
      cpass.set_pipeline(&self.update_vel_pipeline);
      cpass.set_bind_group(0, if self.is_even_frame { &self.update_vel_bg_a } else { &self.update_vel_bg_b }, &[]);
      cpass.dispatch_workgroups(self.group_x, self.group_y, 1);

      // 4. Advect Velocities
      cpass.set_pipeline(&self.advect_vel_pipeline);
      cpass.set_bind_group(0, if self.is_even_frame { &self.advect_vel_bg_a } else { &self.advect_vel_bg_b }, &[]);
      cpass.dispatch_workgroups(self.group_x, self.group_y, 1);

      // 5. Advect Smoke
      cpass.set_pipeline(&self.advect_smoke_pipeline);
      cpass.set_bind_group(0, if self.is_even_frame { &self.advect_smoke_bg_a } else { &self.advect_smoke_bg_b }, &[]);
      cpass.dispatch_workgroups(self.group_x, self.group_y, 1);
    }

    // Determine the result buffers based on ping-pong states (advect writes to opposite buffers)
    let result_vel = if self.is_even_frame { &self.velocities_b_buffer } else { &self.velocities_a_buffer };
    let result_smoke = if self.is_even_frame { &self.smoke_b_buffer } else { &self.smoke_a_buffer };
    let vel_size = (self.width + 1) * (self.height + 1);
    let size = self.width * self.height;

    // Queue a copy up to the staging buffers 
    compute_encoder.copy_buffer_to_buffer(result_vel, 0, &self.velocities_staging, 0, (vel_size * 8) as wgpu::BufferAddress);
    compute_encoder.copy_buffer_to_buffer(result_smoke, 0, &self.smoke_staging, 0, (size * 4) as wgpu::BufferAddress);

    // Since step must function synchronously on the CPU-side exactly like openGL, 
    // submit what we have recorded right now!
    ctx.queue.submit(std::iter::once(compute_encoder.finish()));

    // Wait and sync GPU state
    let vel_slice = self.velocities_staging.slice(..);
    let smoke_slice = self.smoke_staging.slice(..);

    let (tx1, rx1) = std::sync::mpsc::channel();
    vel_slice.map_async(wgpu::MapMode::Read, move |res| tx1.send(res).unwrap());
    
    let (tx2, rx2) = std::sync::mpsc::channel();
    smoke_slice.map_async(wgpu::MapMode::Read, move |res| tx2.send(res).unwrap());
      
    ctx.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
      
    rx1.recv().unwrap().unwrap();
    rx2.recv().unwrap().unwrap();
      
    let vel_data = vel_slice.get_mapped_range().unwrap();
    self.velocities.copy_from_slice(bytemuck::cast_slice(&vel_data));
    drop(vel_data);
    self.velocities_staging.unmap();
      
    let smoke_data = smoke_slice.get_mapped_range().unwrap();
    self.smoke.copy_from_slice(bytemuck::cast_slice(&smoke_data));
    drop(smoke_data);
    self.smoke_staging.unmap();
      
    // Advance frame
    self.is_even_frame = !self.is_even_frame;
  }
}