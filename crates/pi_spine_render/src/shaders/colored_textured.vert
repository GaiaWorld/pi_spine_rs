#version 450

#define SHADER_NAME fragment:ColoredTextured

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec4 a_color;
layout(location = 2) in vec2 a_texCoords;

layout(location = 0) out vec4 v_color;
layout(location = 1) out vec2 v_texCoords;

layout(set = 0, binding = 0) uniform Param {
    mat4 u_projTrans;
    vec4 u_maskflag;
    vec4 u_visibility;
};

void main() {
    v_color = a_color;
    v_texCoords = a_texCoords;
    vec4 pos = u_projTrans * vec4(a_position, 0., 1.);
    pos.zw = (pos.z + pos.w) * 0.5;
    gl_Position = pos;
}