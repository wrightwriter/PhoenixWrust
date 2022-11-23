
W_PC_DEF{
  UboObject ubo;
  uint16_t indices_buff_idx;
  uint16_t vertex_buff_idx;
}


// layout(location = 0) in vec3 vColor;
// layout(location = 1) in vec3 vNorm;
// layout(location = 2) in vec2 vUv;

layout(location = 0) out vec4 oC;
layout(location = 1) out vec4 oGNorm;
// layout(location = 2) out vec4 oGPotato;

void main() {
    // vec3 n = vNorm;
    vec2 uv = U.xy/R.xy;
    
    vec4 albedo = vec4(1,0,0,1);

    oC.w = 1.;

    // oGNorm = vec4(clamp(vNorm.xyz*0.5 + 0.5,0.,1.),1);
    oGNorm = vec4(1,0,0,1);
}
