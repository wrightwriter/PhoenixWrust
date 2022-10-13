#version 450
#extension GL_ARB_separate_shader_objects : enable

#W_DONT_PREPROCESS


#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_buffer_reference_uvec2 : require
#extension GL_EXT_scalar_block_layout : enable

layout(set = 0, binding=0, std430) uniform SharedUbo{
    // vec4 values[];
    mat4 viewMat;
    mat4 projMat;
} shared_ubo; 


layout(location = 0) out vec3 fragColor;

vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5));

vec3 colors[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0));

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);

    gl_Position = shared_ubo.viewMat * gl_Position;
    gl_Position = shared_ubo.projMat * gl_Position;
    // gl_Position *= 0.0;

    

    fragColor = colors[gl_VertexIndex];
}
