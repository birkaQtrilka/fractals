#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod learn_opengl;

use beryllium::{
  events::Event,
  init::InitFlags,
  video::{CreateWinArgs, GlContextFlags, GlProfile, GlSwapInterval},
  *,
};
use ogl33::*;
use learn_opengl as learn;
const WINDOW_TITLE: &str = "Fractals";
type Vertex = [f32; 3];
const VERTICES: [Vertex; 3] =
  [[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]];

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
  sdl.set_gl_context_major_version(3).unwrap();
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
    glClearColor(0.2, 0.3, 0.3, 1.0);
    let _ = win.set_swap_interval(GlSwapInterval::Vsync);

    let mut vao = 0;
    glGenVertexArrays(1, &mut vao);
    assert_ne!(vao, 0);

    let mut vbo = 0;
    glGenBuffers(1, &mut vbo);
    assert_ne!(vbo, 0);
    glBindBuffer(GL_ARRAY_BUFFER, vbo);

    glBufferData(
      GL_ARRAY_BUFFER,
      size_of_val(&VERTICES) as isize,
      VERTICES.as_ptr().cast(),
      GL_STATIC_DRAW,
    );

    glVertexAttribPointer(
      0,
      3,
      GL_FLOAT,
      GL_FALSE,
      size_of::<Vertex>().try_into().unwrap(),
      0 as *const _,
    );
    glEnableVertexAttribArray(0);

    let vertex_shader = glCreateShader(GL_VERTEX_SHADER);
    assert_ne!(vertex_shader, 0);

    glShaderSource(
      vertex_shader,
      1,
      &(VERT_SHADER.as_bytes().as_ptr().cast()),
      &(VERT_SHADER.len().try_into().unwrap()),
    );

    glCompileShader(vertex_shader);
    let mut success = 0;
    glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &mut success);

    if success == 0 {
      let mut v: Vec<u8> = Vec::with_capacity(1024);
      let mut log_len = 0_i32;
      glGetShaderInfoLog(
        vertex_shader,
        1024,
        &mut log_len,
        v.as_mut_ptr().cast(),
      );
      v.set_len(log_len.try_into().unwrap());
      panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
    }

    let fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
    assert_ne!(fragment_shader, 0);

    glShaderSource(
      fragment_shader,
      1,
      &(FRAG_SHADER.as_bytes().as_ptr().cast()),
      &(FRAG_SHADER.len().try_into().unwrap()),
    );

    glCompileShader(fragment_shader);
    let mut success = 0;
    glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &mut success);

    if success == 0 {
      let mut v: Vec<u8> = Vec::with_capacity(1024);
      let mut log_len = 0_i32;
      glGetShaderInfoLog(
        fragment_shader,
        1024,
        &mut log_len,
        v.as_mut_ptr().cast(),
      );
      v.set_len(log_len.try_into().unwrap());
      panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
    }

    let shader_program = glCreateProgram();
    glAttachShader(shader_program, vertex_shader);
    glAttachShader(shader_program, fragment_shader);
    glLinkProgram(shader_program);

    let mut success = 0;
    glGetProgramiv(shader_program, GL_LINK_STATUS, &mut success);
    if success == 0 {
      let mut v: Vec<u8> = Vec::with_capacity(1024);
      let mut log_len = 0_i32;
      glGetProgramInfoLog(
        shader_program,
        1024,
        &mut log_len,
        v.as_mut_ptr().cast(),
      );
      v.set_len(log_len.try_into().unwrap());
      panic!("Program Link Error: {}", String::from_utf8_lossy(&v));
    }

    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);
    glUseProgram(shader_program);
  }

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
      glDrawArrays(GL_TRIANGLES, 0, 3);
      win.swap_window();
    }
  }
}

