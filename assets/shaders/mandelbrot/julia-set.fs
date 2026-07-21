#version 330 core
in vec2 uv;
out vec4 final_color;
uniform vec2 zoom;
uniform vec4 offset;
uniform uint max_iterations;

uniform vec2 julia_const;

float map(float value, float oldMin, float oldMax,
          float newMin, float newMax)
{
  return newMin +
         (value - oldMin) * (newMax - newMin) /
         (oldMax - oldMin);
}

vec2 ds_add(vec2 a, vec2 b) {
    float sum_hi = a.x + b.x;
    
    float temp = sum_hi - a.x;
    float error = (a.x - (sum_hi - temp)) + (b.x - temp);
    
    float sum_lo = error + a.y + b.y;
    
    float final_hi = sum_hi + sum_lo;
    float final_lo = sum_lo - (final_hi - sum_hi);
    
    return vec2(final_hi, final_lo);
}

vec2 split(float a) {
    float temp = 4097.0 * a;
    float hi = temp - (temp - a);
    float lo = a - hi;
    return vec2(hi, lo);
}

vec2 ds_mul(vec2 a, vec2 b) {
    float prod_hi = a.x * b.x;
    
    vec2 a_split = split(a.x);
    vec2 b_split = split(b.x);
    
    float err = ((a_split.x * b_split.x - prod_hi) 
                + a_split.x * b_split.y 
                + a_split.y * b_split.x) 
                + a_split.y * b_split.y;
                
    err += a.x * b.y + a.y * b.x;
    
    float final_hi = prod_hi + err;
    float final_lo = err - (final_hi - prod_hi);
    
    return vec2(final_hi, final_lo);
}

void main() {
  uint n = 0;
  vec2 mapped_a = vec2(map(uv.x, 0, 1, -2, 2), 0.0);
  vec2 mapped_b = vec2(map(uv.y, 0, 1, -2, 2), 0.0);
  vec2 a = ds_mul( mapped_a, zoom);
  vec2 b = ds_mul( mapped_b, zoom); 
  a = ds_add(a, offset.xy);
  b = ds_add(b, offset.zw);

  while (n < max_iterations) {
    vec2 next_a = ds_add(ds_mul(a,a), -ds_mul(b,b));
    vec2 next_b = ds_mul(vec2(2.0, 0.0), ds_mul(a,b));

    a = ds_add(next_a, vec2(-julia_const.x,  0.0));
    b = ds_add(next_b, vec2(-julia_const.y,  0.0));
    vec2 mag_sq = ds_add(ds_mul(a,a), ds_mul(b,b));
    if (mag_sq.x > 16.0) { break; }
    n++;
  }

  float speed = 0.2f;
  float r = map( sin(float(n)*speed ), -1,1, 0, 1);
  float g = map( sin(float(n)*speed + 30), -1,1, 0, 1);
  float b1 = map( sin(float(n)*speed +10), -1,1, 0, 1);
  vec4 col = vec4(r,g,b1, 1.0);
  if(n == max_iterations) col = vec4(0,0,0,1);
  final_color = col;
}