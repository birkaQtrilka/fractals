#version 330 core

uniform float cellSize;
uniform float size;
uniform sampler2D gridTexture; 

in vec2 uv;
out vec4 FragColor;

void main() {
  float density = texture(gridTexture, uv).r;
  
  FragColor = vec4(density, density, density, 1.0);
}