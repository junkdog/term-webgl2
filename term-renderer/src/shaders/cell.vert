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
    vec2 u_cell_size; // unpadded cell size in pixels
    vec2 u_padding_frac; // padding as fraction of cell size
    float u_num_slices;
};

// packs 8b: 2b layer, 3b fg.rgb, 3b bg.rgb
flat out uvec2 v_packed_data;
out vec2 v_tex_coord;

void main() {
    v_tex_coord = a_tex_coord;
    v_packed_data = a_packed_data;

    vec2 offset = vec2(
        floor(float(a_instance_pos.x) * u_cell_size.x + 0.5), // pixel-snapped
        floor(float(a_instance_pos.y) * u_cell_size.y + 0.5)  // pixel-snapped
    );

    gl_Position = u_projection * vec4(a_pos + offset, 0.0, 1.0);
}