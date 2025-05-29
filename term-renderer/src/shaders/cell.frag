#version 300 es

precision mediump float;

uniform mediump sampler3D u_sampler;

// packs 8b: 2b layer, 3b fg.rgb, 3b bg.rgb
flat in uvec2 v_packed_data;
in vec2 v_tex_coord;

out vec4 FragColor;

const float u_num_slices = 16.0; // Number of slices in the 3D texture

float normalize_lsb(uint value) {
    return (float(value & 0xFFu)) / 255.0;
}

void main() {
    // Extract sequential glyph index from packed data
    uint glyph_index = v_packed_data.x & 0xFFFFu;

    // Calculate 3D position from sequential index
    uint slice = glyph_index / 16u;
    uint pos_in_slice = glyph_index % 16u;
    uint grid_x = pos_in_slice % 4u;
    uint grid_y = pos_in_slice / 4u;

    vec3 tex_coord = vec3(
        (float(grid_x) + v_tex_coord.x) / 4.0,
        (float(grid_y) + v_tex_coord.y) / 4.0,
        (float(slice) + 0.5) / u_num_slices
    );

    vec4 glyph = texture(u_sampler, tex_coord);

    vec3 fg = vec3(
        normalize_lsb(v_packed_data.x >> 16),
        normalize_lsb(v_packed_data.x >> 24),
        normalize_lsb(v_packed_data.y)
    );
    vec3 bg = vec3(
        normalize_lsb(v_packed_data.y >> 8),
        normalize_lsb(v_packed_data.y >> 16),
        normalize_lsb(v_packed_data.y >> 24)
    );

    float a = 1.0 - glyph.a;
    FragColor = vec4(mix(fg, bg, a), 1.0);
}