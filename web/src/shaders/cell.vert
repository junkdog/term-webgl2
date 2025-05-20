#version 300 es
precision mediump float;

// cell geometry attributes
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_tex_coord;

// instance attributes
layout(location = 2) in uvec2 a_instance_pos;
layout(location = 3) in uvec2 a_packed_data;

// uniforms
layout(std140) uniform CellUniforms {
    mat4 u_projection;
    vec2 u_cell_size;
};

out vec2 v_tex_coord;
out vec4 v_fg_color;
out vec4 v_bg_color;
out float v_depth;

float normalize_lsb(uint value) {
    return (float(value & 0xFFu)) / 255.0;
}

void main() {
    v_tex_coord = a_tex_coord;

    v_depth = float(a_packed_data.x & 0xFFFFu);
    v_fg_color = vec4(
        normalize_lsb(a_packed_data.x >> 16),
        normalize_lsb(a_packed_data.x >> 24),
        normalize_lsb(a_packed_data.y) / 255.0,
        1.0
    );
    v_bg_color = vec4(
        normalize_lsb(a_packed_data.y >> 8),
        normalize_lsb(a_packed_data.y >> 16),
        normalize_lsb(a_packed_data.y >> 24),
        1.0
    );

    vec2 offset = vec2(
        float(a_instance_pos.x) * u_cell_size.x,
        float(a_instance_pos.y) * u_cell_size.y
    );

    gl_Position = u_projection * vec4(a_pos + offset, 0.0, 1.0);
}