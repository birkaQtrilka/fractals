#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod learn_opengl;
mod program;
mod input_handling;
mod mover;
mod julia_set;
mod flow_field;
mod smoke_grid;
mod bilinear;
mod smoke_data;
mod smoke_runner;
mod smoke_drawer;
mod smoke_interactor;


// use crate::flow_field::Field;
use crate::input_handling::*;
use crate::julia_set::JuliaSet;
use crate::mover::MoveData;
use crate::program::{Mandelbrot, Updater};
use crate::smoke_runner::Smoke;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant; // <-- Added for time tracking

use beryllium::events::{SDLK_1, SDLK_m};
use beryllium::{
  video::GlSwapInterval, *,
};
use ogl33::*;
use learn_opengl::{
  self as learn,
  VertexArray,
  Buffer,
  BufferType,
};

const WINDOW_TITLE: &str = "Fractals";
type Vertex = [f32; 3];

const VERTICES: [Vertex; 3] =
  [[-1.0, -1.0, 0.0], [-1.0, 3.0, 0.0], [3.0, -1.0, 0.0]];

pub struct Context {
  pub input_handler: Rc<RefCell<InputHandler>>,
  pub window_w: i32,
  pub window_h: i32,
  // Added time tracking fields
  pub time: f32,
  pub delta_time: f32,
  pub fps: f32,
}

fn map_range(val: f64, in_min: f64, in_max: f64, out_min: u32, out_max: u32) -> u32 {
    let safe_val = val.max(0.000000000001);
    let safe_in_min = in_min.max(0.000000000001);
    let safe_in_max = in_max.max(0.000000000001);

    let log_val = safe_val.log10();
    let log_in_min = safe_in_min.log10();
    let log_in_max = safe_in_max.log10();
    
    let out_min_f = out_min as f64;
    let out_max_f = out_max as f64;
    
    let mapped = (log_val - log_in_min) / (log_in_max - log_in_min) * (out_max_f - out_min_f) + out_min_f;
    
    // let clamp_min = out_min_f.min(out_max_f);
    // let clamp_max = out_min_f.max(out_max_f);
    
    // mapped.clamp(clamp_min, clamp_max).round() as u32
    mapped.round() as u32
}

fn main() {
  let sdl = Sdl::init(init::InitFlags::EVERYTHING);
  sdl.set_gl_context_major_version(3).unwrap();
  sdl.set_gl_context_minor_version(3).unwrap();
  sdl.set_gl_profile(video::GlProfile::Core).unwrap();
  
  #[cfg(target_os = "macos")]
  {
    sdl
      .set_gl_context_flags(video::GlContextFlags::FORWARD_COMPATIBLE)
      .unwrap();
  }

  let win_args = video::CreateWinArgs {
    title: WINDOW_TITLE,
    width: 900,
    height: 750,
    allow_high_dpi: true,
    borderless: false,
    resizable: false,
  };
  
  let win = sdl
    .create_gl_window(win_args)
    .expect("couldn't make a window and context");
  unsafe {
    load_gl_with(|f_name| win.get_proc_address(f_name as *const u8));
    let _ = win.set_swap_interval(GlSwapInterval::Vsync);
  }

  learn::clear_color(0.2, 0.3, 0.3, 1.0);

  let vao = VertexArray::new().expect("Couldn't make vao");
  vao.bind();

  let vbo = Buffer::new().expect("Couldn't make vbo");
  vbo.bind(BufferType::Array);
  learn::buffer_data(
    BufferType::Array, 
    bytemuck::cast_slice(&VERTICES),
    GL_STATIC_DRAW
  );

  unsafe {
    glVertexAttribPointer(
      0,
      3,
      GL_FLOAT,
      GL_FALSE,
      size_of::<Vertex>().try_into().unwrap(),
      0 as *const _,
    );
    glEnableVertexAttribArray(0);
  }

  learn::polygon_mode(learn::PolygonMode::Fill);

  let resolotion = win.get_window_size();  
  let mut ctx = Context {
    input_handler: Rc::new(RefCell::new(InputHandler::new())),// make this rc?
    window_w: resolotion.0,
    window_h: resolotion.1,
    time: 0.0,
    delta_time: 0.0,
    fps: 0.0,
  };
  
  let mut worlds: Vec<Box<dyn Updater>> = vec![
    Box::new(Mandelbrot::new(
      MoveData::new(0.95,0.02), 
      "zoom", 
      "offset", 
      "assets/shaders/mandelbrot/mandelbrot.fs",
      250
    )),
    Box::new(JuliaSet::new(
      MoveData::new(0.95,0.02), 
      "zoom", 
      "offset", 
      "assets/shaders/mandelbrot/julia-set.fs",
      "julia_const",
      0.001,
      "save-file.txt",
      250
    )),
    Box::new(Smoke::start(
      ctx.window_w as f32, 
      ctx.window_h as f32, 
      120,
      100,
      1.0, 
      0.04, 
      Rc::clone(&ctx.input_handler)
    )),
  ];
  let mut world_index = 2;

  let start_time = Instant::now();
  let mut last_frame_time = start_time;
  
  let mut fps_timer = start_time;
  let mut frames_this_second = 0;
  worlds[world_index].on_enable();

  'main_loop: loop {
    let current_time = Instant::now();
    ctx.delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
    ctx.time = current_time.duration_since(start_time).as_secs_f32();
    last_frame_time = current_time;

    frames_this_second += 1;
    let fps_elapsed = current_time.duration_since(fps_timer).as_secs_f32();
    
    // Update FPS value every 0.5 seconds to make it readable (less jittery)
    if fps_elapsed >= 0.5 {
        ctx.fps = frames_this_second as f32 / fps_elapsed;
        frames_this_second = 0;
        fps_timer = current_time;
    }
    {
      let mut input = ctx.input_handler.borrow_mut();
      input.main_loop();
      while let Some((event, _remaining)) = sdl.poll_events() {
        if input.process_events(event) {
          break 'main_loop;
        }
      }
    }

    worlds[world_index].update(&ctx);
    {
      let inp = ctx.input_handler.borrow();
      if inp.is_key_down(SDLK_1) {
        worlds[world_index].on_disable();
        
        if world_index == worlds.len()-1 {world_index = 0;}
        else {world_index += 1;}
        
        worlds[world_index].on_enable();
      } else if inp.is_key_down(SDLK_m) {
        println!("{}", frames_this_second as f32 / fps_elapsed)
      }   
    }

    unsafe {
      glClear(GL_COLOR_BUFFER_BIT);
      glDrawArrays(GL_TRIANGLES, 0, 3);
      win.swap_window();
    }
  }
}