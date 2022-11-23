#include "utils.include"

layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 C;

W_PC_DEF{
  UboObject ubo;
  uint16_t idx_a;
  // uint8_t idx_gnorm;
  // uint8_t idx_depth;
  // uint8_t idx_prev_frame;
}


// https://www.shadertoy.com/view/wt2GDW

void main() {
    vec2 uVar = vUv;
    vec2 uv = (uVar + 1.)/2.;
    
    
    vec4 t = tex_(int(PC.idx_a)+1, fract(uv));
    // vec4 t = tex_(3, fract(uv));
    
    C = t;
    C.w = 1.;
    

    C = max(pow(C,vec4(0.454545)),.0);
}
