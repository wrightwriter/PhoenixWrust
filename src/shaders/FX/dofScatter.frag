#include "utils.include"
#include "chroma.include"

layout(location = 0) in vec2 vUv;
layout(location = 1) flat in float vAlpha;
layout(location = 2) flat in vec3 vCol;

layout(location = 0) out vec4 C;

W_PC_DEF{
  UboObject ubo;
  u16 idx_composite;
  u16 idx_depth;
  u16 idx_far;
  u16 idx_near;
  u8 seg;
}


void main() {
    vec2 uv = vUv;
    // uv = clamp(uv,0.5/R.y,1. + 1./R.y);
    
    // uv *= smoothstep(1./R.x,0.,length)
    float sd = length(vUv);
    float alpha = vAlpha * smoothstep(fwidth(sd),0.,sd - 0.33)*1.;
    

    // float alpha = vAlpha * smoothstep(fwidth(sd),0.,sd - 1.);
    // alpha = vAlpha ;
    
    // alpha = 1.;
    


    // C = tex_(int(PC.idx_composite),uv);
    C.xyz = vCol;
    // C.w = 0.4;
    C.w = alpha;
    // if(vUv.x < 0.5/R.x){
    //     C = vec4(1,0,0,1);
    // }

    // C = fract(depth*10.)*vec4(1);
}



        // // if(depth > focus_d)
        // //     C = tex_(int(PC.idx_composite),uv);
        // // if(depth < focus_d)
        // //     C = tex_(int(PC.idx_composite),uv);
        // C.w = 1.;
        // C = vec4(0.001);
            
        // float tot = 1.;
        // float radius = rad_sc;
        // for (float ang = 0.0; radius<max_rad; ang += GOLDEN_ANGLE) {
        //     // vec2 tc = uv + vec2(cos(ang), sin(ang)) * 1./max(R.x,R.y) * radius * vec2(1.,R.x/R.y);
        //     float r = radius * vec2(1.,R.x/R.y);
        //     vec2 tc = uv + vec2(cos(ang), sin(ang)) * r;

        //     vec3 sampleColor = tex_(PC.idx_composite, tc).xyz;

        //     float sampleDepth = linearDepth(tex_(PC.idx_depth, tc).x, zNear, zFar);

        //     float sampleCoc = getCoc(sampleDepth);

        //     // if (sampleDepth > centerDepth)
        //     //     sampleSize = clamp(sampleSize, 0.0, centerSize*2.0);

        //     if (
        //         (PC.seg == 0 && sampleDepth > focus_d) ||
        //         (PC.seg == 1 && sampleDepth < focus_d)
        //         ) {
        //         C.xyz += C.xyz/tot;
        //     } else {
        //         if(radius < sampleCoc)
        //             C.xyz += sampleColor;
        //         else 
        //             C.xyz += C.xyz/tot;
        //     }

        //     // float m = smoothstep(radius-0.5, radius+0.5, sampleSize);
        //         // C.xyz += mix(C.xyz/tot, sampleColor, m);
        //     tot += 1.0;
        //     radius += rad_sc/radius;
        // }
        // C/=tot;
        // // if(tot < 5.)
        // //     C.xyz += uv.xyx;
        // C.w = 1.;