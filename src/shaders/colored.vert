#version 450

#define SHADER_NAME fragment:Colored

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec4 a_color;

layout(location = 0) out vec4 v_color;

layout(set = 0, binding = 0) uniform Param {
    mat4 u_projTrans;
    vec4 u_maskflag;
};

void main() {
    v_color = a_color;
    gl_Position = u_projTrans * vec4(a_position, 0., 1.);
}