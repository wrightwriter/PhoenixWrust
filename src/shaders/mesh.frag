W_UBO_DEF{
    vec4 values;
}

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

void main() {
    // registers.dst.values[index] = registers.src.values[index];

    outColor = vec4(fragColor, 1.0);
    outColor = sin(gl_FragCoord.xyxy/shared_ubo.R.xyxy);
    vec2 uv = gl_FragCoord.xy/shared_ubo.R.xy;
    outColor.xyz = tex(shared_textures[2],uv).xyz;
    // outColor = vec4(vec3(1,1,1), 1.0);
    // if (PC.frame % 2 == 1){
    //     outColor = 1. - outColor;
    // }
    // outColor.r += PC.ubo.values[0].r;
    // outColor.r += PC.ubo.values.r;
    //outColor.r += shared_ubo.values[0].r;
    // outColor.b += object_ubo.values[0].r;
    //outColor.b += PC.ubo.values[0].r;

    // outColor.g += imageLoad(shared_images[0], ivec2(1)).x;

        
}
