use ogl33::*;
use std::os::raw::c_void;
use std::ffi::CString;

/// Sets the color to clear to when clearing the screen.
pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
  unsafe { glClearColor(r, g, b, a) }
}
/// Basic wrapper for a VAO or [Vertex Array
/// Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object).
pub struct VertexArray(pub GLuint);
impl VertexArray {
  /// Creates a new vertex array object
  pub fn new() -> Option<Self> {
    let mut vao = 0;
    unsafe { glGenVertexArrays(1, &mut vao) };
    if vao != 0 {
      Some(Self(vao))
    } else {
      None
    }
  }

  /// Bind this vertex array as the current vertex array object
  pub fn bind(&self) {
    unsafe { glBindVertexArray(self.0) }
  }

  /// Clear the current vertex array object binding.
  pub fn clear_binding() {
    unsafe { glBindVertexArray(0) }
  }
}

/// The types of buffer object that you can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
  /// Array Buffers holds arrays of vertex data for drawing.
  Array = GL_ARRAY_BUFFER as isize,
  /// Element Array Buffers hold indexes of what vertexes to use for drawing.
  ElementArray = GL_ELEMENT_ARRAY_BUFFER as isize,
}

/// Basic wrapper for a [Buffer
/// Object](https://www.khronos.org/opengl/wiki/Buffer_Object).
pub struct Buffer(pub GLuint);
impl Buffer {
  /// Makes a new vertex buffer
  pub fn new() -> Option<Self> {
    let mut vbo = 0;
    unsafe {
      glGenBuffers(1, &mut vbo);
    }
    if vbo != 0 {
      Some(Self(vbo))
    } else {
      None
    }
  }

  /// Bind this vertex buffer for the given type
  pub fn bind(&self, ty: BufferType) {
    unsafe { glBindBuffer(ty as GLenum, self.0) }
  }

  /// Clear the current vertex buffer binding for the given type.
  pub fn clear_binding(ty: BufferType) {
    unsafe { glBindBuffer(ty as GLenum, 0) }
  }
}

/// Places a slice of data into a previously-bound buffer.
pub fn buffer_data(ty: BufferType, data: &[u8], usage: GLenum) {
  unsafe {
    glBufferData(
      ty as GLenum,
      data.len().try_into().unwrap(),
      data.as_ptr().cast(),
      usage,
    );
  }
}

/// The types of shader object.
pub enum ShaderType {
  /// Vertex shaders determine the position of geometry within the screen.
  Vertex = GL_VERTEX_SHADER as isize,
  /// Fragment shaders determine the color output of geometry.
  ///
  /// Also other values, but mostly color.
  Fragment = GL_FRAGMENT_SHADER as isize,
}

/// A handle to a [Shader
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Shader_objects)
pub struct Shader(pub GLuint);

impl Shader {
  /// Makes a new shader.
  ///
  /// Prefer the [`Shader::from_source`](Shader::from_source) method.
  ///
  /// Possibly skip the direct creation of the shader object and use
  /// [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag).
  pub fn new(ty: ShaderType) -> Option<Self> {
    let shader = unsafe { glCreateShader(ty as GLenum) };
    if shader != 0 {
      Some(Self(shader))
    } else {
      None
    }
  }

  /// Assigns a source string to the shader.
  ///
  /// Replaces any previously assigned source.
  pub fn set_source(&self, src: &str) {
    unsafe {
      glShaderSource(
        self.0,
        1,
        &(src.as_bytes().as_ptr().cast()),
        &(src.len().try_into().unwrap()),
      );
    }
  }

  /// Compiles the shader based on the current source.
  pub fn compile(&self) {
    unsafe { glCompileShader(self.0) };
  }

  /// Checks if the last compile was successful or not.
  pub fn compile_success(&self) -> bool {
    let mut compiled = 0;
    unsafe { glGetShaderiv(self.0, GL_COMPILE_STATUS, &mut compiled) };
    compiled == i32::from(GL_TRUE)
  }

  /// Gets the info log for the shader.
  ///
  /// Usually you use this to get the compilation log when a compile failed.
  pub fn info_log(&self) -> String {
    let mut needed_len = 0;
    unsafe { glGetShaderiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
    let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
    let mut len_written = 0_i32;
    unsafe {
      glGetShaderInfoLog(
        self.0,
        v.capacity().try_into().unwrap(),
        &mut len_written,
        v.as_mut_ptr().cast(),
      );
      v.set_len(len_written.try_into().unwrap());
    }
    String::from_utf8_lossy(&v).into_owned()
  }

  /// Marks a shader for deletion.
  ///
  /// Note: This _does not_ immediately delete the shader. It only marks it for
  /// deletion. If the shader has been previously attached to a program then the
  /// shader will stay allocated until it's unattached from that program.
  pub fn delete(self) {
    unsafe { glDeleteShader(self.0) };
  }

  /// Takes a shader type and source string and produces either the compiled
  /// shader or an error message.
  ///
  /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
  /// it makes a complete program from the vertex and fragment sources all at
  /// once.
  pub fn from_source(ty: ShaderType, source: &str) -> Result<Self, String> {
    let id = Self::new(ty)
      .ok_or_else(|| "Couldn't allocate new shader".to_string())?;
    id.set_source(source);
    id.compile();
    if id.compile_success() {
      Ok(id)
    } else {
      let out = id.info_log();
      id.delete();
      Err(out)
    }
  }
}

/// A handle to a [Program
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Program_objects)
pub struct ShaderProgram(pub GLuint);
impl ShaderProgram {
  /// Allocates a new program object.
  ///
  /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
  /// it makes a complete program from the vertex and fragment sources all at
  /// once.
  pub fn new() -> Option<Self> {
    let prog = unsafe { glCreateProgram() };
    if prog != 0 {
      Some(Self(prog))
    } else {
      None
    }
  }

