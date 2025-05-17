#version 300 es

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_tex_coord;
layout(location = 2) in float a_depth;

uniform mat4 u_projection;

out vec2 v_tex_coord;
out float v_depth;

void main() {
    v_tex_coord = a_tex_coord;
    v_depth = a_depth;
    gl_Position = u_projection * vec4(a_pos, 0.0, 1.0);
}