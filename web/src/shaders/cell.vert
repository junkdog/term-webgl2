#version 300 es
precision mediump float;

// cell geometry attributes
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_tex_coord;

// instance attributes
layout(location = 2) in uvec2 a_instance_pos;
layout(location = 3) in float a_depth;
layout(location = 4) in uint a_fg_color;
layout(location = 5) in uint a_bg_color;

// uniforms
layout(std140) uniform CellUniforms {
    mat4 u_projection;
    vec2 u_cell_size;
};

out vec2 v_tex_coord;
out vec4 v_fg_color;
out vec4 v_bg_color;
out float v_depth;

vec4 unpack_color(uint color) {
    float r = float((color >> 24) & 0xFFu) / 255.0;
    float g = float((color >> 16) & 0xFFu) / 255.0;
    float b = float((color >> 8) & 0xFFu) / 255.0;
    float a = float(color & 0xFFu) / 255.0;
    return vec4(r, g, b, a);
}

void main() {
    v_tex_coord = a_tex_coord;
    v_depth = a_depth;
    v_fg_color = unpack_color(a_fg_color);
    v_bg_color = unpack_color(a_bg_color);

    vec2 offset = vec2(
        float(a_instance_pos.x) * u_cell_size.x,
        float(a_instance_pos.y) * u_cell_size.y
    );

    gl_Position = u_projection * vec4(a_pos + offset, 0.0, 1.0);
}