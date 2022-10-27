
W_PC_DEF{
  UboObject ubo;
  uint8_t indices_buff_idx;
  uint8_t vertex_buff_idx;
  uint8_t diffuse_tex_idx;
  uint8_t normal_tex_idx;
  uint8_t metallic_roughness_tex_idx;
  uint8_t occlusion_tex_idx;

  // BuffIndices indices;
  // BuffVerts verts;
}


layout(location = 0) in vec3 vColor;
layout(location = 1) in vec3 vNorm;
layout(location = 2) in vec2 vUv;

layout(location = 0) out vec4 oC;
layout(location = 1) out vec4 oGNorm;
// layout(location = 2) out vec4 oGPotato;

void main() {
    vec3 n = vNorm;
    vec2 uv = U.xy/R.xy;
    
    vec4 albedo = texMip(shared_textures[int(PC.diffuse_tex_idx)-1], (vUv));
    // vec4 normal_map = tex(shared_textures[int(PC.normal_tex_idx)-1], (vUv));
    // vec4 occlusion_map = tex(shared_textures[int(PC.occlusion_tex_idx)-1], (vUv));
    // vec4 metallic_roughness_map = tex(shared_textures[int(PC.metallic_roughness_tex_idx)-1], (vUv));
    // vec4 albedo = vec4(1,0,0,0);

    // oC = vNorm.xyzx;
    oC = pow(abs(albedo),vec4(1./0.4545));
    // oC = albedo.xyzz;
    oC.w = 1.;

    oGNorm = vec4(vNorm.xyz,1);

    // oGNorm = vec4(0,0,1.,1);

    // oGPotato = vec4(0,1,1,1);
    // if(int(PC.occlusion_tex_idx) != 69 -1){
    //     oC *= occlusion_map;
    // }

    // oC = abs(oC);
    //   uint8_t diffuse_tex_idx;
    //   uint8_t diffuse_tex_idx;
    //   uint8_t normal_tex_idx;
    //   uint8_t metallic_roughness_tex_idx;
    //   uint8_t occlusion_tex_idx;

    // oC = fract(vUv.xyxy);
    





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

    // outColor.g += imageLoad(shared_images_rgba32f[0], ivec2(1)).x;
}
