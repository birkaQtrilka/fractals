#version 430
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(std430, binding = 0) readonly buffer SolidMap  { uint  solid_map[]; };
layout(std430, binding = 2) writeonly buffer Rhs      { float rhs[]; };
layout(std430, binding = 3) writeonly buffer InvTotal { float inv_total[]; };
layout(std430, binding = 4) readonly buffer Velocities { vec2 velocities[]; };

uniform int width;
uniform int height;
uniform float ady; 

bool is_solid(int idx) { return solid_map[idx] != 0u; }

void main() {
    ivec2 p = ivec2(gl_GlobalInvocationID.xy);
    if (p.x >= width || p.y >= height) return;
    int i = p.y * width + p.x;

    if (is_solid(i)) {
        rhs[i] = 0.0;
        inv_total[i] = 0.0;
        return;
    }

    float total = 0.0;
    if (p.y > 0            && !is_solid(i - width))  total += 1.0;
    if (p.x > 0            && !is_solid(i - 1))      total += 1.0;
    if (p.x + 1 < width    && !is_solid(i + 1))      total += 1.0;
    if (p.y + 1 < height   && !is_solid(i + width))  total += 1.0;

    int vi = p.x + p.y * (width + 1);
    float top = velocities[vi].x;
    float left = velocities[vi].y;
    float bottom = velocities[vi + width + 1].x;
    float right = velocities[vi + 1].y;
    float div = right - left + bottom - top;

    rhs[i] = -ady * div;
    inv_total[i] = total > 0.0 ? 1.0 / total : 0.0;
}