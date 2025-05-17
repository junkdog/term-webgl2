#version 300 es

precision mediump float;

uniform mediump sampler2DArray u_sampler;

in vec2 v_tex_coord;
in float v_depth;

out vec4 FragColor;

void main() {
    vec4 color = texture(u_sampler, vec3(v_tex_coord, v_depth));
    FragColor = vec4(color.rgb * color.a, 1.0 - color.a);
}