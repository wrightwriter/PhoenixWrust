#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_buffer_reference_uvec2 : require
#extension GL_EXT_scalar_block_layout : enable


layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

 // These define pointer types.
// layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer ReadVec4
layout(buffer_reference, scalar, buffer_reference_align = 1, align = 1) readonly buffer ReadVec4
{
    vec4 values[];
};



// layout(buffer_reference, std430, buffer_reference_align = 16) writeonly buffer WriteVec4
// {
//     vec4 values[];
// };

// layout(buffer_reference, std430, buffer_reference_align = 4) readonly buffer UnalignedVec4
// {
//     vec4 value;
// };


layout( push_constant ) uniform constants{
    ReadVec4 ubo;
	int frame;
} PC;


layout(set = 0, binding=0) uniform SharedUbo{
    vec4 values[];
} shared_ubo;

layout(rgba32f, set = 0, binding = 1) uniform image2D shared_images[10];


// layout(set = 1, binding=0) uniform ObjectUbo{
//     vec4 values[];
// } object_ubo;

void main() {
    // registers.dst.values[index] = registers.src.values[index];

    outColor = vec4(fragColor, 1.0);
    // if (PC.frame % 2 == 1){
    //     outColor = 1. - outColor;
    // }
    outColor.r += PC.ubo.values[0].r;
    //outColor.r += shared_ubo.values[0].r;
    // outColor.b += object_ubo.values[0].r;
    //outColor.b += PC.ubo.values[0].r;

    outColor.g += imageLoad(shared_images[0], ivec2(1)).x;
        
}
