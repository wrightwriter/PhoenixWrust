

layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 oC;

W_PC_DEF{
  UboObject ubo;
  uint8_t idx_galbedo;
  uint8_t idx_gnorm;
  uint8_t idx_vid;
  // BuffIndices indices;
  // BuffVerts verts;
}






void main() {
    // oC = tex(shared_textures[int(PC.idx_gbuff)-1],fract(vUv.xy));
    vec2 uv = vUv.xy;
    uv += 1.;
    vec2 uvn = uv*0.5;
    // uv *= rot(T);
    // oC = imageLoad(shared_images[max(int(PC.idx_gbuff)-6,0)], ivec2(fract(uvn)*R));
    vec4 albedo = tex(shared_textures[max(int(PC.idx_galbedo)-1,0)], fract(uvn));

    vec4 norm = tex(shared_textures[max(int(PC.idx_gnorm)-1,0)], fract(uvn));

    vec4 vid = tex(shared_textures[max(int(PC.idx_vid)-1,0)], fract(uvn));

    
    
    // oC = abs(albedo);
    // oC = abs(norm);
    oC = abs(pow(vid,vec4(1./0.4545)));



    // tonemap
    
    oC = oC/(oC*1. + 1.4);
    oC *= 1.5;
    oC = mix(oC, smoothstep(0.,1.,oC),0.5);


    // gamma


    oC = max(pow(oC,vec4(0.454545)),.0);
    oC.w = 1.;
    
    // oC = tex(shared_textures[4], fract(uvn*rot(0.)/1.));
  


    // oC = vec4(vUv.xyx + sin(float(frame)), 1.0);
}
