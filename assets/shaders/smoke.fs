#version 330 core

uniform float cellSize;
uniform float size;
uniform ivec2 resolution;
uniform sampler2D gridTexture; 

in vec2 uv;
out vec4 FragColor;

void main() {
  // uv it's 0 to 1
  // need to figure out ratio
  float ratio = resolution.x / float(resolution.y);
  // assuming height is shorter
  // need to know 
  float x = uv.x * ratio;
  float y = 1-uv.y;
  float density = 0;
  if(y * size < cellSize || y * size > size-cellSize || x * size < cellSize || x * size > size-cellSize) {
    density = 1.0;
    if(x * size < size)
      FragColor = vec4(1, .5, 0, 1.0);
    else 
      FragColor = vec4(0.0,0.0,0.0,1.0);
    return;
  }
  density = texture(gridTexture, vec2(x, y)).r;
  
  FragColor = vec4(density, density, density, 1.0);
  // FragColor = vec4(uv.x, 1-uv.y,0.0,1.0);
}