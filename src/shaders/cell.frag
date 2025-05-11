#version 300 es

precision mediump float;

uniform sampler2D u_sampler;
in vec2 v_tex_coord;

out vec4 FragColor;

void main() {
    FragColor = texture(u_sampler, v_tex_coord);
}