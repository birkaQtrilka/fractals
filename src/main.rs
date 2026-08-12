#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod learn_opengl;
mod updater;
mod input_handling;
mod smoke_grid;
mod bilinear;
mod smoke_data;
mod smoke_runner;
mod smoke_drawer;
mod smoke_interactor;
mod compute_ext;
mod grid_gpu;

use crate::compute_ext::ComputeExt;
use crate::input_handling::*;
use crate::updater::{Updater};
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
  pub time: f32,
  pub delta_time: f32,
  pub fps: f32,
}

fn main() {
  let sdl = Sdl::init(init::InitFlags::EVERYTHING);
  sdl.set_gl_context_major_version(4).unwrap();
  sdl.set_gl_context_minor_version(3).unwrap();
  sdl.set_gl_profile(video::GlProfile::Core).unwrap();
  
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
  let compute_extern = unsafe {
    Rc::new(ComputeExt::load(|name| win.get_proc_address(name.as_ptr().cast())))
  };
    
  let version = unsafe { std::ffi::CStr::from_ptr(glGetString(GL_VERSION) as *const i8) };
  println!("OpenGL version: {:?}", version);
  
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
    Box::new(Smoke::start(
      ctx.window_w as f32, 
      ctx.window_h as f32, 
      100,
      90,
      1.0, 
      0.04, 
      Rc::clone(&ctx.input_handler),
      Rc::clone(&compute_extern),
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
    let (switch_world, print_fps) = {
      let inp = ctx.input_handler.borrow();
      (
        inp.is_key_down(SDLK_1) && worlds.len() > 1,
        inp.is_key_down(SDLK_m)
      )
    };

    if switch_world {
      worlds[world_index].on_disable();
      
      if world_index == worlds.len()-1 {world_index = 0;}
      else {world_index += 1;}
      worlds[world_index].on_enable();
    } else if print_fps {
      println!("{}", frames_this_second as f32 / fps_elapsed)
    }

    unsafe {
      glClear(GL_COLOR_BUFFER_BIT);
      glDrawArrays(GL_TRIANGLES, 0, 3);
      win.swap_window();
    }
  }
}