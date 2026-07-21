#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod learn_opengl;
mod program;
mod input_handling;
mod mover;
mod julia_set;

use crate::input_handling::*;
use crate::julia_set::JuliaSet;
use crate::mover::MoveData;
use crate::program::{Mandelbrot, Updater};


use beryllium::events::SDLK_1;
use beryllium::{
  events::{Event, SDLK_ESCAPE}, video::GlSwapInterval, *,
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
        width: 1100,
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
    ))
  ];
  let mut world_index = 0;

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
    worlds[world_index].update(&ctx);
    if ctx.input_handler.is_key_down(SDLK_1) {
      if world_index == 1 {world_index = 0;}
      else {world_index = 1;}
    }    

    unsafe {
      glClear(GL_COLOR_BUFFER_BIT);
      // glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 as *const _);
      glDrawArrays(GL_TRIANGLES, 0, 3);
      win.swap_window();
    }
  }
}
