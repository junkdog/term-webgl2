#version 300 es

precision mediump float;

uniform mediump sampler2DArray u_sampler;

// packs 8b: 2b layer, 3b fg.rgb, 3b bg.rgb
flat in uvec2 v_packed_data;
in vec2 v_tex_coord;

out vec4 FragColor;

float normalize_lsb(uint value) {
    return (float(value & 0xFFu)) / 255.0;
}

void main() {
    float layer = float(v_packed_data.x & 0xFFFFu);
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

    float a = 1.0 - texture(u_sampler, vec3(v_tex_coord, layer)).a;
    FragColor = vec4(mix(fg, bg, a), 1.0);
}