  /// Attaches a shader object to this program object.
  pub fn attach_shader(&self, shader: &Shader) {
    unsafe { glAttachShader(self.0, shader.0) };
  }

  /// Links the various attached, compiled shader objects into a usable program.
  pub fn link_program(&self) {
    unsafe { glLinkProgram(self.0) };
  }

  /// Checks if the last linking operation was successful.
  pub fn link_success(&self) -> bool {
    let mut success = 0;
    unsafe { glGetProgramiv(self.0, GL_LINK_STATUS, &mut success) };
    success == i32::from(GL_TRUE)
  }

  /// Gets the log data for this program.
  ///
  /// This is usually used to check the message when a program failed to link.
  pub fn info_log(&self) -> String {
    let mut needed_len = 0;
    unsafe { glGetProgramiv(self.0, GL_INFO_LOG_LENGTH, &mut needed_len) };
    let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
    let mut len_written = 0_i32;
    unsafe {
      glGetProgramInfoLog(
        self.0,
        v.capacity().try_into().unwrap(),
        &mut len_written,
        v.as_mut_ptr().cast(),
      );
      v.set_len(len_written.try_into().unwrap());
    }
    String::from_utf8_lossy(&v).into_owned()
  }

  /// Sets the program as the program to use when drawing.
  pub fn use_program(&self) {
    unsafe { glUseProgram(self.0) };
  }

  /// Marks the program for deletion.
  ///
  /// Note: This _does not_ immediately delete the program. If the program is
  /// currently in use it won't be deleted until it's not the active program.
  /// When a program is finally deleted and attached shaders are unattached.
  pub fn delete(self) {
    unsafe { glDeleteProgram(self.0) };
  }

  /// Takes a vertex shader source string and a fragment shader source string
  /// and either gets you a working program object or gets you an error message.
  ///
  /// This is the preferred way to create a simple shader program in the common
  /// case. It's just less error prone than doing all the steps yourself.
  pub fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
    let p =
      Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
    let v = Shader::from_source(ShaderType::Vertex, vert)
      .map_err(|e| format!("Vertex Compile Error: {}", e))?;
    let f = Shader::from_source(ShaderType::Fragment, frag)
      .map_err(|e| format!("Fragment Compile Error: {}", e))?;
    p.attach_shader(&v);
    p.attach_shader(&f);
    p.link_program();
    v.delete();
    f.delete();
    if p.link_success() {
      Ok(p)
    } else {
      let out = format!("Program Link Error: {}", p.info_log());
      p.delete();
      Err(out)
    }
  }

  pub fn get_unif_location(&self, name: &str) -> i32 {
    unsafe { return glGetUniformLocation(self.0, CString::new(name).unwrap().as_ptr()) }
  }
}

/// The polygon display modes you can set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonMode {
  /// Just show the points.
  Point = GL_POINT as isize,
  /// Just show the lines.
  Line = GL_LINE as isize,
  /// Fill in the polygons.
  Fill = GL_FILL as isize,
}

/// Sets the font and back polygon mode to the mode given.
pub fn polygon_mode(mode: PolygonMode) {
  unsafe { glPolygonMode(GL_FRONT_AND_BACK, mode as GLenum) };
}


pub fn create_gl_texture_from_img(file_path: &str) -> u32 {
  let img = image::open(file_path).expect("Failed to load texture file");
  
  let img = img.flipv();
  let rgba = img.into_rgba8();
  let (width, height) = rgba.dimensions();
  let raw_data = rgba.as_raw();
  unsafe {
    let mut texture_id: u32 = 0;
    glGenTextures(1, &mut texture_id);
    
    glBindTexture(GL_TEXTURE_2D, texture_id);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
    
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
    
    glTexImage2D(
        GL_TEXTURE_2D,
        0,
        GL_RGBA8 as i32,
        width as i32,
        height as i32,
        0,
        GL_RGBA,
        GL_UNSIGNED_BYTE,
        raw_data.as_ptr() as *const c_void
      );
      glGenerateMipmap(GL_TEXTURE_2D);
      glBindTexture(GL_TEXTURE_2D, 0);
    texture_id
  }
}

pub fn create_gl_texture_from_bytes(width: u32, height: u32, data: &[u8]) -> u32 {
    assert_eq!(
        data.len(),
        (width * height * 4) as usize,
        "Data length does not match width * height * 4"
    );
    unsafe {

      let mut texture_id: u32 = 0;
      glGenTextures(1, &mut texture_id);
      glBindTexture(GL_TEXTURE_2D, texture_id);
      
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as i32);
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT as i32);
      
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as i32);
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as i32);
      
      glTexImage2D(
        GL_TEXTURE_2D,
        0,
        GL_RGBA8 as i32,
        width as i32,
        height as i32,
        0,
        GL_RGBA,
        GL_UNSIGNED_BYTE,
        data.as_ptr() as *const c_void,
      );
      
      glGenerateMipmap(GL_TEXTURE_2D);
      
      glBindTexture(GL_TEXTURE_2D, 0);
      
      texture_id
    }
}

/// A handy helper that generates a texture using a function/closure for each pixel.
pub fn generate_texture<F>(width: u32, height: u32, mut pixel_color_func: F) -> Vec<u8>
where
    F: FnMut(u32, u32) -> [u8; 4],
{
    let mut pixel_data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let rgba = pixel_color_func(x, y);
            
            pixel_data.extend_from_slice(&rgba);
        }
    }
    pixel_data
    // create_gl_texture_from_bytes(width, height, &pixel_data)
}