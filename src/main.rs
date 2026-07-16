#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod learn_opengl;

use std::{collections::HashSet, ffi::CString};

use beryllium::{
  events::{Event, SDL_Keycode, SDLK_DOWN, SDLK_ESCAPE, SDLK_LEFT, SDLK_RIGHT, SDLK_UP, SDLK_x, SDLK_z}, video::GlSwapInterval, *,
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
// type TriIndexes = [u32; 3];


// const VERTICES: [Vertex; 4] =
//   [[0.5, 0.5, 0.0], [0.5, -0.5, 0.0], [-0.5, -0.5, 0.0], [-0.5, 0.5, 0.0]];
const VERTICES: [Vertex; 3] =
  [[-1.0, -1.0, 0.0], [-1.0, 3.0, 0.0], [3.0, -1.0, 0.0]];

// const INDICES: [TriIndexes; 2] = [[0, 1, 3], [1, 2, 3]];

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec3 pos;
  out vec2 uv;

  void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0);
    uv = (pos * 0.5 + 0.5).xy;
  }
"#;
const FRAG_SHADER: &str = r#"#version 330 core
  in vec2 uv;
  out vec4 final_color;
  uniform float zoom;
  uniform vec2 offset;

  float map(float value, float oldMin, float oldMax,
            float newMin, float newMax)
  {
    return newMin +
           (value - oldMin) * (newMax - newMin) /
           (oldMax - oldMin);
  }

  // vec2 where x is number and y is 10 to the power

  void main() {
    int max_iterations = 100;

    int n = 0;
    float a = map(uv.x, 0, 1, -2 * zoom, 2 * zoom);
    float b = map(uv.y, 0, 1, -2 * zoom, 2 * zoom); 

    a += offset.x;
    b += offset.y;

    float orig_a = a;
    float orig_b = b;
    while (n < max_iterations) {
      float next_a = a * a - b * b;
      float next_b = 2 * a * b;
      a = next_a + orig_a;
      b = next_b + orig_b;

      if(abs(a+b) > 16) { break; }

      n++;
    }
    float bright = map(n, 0, max_iterations, 0, 1);
    if(n == max_iterations) bright = 0;

    final_color = vec4(bright, bright, 0.2, 1.0);
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
  let programm = ShaderProgram::from_vert_frag(&VERT_SHADER, &FRAG_SHADER).expect("couldn't create programm");
  programm.use_program();

  learn::polygon_mode(learn::PolygonMode::Fill);

  let mut zoom: f32 = 1.0;
  let zoom_speed = 0.95;
  let mut pos = (0.0_f32, 0.0_f32);
  let move_speed = 0.02;

  let zoom_name = CString::new("zoom").unwrap();
  let zoom_location = unsafe {
    glGetUniformLocation(programm.0, zoom_name.as_ptr())
  };

  let pos_name = CString::new("offset").unwrap();
  let pos_location = unsafe {
    glGetUniformLocation(programm.0, pos_name.as_ptr())
  };
  let mut keys_pressed: HashSet<SDL_Keycode> = HashSet::new();
  
  'main_loop: loop {
    // Process all events
    while let Some((event, _remaining)) = sdl.poll_events() {
      match event {
        Event::Quit => break 'main_loop,
        Event::Key { pressed, keycode, .. } => {
          if pressed {
            keys_pressed.insert(keycode);
            // Check for escape key immediately
            if keycode == SDLK_ESCAPE {
              break 'main_loop;
            }
          } else {
            keys_pressed.remove(&keycode);
          }
        }
        _ => (),
      }
    }
    
    // Process all currently pressed keys
    let relative_speed = move_speed * zoom;
    
    if keys_pressed.contains(&SDLK_x) {
      zoom *= zoom_speed;
    }
    if keys_pressed.contains(&SDLK_z) {
      zoom /= zoom_speed;
      if zoom > 1.0 {zoom = 1.0}
    }
    if keys_pressed.contains(&SDLK_DOWN) {
      pos.1 -= relative_speed;
    }
    if keys_pressed.contains(&SDLK_UP) {
      pos.1 += relative_speed;
    }
    if keys_pressed.contains(&SDLK_RIGHT) {
      pos.0 += relative_speed;
    }
    if keys_pressed.contains(&SDLK_LEFT) {
      pos.0 -= relative_speed;
    }
    // now the events are clear
    // here's where we could change the world state and draw.
    unsafe {
      glClear(GL_COLOR_BUFFER_BIT);
      // glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 as *const _);
      glDrawArrays(GL_TRIANGLES, 0, 3);
      glUniform1f(zoom_location, zoom);
      glUniform2f(pos_location, pos.0, pos.1);
      win.swap_window();
    }
  }
}
