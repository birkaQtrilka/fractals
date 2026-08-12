#version 430
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(std430, binding = 0) readonly buffer SolidMap  { uint solid_map[]; };
layout(std430, binding = 4) readonly buffer Velocities { vec2 velocities[]; };
layout(std430, binding = 6) readonly buffer Smoke { float smoke[]; };
layout(std430, binding = 7) writeonly buffer TempSmoke { float temp_smoke[]; };

uniform int width;
uniform int height;
uniform float cell_size;
uniform float time_step;

bool is_solid(int idx) { return solid_map[idx] != 0u; }

// Duplicate get_u, get_v, sample_u, sample_v, sample_bilinear from advect_velocities here
float get_u(int x, int y) {
    x = clamp(x, 0, width);
    y = clamp(y, 0, height - 1);
    return velocities[x + y * (width + 1)].y;
}

float get_v(int x, int y) {
    x = clamp(x, 0, width - 1);
    y = clamp(y, 0, height);
    return velocities[x + y * (width + 1)].x;
}

float sample_u(float px, float py) {
    float sample_y = py - 0.5;
    int x0 = int(floor(px)), y0 = int(floor(sample_y));
    int x1 = x0 + 1, y1 = y0 + 1;
    float tx = px - float(x0), ty = sample_y - float(y0);
    return mix(mix(get_u(x0, y0), get_u(x1, y0), tx), mix(get_u(x0, y1), get_u(x1, y1), tx), ty);
}

float sample_v(float px, float py) {
    float sample_x = px - 0.5;
    int x0 = int(floor(sample_x)), y0 = int(floor(py));
    int x1 = x0 + 1, y1 = y0 + 1;
    float tx = sample_x - float(x0), ty = py - float(y0);
    return mix(mix(get_v(x0, y0), get_v(x1, y0), tx), mix(get_v(x0, y1), get_v(x1, y1), tx), ty);
}

vec2 sample_bilinear(float world_x, float world_y) {
    float px = world_x / cell_size;
    float py = world_y / cell_size;
    return vec2(sample_v(px, py), sample_u(px, py));
}

float sample_smoke(float px, float py) {
    px = px - 0.5;
    py = py - 0.5;
    int x = int(floor(px));
    int y = int(floor(py));
    float x_frac = clamp(px - float(x), 0.0, 1.0);
    float y_frac = clamp(py - float(y), 0.0, 1.0);

    int x0 = clamp(x, 0, width - 1);
    int x1 = clamp(x + 1, 0, width - 1);
    int y0 = clamp(y, 0, height - 1);
    int y1 = clamp(y + 1, 0, height - 1);

    float bottom_left  = smoke[x0 + y0 * width];
    float bottom_right = smoke[x1 + y0 * width];
    float top_left     = smoke[x0 + y1 * width];
    float top_right    = smoke[x1 + y1 * width];

    float interpolated_bottom = mix(bottom_left, bottom_right, x_frac);
    float interpolated_top = mix(top_left, top_right, x_frac);
    return mix(interpolated_bottom, interpolated_top, y_frac);
}

void main() {
    ivec2 p = ivec2(gl_GlobalInvocationID.xy);
    if (p.x >= width || p.y >= height) return;
    int i = p.y * width + p.x;

    if (is_solid(i)) {
        temp_smoke[i] = 0.0;
        return;
    }

    float center_x = (float(p.x) + 0.5) * cell_size;
    float center_y = (float(p.y) + 0.5) * cell_size;
    
    vec2 vel_at_center = sample_bilinear(center_x, center_y);
    float prev_x = center_x - vel_at_center.y * time_step; 
    float prev_y = center_y - vel_at_center.x * time_step;

    temp_smoke[i] = sample_smoke(prev_x / cell_size, prev_y / cell_size);
}