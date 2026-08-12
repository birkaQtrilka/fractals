use std::ffi::c_void;
use ogl33::{GLuint, GLenum, GLint, GLsizei, GLbitfield, GLboolean};

pub const GL_COMPUTE_SHADER: GLenum = 0x91B9;
pub const GL_SHADER_STORAGE_BUFFER: GLenum = 0x90D2;
pub const GL_SHADER_STORAGE_BARRIER_BIT: GLbitfield = 0x2000;
pub const GL_ALL_BARRIER_BITS: GLbitfield = 0xFFFFFFFF;
pub const GL_READ_WRITE: GLenum = 0x88BA;
pub const GL_RGBA32F: GLenum = 0x8814;

type DispatchComputeFn = unsafe extern "system" fn(GLuint, GLuint, GLuint);
type MemoryBarrierFn = unsafe extern "system" fn(GLbitfield);
type BindImageTextureFn = unsafe extern "system" fn(
  GLuint, GLuint, GLint, GLboolean, GLint, GLenum, GLenum,
);

pub struct ComputeExt {
  pub dispatch_compute: DispatchComputeFn,
  pub memory_barrier: MemoryBarrierFn,
  pub bind_image_texture: BindImageTextureFn,
}

impl ComputeExt {
  pub unsafe fn load(get_proc: impl Fn(&str) -> *const c_void) -> Self {
    // Helper to load and verify the pointer isn't null
    let load_ptr = |name: &str| {
        let ptr = get_proc(name);
        assert!(!ptr.is_null(), "Failed to load OpenGL function: {}", name);
        ptr
    };

    unsafe {
      Self {
        // Note the added \0 at the end of each string
        dispatch_compute: std::mem::transmute(load_ptr("glDispatchCompute\0")),
        memory_barrier: std::mem::transmute(load_ptr("glMemoryBarrier\0")),
        bind_image_texture: std::mem::transmute(load_ptr("glBindImageTexture\0")),
      }
    } 
  }
}