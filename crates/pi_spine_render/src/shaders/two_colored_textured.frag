#version 450

#define SHADER_NAME fragment:TwoColoredTextured

layout(location = 0) in vec4 v_light;
layout(location = 1) in vec4 v_dark;
layout(location = 2) in vec2 v_texCoords;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform Param {
    mat4 u_projTrans;
    vec4 u_maskflag;
    vec4 u_visibility;
};

layout(set = 0, binding = 1) uniform texture2D u_texture;
layout(set = 0, binding = 2) uniform sampler sampler_u_texture;

vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

void main() {
    vec4 texColor = texture(sampler2D(u_texture, sampler_u_texture), v_texCoords);
    vec4 baseColor = vec4(1., 1., 1., 1.);
    baseColor.a = texColor.a * v_light.a;
    baseColor.rgb = ((texColor.a - 1.0) * v_dark.a + 1.0 - texColor.rgb) * v_dark.rgb + texColor.rgb * v_light.rgb;

    if (u_maskflag.w == 1.0) {
        baseColor.rgb = u_maskflag.rgb * baseColor.a;
    }
    if (u_maskflag.w == 2.0) {
        vec4 c = baseColor;
        vec3 hsvValue = u_maskflag.rgb;

        vec3 hsv = rgb2hsv(c.rgb);
        hsv.r += hsvValue.r;
        c.rgb = hsv2rgb(hsv);

        // 注：saturate大于0时，公式和PS不大一样
        float gray = max(c.r, max(c.g, c.b)) + min(c.r, min(c.g, c.b));
        c.rgb = mix(c.rgb, vec3(0.5 * gray), -hsvValue.g);

        if (hsvValue.b >= 0.0) {
            c.rgb = mix(c.rgb, vec3(1.0), hsvValue.b);
        } else {
            c.rgb *= 1.0 + hsvValue.b;
        }
        baseColor = c;
    }

    baseColor.rgb *= u_visibility.x;
    baseColor.a   *= u_visibility.z;
    baseColor.rgb *= mix(1.0, baseColor.a, u_visibility.y);
    gl_FragColor = baseColor;
}