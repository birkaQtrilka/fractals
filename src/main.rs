#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod learn_opengl;

use beryllium::{
  events::Event,
  init::InitFlags,
  video::{CreateWinArgs, GlContextFlags, GlProfile, GlSwapInterval},
  *,
};
use ogl33::*;
use learn_opengl::{
  self as learn,
  VertexArray,
  Buffer,
  BufferType,
  ShaderProgram
};

const WINDOW_TITLE: &str = "Fractals";
type Vertex = [f32; 3];
type TriIndexes = [u32; 3];


const VERTICES: [Vertex; 4] =
  [[0.5, 0.5, 0.0], [0.5, -0.5, 0.0], [-0.5, -0.5, 0.0], [-0.5, 0.5, 0.0]];

const INDICES: [TriIndexes; 2] = [[0, 1, 3], [1, 2, 3]];

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;
  void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
  }
"#;
const FRAG_SHADER: &str = r#"#version 330 core
  out vec4 final_color;

  void main() {
    final_color = vec4(1.0, 0.5, 0.2, 1.0);
  }
"#;


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
        width: 800,
        height: 600,
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

  let mut vbo = Buffer::new().expect("Couldn't make vbo");
  vbo.bind(BufferType::Array);
  learn::buffer_data(
    BufferType::Array, 
    bytemuck::cast_slice(&VERTICES),
    GL_STATIC_DRAW
  );

  let mut ebo = Buffer::new().expect("Couldn't make ebo");
  ebo.bind(BufferType::ElementArray);
  learn::buffer_data(
    BufferType::ElementArray, 
    bytemuck::cast_slice(&INDICES),
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
  let programm = ShaderProgram::from_vert_frag(&VERT_SHADER, &FRAG_SHADER).expect("couldn't create programm");
  programm.use_program();

  learn::polygon_mode(learn::PolygonMode::Fill);

  'main_loop: loop {
    while let Some(event) = sdl.poll_events() {
      match event {
          (events::Event::Quit, _) => break 'main_loop,
          _ => (),
      }
    }
    // now the events are clear
    // here's where we could change the world state and draw.
    unsafe {
      glClear(GL_COLOR_BUFFER_BIT);
      glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 as *const _);
      win.swap_window();
    }
  }
}

