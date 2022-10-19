W_UBO_DEF{
    vec4 values;
}

layout(location = 0) in vec3 vColor;
layout(location = 1) in vec3 vNorm;

layout(location = 0) out vec4 oC;

void main() {
    oC = vec4(vColor, 1.0);
    
    vec3 n = vNorm;
    oC = vec4(n * 0.5 + 0.5, 1.0);
    vec2 uv = U.xy/R.xy;
    
    
    oC = max(pow(oC,vec4(0.454545)),.0);





    // if (PC.frame % 2 == 1){
    //     outColor = 1. - outColor;
    // }
    // outColor.xyz = tex(shared_textures[2],uv).xyz;
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
