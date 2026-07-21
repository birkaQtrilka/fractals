#version 330 core
in vec2 uv;
out vec4 final_color;
uniform float zoom;
uniform vec2 offset;

uniform vec2 julia_const;

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

  while (n < max_iterations) {
    float next_a = a * a - b * b;
    float next_b = 2 * a * b;
    a = next_a - julia_const.x;
    b = next_b - julia_const.y;
    if(abs(a+b) > 16) { break; }
    n++;
  }

  float speed = 0.2f;
  float r = map( sin(n*speed ), -1,1, 0, 1);
  float g = map( sin(n*speed + 30), -1,1, 0, 1);
  float b1 = map( sin(n*speed +10), -1,1, 0, 1);
  vec4 col = vec4(r,g,b1, 1.0);
  // float bright = map(n, 0, max_iterations, 0, 1);
  if(n == max_iterations) col = vec4(0,0,0,1);
  final_color = col;
}