#version 330 core
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