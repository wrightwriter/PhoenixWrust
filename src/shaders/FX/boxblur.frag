#include "utils.include"
#include "chroma.include"

layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 C;

W_PC_DEF{
  UboObject ubo;
  u16 idx_input_tex;
  u16 idx_orig_tex;
  u8 iter;
  u8 iter_cnt;
  u8 seg;
}

float uThreshold = 0.2;
float uRamp = 1.5;
float uAmount = .0;


bool uIsDownsampling = false;
bool uIsUpsampling = false;

bool isThresholded = false;

vec4 get(u16 t_idx, vec2 uv){
    vec2 pxSz = 0.5/texSz(t_idx);
    uv = clamp(
        uv,
        vec2(0. + pxSz.x,0. + pxSz.y),
        vec2(1. - pxSz.x,1. - pxSz.y)
    );
    if (!isThresholded)
        return tex_(t_idx,uv);
    else {
        vec4 t = tex_(t_idx,uv);
        return t * smoothstep(uThreshold, uThreshold + uRamp,luma(t));
    }
}

vec4 blur(vec2 uv, u16 t_idx, float stepSz){
    vec2 pxSz = 1./texSz(t_idx);
    vec2 step = (pxSz*stepSz);
    return (
        get(t_idx,uv + step*vec2(-1,1)) +
        get(t_idx,uv + step*vec2(1,1)) +
        get(t_idx,uv + step*vec2(1,-1)) +
        get(t_idx,uv + step*vec2(-1,-1))
    ) / 4.;
}

void main() {
    const vec2 uv = (vUv+ 1.)/2.;
    // C = uv.xyxy;
    C = tex_(int(PC.idx_input_tex),uv);
    // C = tex_(int(PC.idx_orig_tex),uv);
    C.w = 1.;
    // return;

    u8 uCurrIter = PC.iter;
    u8 uIterationCnt = PC.iter_cnt;
    u8 uSeg = PC.seg;
    
    if(uSeg == 0){
        uIsDownsampling = true;
    } else if(uSeg == 1){
        uIsUpsampling = true;
    } 
    if (uSeg == 2){
        // Final step.
        C = tex_(PC.idx_orig_tex, uv)+ blur(uv, PC.idx_input_tex, 0.5)*uAmount;
    } else if (uIsDownsampling){
        // Downsample step.
        if (uCurrIter == 0)
            isThresholded = true;
        C = blur(uv,PC.idx_input_tex,1.);
    } else if (uIsUpsampling){
        // Upsampling step.
        C = blur(uv,PC.idx_input_tex, 0.5);
    }
    C.a = 1.;
}