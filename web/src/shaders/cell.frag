#version 300 es

precision mediump float;

uniform sampler2D u_sampler;
in vec2 v_tex_coord;

out vec4 FragColor;

void main() {
    vec4 color = texture(u_sampler, v_tex_coord);
    FragColor = vec4(color.rgb * color.a, 1.0 - color.a);
}