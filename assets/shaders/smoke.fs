#version 330 core

uniform float cellSize;
uniform float size;
uniform vec2 gridSize;
uniform ivec2 resolution;
uniform sampler2D gridTexture; 

in vec2 uv;
out vec4 FragColor;

void main() {
  float ratio = resolution.x / float(resolution.y);
  
  float phys_x = uv.x * ratio * size;
  float phys_y = (1.0 - uv.y) * size;
  
  if(phys_y < cellSize || phys_y > gridSize.y - cellSize || phys_x < cellSize || phys_x > gridSize.x - cellSize) {
    FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    return;
  }
  
  // Transform physical coordinates onto [0..1] range for texture lookup
  float tx = phys_x / gridSize.x;
  float ty = phys_y / gridSize.y;
  
  float density = texture(gridTexture, vec2(tx, ty)).r;
  
  FragColor = vec4(density, density, density, 1.0);
}