#version 330 core
in vec2 uv;
out vec4 final_color;
uniform vec2 zoom;
uniform vec4 offset;

uniform vec2 julia_const;

float map(float value, float oldMin, float oldMax,
          float newMin, float newMax)
{
  return newMin +
         (value - oldMin) * (newMax - newMin) /
         (oldMax - oldMin);
}

// GLSL Shader code
vec2 ds_add(vec2 a, vec2 b) {
    // 1. Add the high parts together
    float sum_hi = a.x + b.x;
    
    // 2. Figure out what got rounded off during step 1
    float temp = sum_hi - a.x;
    float error = (a.x - (sum_hi - temp)) + (b.x - temp);
    
    // 3. Add the captured error to the existing low parts
    float sum_lo = error + a.y + b.y;
    
    // 4. Normalize the result back into a strict high/low pair
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
    // 1. Multiply the high parts normally
    float prod_hi = a.x * b.x;
    
    // 2. Split the high parts into 16-bit halves to calculate the error exactly
    vec2 a_split = split(a.x);
    vec2 b_split = split(b.x);
    
    // 3. Calculate the exact rounding error of prod_hi
    float err = ((a_split.x * b_split.x - prod_hi) 
                + a_split.x * b_split.y 
                + a_split.y * b_split.x) 
                + a_split.y * b_split.y;
                
    // 4. Add the cross-multiplications of the low parts
    err += a.x * b.y + a.y * b.x;
    
    // 5. Normalize into a strict high/low pair
    float final_hi = prod_hi + err;
    float final_lo = err - (final_hi - prod_hi);
    
    return vec2(final_hi, final_lo);
}

// vec2 where x is number and y is 10 to the power
void main() {
  int max_iterations = 100;
  int n = 0;
  vec2 mapped_a = vec2(map(uv.x, 0, 1, -2, 2), 0.0);
  vec2 mapped_b = vec2(map(uv.y, 0, 1, -2, 2), 0.0);
  vec2 a = ds_mul( mapped_a, zoom);
  vec2 b = ds_mul( mapped_b, zoom); 
  a = ds_add(a, offset.xy);
  b = ds_add(b, offset.zw);

  while (n < max_iterations) {
    vec2 next_a = ds_mul(a,a) - ds_mul(b,b);
    vec2 next_b = ds_mul(vec2(2.0,0.0), ds_mul(a,b));

    a = ds_add(next_a, vec2(-julia_const.x,  0.0));
    b = ds_add(next_b, vec2(-julia_const.y,  0.0));
    vec2 mag_sq = ds_add(ds_mul(a,a), ds_mul(b,b));
    if (mag_sq.x > 16.0) { break; }
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