use image::ColorType;
use noise::{NoiseFn, Perlin, Seedable};
use crate::learn_opengl::generate_texture;
pub struct Field {
  scale: f64,
  // width: i32,
  // height: i32,

}

impl Field {
  pub fn new(scale: f64, width: u32, height: u32) -> Field {
    let perlin = Perlin::new(1);
    
    let tex_arr = generate_texture(width, height, |x, y| {
      let brightness = (perlin.get([x as f64 / scale, y as f64 / scale]) * 255.0) as u8;
      [brightness, brightness, brightness, 255]
    });

    image::save_buffer(
        "my_array_texture.png",
        &tex_arr,
        width,
        height,
        ColorType::Rgba8, // Tells the crate how to interpret the bytes
    ).expect("Failed to save image");

    Field {
      scale 

    }
  }
}