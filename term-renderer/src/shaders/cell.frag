#version 300 es

precision mediump float;

// uniforms
uniform mediump sampler3D u_sampler;
layout(std140) uniform CellUniforms {
    mat4 u_projection;
    vec2 u_cell_size;
    float u_num_slices;
};


// packs 8b: 2b layer, 3b fg.rgb, 3b bg.rgb
// ref: https://github.com/junkdog/term-webgl2?tab=readme-ov-file#glyph-id-bit-layout-16-bit
flat in uvec2 v_packed_data;
in vec2 v_tex_coord;

out vec4 FragColor;

float normalize_lsb(uint value) {
    return (float(value & 0xFFu)) / 255.0;
}

void main() {
    // Extract sequential glyph index from packed data
    uint glyph_index = v_packed_data.x & 0xFFFFu;

    // Calculate 3D position from sequential index
    uint slice = (glyph_index & 0xCFFFu) >> 4; // strip underline/strikethrough bits
    uint pos_in_slice = glyph_index & 0x0Fu;
    uint grid_x = pos_in_slice % 4u;
    uint grid_y = pos_in_slice / 4u;

    vec3 tex_coord = vec3(
        (float(grid_x) + v_tex_coord.x) / 4.0,
        (float(grid_y) + v_tex_coord.y) / 4.0,
        (float(slice) + 0.5) / u_num_slices
    );

    vec4 glyph = texture(u_sampler, tex_coord);

    // 0.0 for normal glyphs, 1.0 for emojis: used for determining color source
    float emoji_factor = float((glyph_index >> 11) & 0x1u);

    // color for normal glyphs are taken from the packed data;
    // emoji colors are sampled from the texture directly
    vec3 fg = mix(
        vec3(
            normalize_lsb(v_packed_data.x >> 16),
            normalize_lsb(v_packed_data.x >> 24),
            normalize_lsb(v_packed_data.y)
        ),
        glyph.rgb,
        emoji_factor
    );

    vec3 bg = vec3(
        normalize_lsb(v_packed_data.y >> 8),
        normalize_lsb(v_packed_data.y >> 16),
        normalize_lsb(v_packed_data.y >> 24)
    );

    FragColor = vec4(mix(bg, fg, glyph.a), 1.0);
}