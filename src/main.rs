#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod updater;
mod input_handling;
mod bilinear;
mod smoke_data;
mod smoke_runner;
mod smoke_drawer;
mod smoke_interactor;
mod grid_gpu;

use crate::input_handling::*;
use crate::updater::Updater;
use crate::smoke_runner::Smoke;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

// Use web_time for WebAssembly compatibility
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use winit::{
  application::ApplicationHandler,
  event::*,
  event_loop::{ActiveEventLoop, EventLoop},
  keyboard::KeyCode,
  window::{Window, WindowId},
};

const WINDOW_TITLE: &str = "Fractals (wgpu)";

// Context now carries the wgpu primitives needed per-frame
pub struct Context<'a> {
  pub input_handler: Rc<RefCell<InputHandler>>,
  pub window_w: u32,
  pub window_h: u32,
  pub time: f32,
  pub delta_time: f32,
  pub fps: f32,
  
  // wgpu specific frame data
  pub device: Rc<wgpu::Device>,
  pub queue: Rc<wgpu::Queue>,
  pub encoder: &'a mut wgpu::CommandEncoder,
  pub view: &'a wgpu::TextureView,
  pub surface_format: wgpu::TextureFormat,
}

struct App {
  window: Option<Arc<Window>>, // Arc required by wgpu v30+
  surface: Option<wgpu::Surface<'static>>,
  device: Option<Rc<wgpu::Device>>,
  queue: Option<Rc<wgpu::Queue>>,
  config: Option<wgpu::SurfaceConfiguration>,

  input_handler: Rc<RefCell<InputHandler>>,
  worlds: Vec<Box<dyn Updater>>,
  world_index: usize,

  start_time: Option<Instant>,
  last_frame_time: Option<Instant>,
  fps_timer: Option<Instant>,
  frames_this_second: u32,
  fps: f32,
  time: f32,
  delta_time: f32,
}

impl App {
  fn new() -> Self {
    Self {
      window: None,
      surface: None,
      device: None,
      queue: None,
      config: None,
      input_handler: Rc::new(RefCell::new(InputHandler::new())),
      worlds: Vec::new(),
      world_index: 0,
      start_time: None,
      last_frame_time: None,
      fps_timer: None,
      frames_this_second: 0,
      fps: 0.0,
      time: 0.0,
      delta_time: 0.0,
    }
  }
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      let win_attrs = Window::default_attributes()
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(900.0, 750.0))
        .with_resizable(false);

      let window = Arc::new(event_loop.create_window(win_attrs).unwrap());
      self.window = Some(window.clone());

      // --- WGPU INITIALIZATION ---
      let instance = wgpu::Instance::default();
      let surface = instance.create_surface(window.clone()).unwrap();

      let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        apply_limit_buckets: false,
      })).expect("Failed to find an appropriate adapter");

      let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          required_features: wgpu::Features::empty(),
          required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
          ..Default::default()
        },
      )).expect("Failed to create device");

      let size = window.inner_size();
      let config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
      surface.configure(&device, &config);

      let device = Rc::new(device);
      let queue = Rc::new(queue);

      // --- APP SETUP ---
      self.worlds.push(Box::new(Smoke::start(
        size.width as f32,
        size.height as f32,
        100,
        90,
        1.0,
        0.04,
        Rc::clone(&self.input_handler),
        Rc::clone(&device),
        Rc::clone(&queue),
      )));
      self.worlds[self.world_index].on_enable();

      self.surface = Some(surface);
      self.device = Some(device);
      self.queue = Some(queue);
      self.config = Some(config);

      let now = Instant::now();
      self.start_time = Some(now);
      self.last_frame_time = Some(now);
      self.fps_timer = Some(now);
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
    let window_id = self.window.as_ref().map(|w| w.id());
    if Some(id) != window_id {
      return;
    }

    let mut input = self.input_handler.borrow_mut();
    if input.process_events(&event) {
      event_loop.exit();
    }
    drop(input); // Drop early so it can be borrowed again in redraw

    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::Resized(physical_size) => {
        if let (Some(surface), Some(device), Some(config)) = (
          &self.surface, &self.device, &mut self.config
        ) {
          config.width = physical_size.width;
          config.height = physical_size.height;
          surface.configure(device, config);
        }
      }
      WindowEvent::RedrawRequested => {
        if let (Some(surface), Some(device), Some(queue), Some(config)) = (
          &self.surface, &self.device, &self.queue, &self.config
        ) {
          // Timing
          let current_time = Instant::now();
          if let (Some(last_frame), Some(start), Some(fps_timer)) = (
            self.last_frame_time, self.start_time, self.fps_timer
          ) {
            self.delta_time = current_time.duration_since(last_frame).as_secs_f32();
            self.time = current_time.duration_since(start).as_secs_f32();
            self.last_frame_time = Some(current_time);

            self.frames_this_second += 1;
            let fps_elapsed = current_time.duration_since(fps_timer).as_secs_f32();
            
            if fps_elapsed >= 0.5 {
              self.fps = self.frames_this_second as f32 / fps_elapsed;
              self.frames_this_second = 0;
              self.fps_timer = Some(current_time);
            }
          }

          let (switch_world, print_fps) = {
            let inp = self.input_handler.borrow();
            (
              inp.is_key_down(KeyCode::Digit1) && self.worlds.len() > 1,
              inp.is_key_down(KeyCode::KeyM)
            )
          };

          if switch_world {
            self.worlds[self.world_index].on_disable();
            self.world_index = if self.world_index == self.worlds.len() - 1 { 0 } else { self.world_index + 1 };
            self.worlds[self.world_index].on_enable();
          } else if print_fps {
            println!("{fps}", fps = self.fps);
          }

          // --- WGPU RENDERING ---
          let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Outdated => {
              surface.configure(device, config);
              return;
            }
            _ => return, // Lost, Timeout, or Validation error
          };

          let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
          let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Main Encoder") });

          // Scoped context so encoder borrow drops before submit
          {
            let mut ctx = Context {
              input_handler: Rc::clone(&self.input_handler),
              window_w: config.width,
              window_h: config.height,
              time: self.time,
              delta_time: self.delta_time,
              fps: self.fps,
              device: Rc::clone(device),
              queue: Rc::clone(queue),
              encoder: &mut encoder,
              view: &view,
              surface_format: config.format,
            };

            self.worlds[self.world_index].update(&mut ctx);
          }

          // Submit compute/render passes
          queue.submit(std::iter::once(encoder.finish()));
          
          // Present the texture (moved to `queue` in wgpu v30)
          queue.present(output);
        }
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(window) = &self.window {
      window.request_redraw();
    }
  }
}

pub fn run() {
  let event_loop = EventLoop::new().expect("Failed to create event loop");
  let mut app = App::new();
  let _ = event_loop.run_app(&mut app);
}

fn main() {
  run();
}