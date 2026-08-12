#version 430
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(std430, binding = 0) readonly buffer SolidMap  { uint solid_map[]; };
layout(std430, binding = 4) readonly buffer Velocities { vec2 velocities[]; };
layout(std430, binding = 5) writeonly buffer TempVelocities { vec2 temp_velocities[]; };

uniform int width;
uniform int height;
uniform float cell_size;
uniform float time_step;

bool is_solid(int idx) { return solid_map[idx] != 0u; }

float get_u(int x, int y) {
    x = clamp(x, 0, width);
    y = clamp(y, 0, height - 1);
    int vi = x + y * (width + 1);
    return velocities[vi].y; // y matches Pair.left
}

float get_v(int x, int y) {
    x = clamp(x, 0, width - 1);
    y = clamp(y, 0, height);
    int vi = x + y * (width + 1);
    return velocities[vi].x; // x matches Pair.top
}

float sample_u(float px, float py) {
    float sample_y = py - 0.5;
    int x0 = int(floor(px));
    int y0 = int(floor(sample_y));
    int x1 = x0 + 1;
    int y1 = y0 + 1;
    float tx = px - float(x0);
    float ty = sample_y - float(y0);

    float u0 = mix(get_u(x0, y0), get_u(x1, y0), tx);
    float u1 = mix(get_u(x0, y1), get_u(x1, y1), tx);
    return mix(u0, u1, ty);
}

float sample_v(float px, float py) {
    float sample_x = px - 0.5;
    int x0 = int(floor(sample_x));
    int y0 = int(floor(py));
    int x1 = x0 + 1;
    int y1 = y0 + 1;
    float tx = sample_x - float(x0);
    float ty = py - float(y0);

    float v0 = mix(get_v(x0, y0), get_v(x1, y0), tx);
    float v1 = mix(get_v(x0, y1), get_v(x1, y1), tx);
    return mix(v0, v1, ty);
}

vec2 sample_bilinear(float world_x, float world_y) {
    float px = world_x / cell_size;
    float py = world_y / cell_size;
    float vx = sample_u(px, py);
    float vy = sample_v(px, py);
    return vec2(vy, vx);
}

void main() {
    ivec2 p = ivec2(gl_GlobalInvocationID.xy);
    if (p.x >= width || p.y >= height) return;
    
    int i = p.y * width + p.x;
    int vi = p.x + p.y * (width + 1);

    if (is_solid(i)) {
        temp_velocities[vi] = vec2(0.0, 0.0);
        return;
    }

    bool is_top_solid = is_solid(i - width);
    bool is_left_solid = is_solid(i - 1);

    float new_left_vel = 0.0;
    if (!is_left_solid) {
        float face_u_x = float(p.x) * cell_size;
        float face_u_y = (float(p.y) + 0.5) * cell_size;

        vec2 vel_at_face_u = sample_bilinear(face_u_x, face_u_y);
        float prev_x = face_u_x - vel_at_face_u.y * time_step; // .y is left/U
        float prev_y = face_u_y - vel_at_face_u.x * time_step; // .x is top/V

        new_left_vel = sample_u(prev_x / cell_size, prev_y / cell_size);
    }

    float new_top_vel = 0.0;
    if (!is_top_solid) {
        float face_v_x = (float(p.x) + 0.5) * cell_size;
        float face_v_y = float(p.y) * cell_size;

        vec2 vel_at_face_v = sample_bilinear(face_v_x, face_v_y);
        float prev_x = face_v_x - vel_at_face_v.y * time_step;
        float prev_y = face_v_y - vel_at_face_v.x * time_step;

        new_top_vel = sample_v(prev_x / cell_size, prev_y / cell_size);
    }

    temp_velocities[vi] = vec2(new_top_vel, new_left_vel);
}