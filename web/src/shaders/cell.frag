#version 300 es

precision mediump float;

uniform mediump sampler2DArray u_sampler;

in vec2 v_tex_coord;
in float v_depth;
in vec4 v_fg_color;
in vec4 v_bg_color;

out vec4 FragColor;

void main() {
    vec4 color = texture(u_sampler, vec3(v_tex_coord, v_depth));
    float a = 1.0 - color.a;

    vec4 c = mix(v_fg_color, v_bg_color, a);
//    FragColor = vec4(color.rgb * color.a, a);
    FragColor = vec4(c.rgb, 1.0);
}