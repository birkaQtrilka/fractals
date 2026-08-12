#version 430
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(std430, binding = 0) readonly buffer SolidMap  { uint solid_map[]; };
layout(std430, binding = 1) readonly buffer Pressures { float pressures[]; };
layout(std430, binding = 4) buffer Velocities         { vec2 velocities[]; }; // x=top, y=left

uniform int width;
uniform int height;
uniform float k;

bool is_solid(int idx) { return solid_map[idx] != 0u; }

void main() {
    ivec2 p = ivec2(gl_GlobalInvocationID.xy);
    if (p.x >= width || p.y >= height) return;
    int i = p.y * width + p.x;
    int vi = p.x + p.y * (width + 1);

    if (is_solid(i)) {
        velocities[vi] = vec2(0.0, 0.0);
        return;
    }

    float v_top = velocities[vi].x;
    float v_left = velocities[vi].y;

    if (!is_solid(i - width)) {
        v_top -= k * (pressures[i] - pressures[i - width]);
    } else {
        v_top = 0.0;
    }

    if (!is_solid(i - 1)) {
        v_left -= k * (pressures[i] - pressures[i - 1]);
    } else {
        v_left = 0.0;
    }

    velocities[vi] = vec2(v_top, v_left);
}