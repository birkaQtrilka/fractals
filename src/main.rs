#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod learn_opengl;
mod program;
mod input_handling;

use std::collections::HashMap;
use std::{collections::HashSet};
use crate::input_handling::*;
use crate::program::{Mandelbrot, Programm};

use std::fs::{File, OpenOptions};
use std::io::{self, Write};

use beryllium::events::SDLK_p;
use beryllium::{
  events::{Event, SDL_Keycode, SDLK_ESCAPE}, video::GlSwapInterval, *,
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
// type TriIndexes = [u32; 3];


// const VERTICES: [Vertex; 4] =
//   [[0.5, 0.5, 0.0], [0.5, -0.5, 0.0], [-0.5, -0.5, 0.0], [-0.5, 0.5, 0.0]];
const VERTICES: [Vertex; 3] =
  [[-1.0, -1.0, 0.0], [-1.0, 3.0, 0.0], [3.0, -1.0, 0.0]];

// const INDICES: [TriIndexes; 2] = [[0, 1, 3], [1, 2, 3]];
struct Context {
  input_handler: InputHandler,
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
        width: 1200,
        height: 1000,
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

  // let mut ebo = Buffer::new().expect("Couldn't make ebo");
  // ebo.bind(BufferType::ElementArray);
  // learn::buffer_data(
  //   BufferType::ElementArray, 
  //   bytemuck::cast_slice(&INDICES),
  //   GL_STATIC_DRAW
  // );

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

  
  let mut ctx = Context {
    input_handler: InputHandler::new(),
  };
  
  // let mut world= Mandelbrot::new(0.95, 0.02, "zoom", "offset", 
  // "assets/shaders/mandelbrot/mandelbrot.fs");
  let mut world= Mandelbrot::new(
    0.95, 
    0.02, 
    "zoom", 
    "offset", 
    "julia_const",
    0.001,
    "assets/shaders/mandelbrot/julia-set.fs",
  );
  // let mut world= Mandelbrot::new(0.95, 0.02, "zoom", "offset");
  // let mut file = File::create("save-file.txt");
  let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("save-file.txt")
    .expect("file couldn't be created or opened");

  'main_loop: loop {
    ctx.input_handler.update_key_state();

    while let Some((event, _remaining)) = sdl.poll_events() {
      match event {
        Event::Quit => break 'main_loop,
        Event::Key { pressed, keycode, .. } => {
          if pressed {
            ctx.input_handler.activate_key(keycode);
            // Check for escape key immediately
            if keycode == SDLK_ESCAPE {
              break 'main_loop;
            }
          } else {
            ctx.input_handler.deactivate_key(keycode);
          }
        }
        _ => (),
      }
    }
    
    world.update(&ctx);

    if ctx.input_handler.get_key(SDLK_p).state == PressState::Down {
      world.save(&mut file);
    }

    unsafe {
      glClear(GL_COLOR_BUFFER_BIT);
      // glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 as *const _);
      glDrawArrays(GL_TRIANGLES, 0, 3);
      win.swap_window();
    }
  }
}
