#include "utils.include"
#include "chroma.include"

layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 C;

W_PC_DEF{
  UboObject ubo;
  u16 idx_composite;
  u16 idx_depth;
  u16 idx_far;
  u16 idx_near;
  u8 seg;
}



const float GOLDEN_ANGLE = 2.39996323;


float sides;

const bool debug_distribution = false;
vec2 UnitSquareToUnitDiskPolar(float a, float b) {
    float radius, angle;
    if (abs(a) > abs(b)) { // First region (left and right quadrants of the disk)
        radius = a;
        angle = b / (a + 0.000001) * pi/4.;
    } else { // Second region (top and botom quadrants of the disk)
        radius = b;
        angle = pi/2. - (a / (b + 0.000001) * pi/4.);
    }
    if (radius < 0.) { // Always keep radius positive
        radius *= -1.0f;
        angle += pi;
    }
    return vec2(radius, angle);
}
// https://ia800902.us.archive.org/25/items/crytek_presentations/Sousa_Graphics_Gems_CryENGINE3.pdf
// https://www.adriancourreges.com/blog/2018/12/02/ue4-optimized-post-effects/
// vec3 getCrytek(vec2 uv){
//     vec3 col = vec3(0);
    
//     for(float i = 0.; i < sample_cnt; i++){
//         float x = r11()*2. - 1.;
//         float y = r11()*2. - 1.;

//         vec2 p = UnitSquareToUnitDiskPolar(x, y);
        
//         float rad_sc = cos(pi / sides) 
//             / cos(p.y - (2.0 * pi / sides) * floor((sides*p.y + pi) / 2.0 / pi ) );

//         float rad = mix(1., rad_sc, env);
//         p.x *= rad;

//         p = vec2(p.x*cos(p.y), p.x*sin(p.y)); 


            
//         if(debug_distribution)
//             col = mix(col,vec3(1),smoothstep(fwidth(uv.y),0.,length(uv - p) - 0.003));
//         else
//             col += texture(iChannel0,uv + p*size*vec2(iResolution.y/iResolution.x,1.)).xyz;
//     }
//     //}
//     return col/sample_cnt;
// }

vec2 getCrytek(float x, float y){
    x = x*2. - 1.;
    y = y*2. - 1.;

    vec2 p = UnitSquareToUnitDiskPolar(x, y);
    
    float rad_sc = cos(pi / sides) 
        / cos(p.y - (2.0 * pi / sides) * floor((sides*p.y + pi) / 2.0 / pi ) );

    float rad = mix(1., rad_sc, 0.5);
    p.x *= rad;

    p = vec2(p.x*cos(p.y), p.x*sin(p.y)); 

    return p;
}

const float focus_d = 0.9   ;

// const float max_rad = 1.4;
// const float rad_sc = 0.04;

#define getCoc(d) min(abs(d - focus_d)*coc_rad,max_cock)


const float coc_rad = 14.;
const float max_cock = 50.02;


void main() {
    const vec2 uv = (vUv+ 1.)/2.;
    
    vec4 orig_col = tex_(int(PC.idx_composite),uv);

    float depth = tex_(int(PC.idx_depth),uv).x;
    depth = linearDepth(depth, zNear, zFar);

    float pixel_coc = getCoc(depth);
    
    C = vec4(0);

    
    
    if(PC.seg == 0 || PC.seg == 1){
        vec3 col = vec3(0);
        col = orig_col.xyz;

        bool is_near_seg = PC.seg == 1;
        bool is_far_seg = PC.seg == 0;

        float tot = 0.;


#define RING_COUNT	 5
#define RING_SPACING (1./float(RING_COUNT))
#define RING_DENSITY 10.0



        vec3 bin_accum[RING_COUNT];
        float bin_dens[RING_COUNT];
        int bin_sample_cnt[RING_COUNT];
        float bin_radii[RING_COUNT];

        int bin = 0;
        
        for(int it = 0; it < RING_COUNT; it++){
            bin_accum[it] = vec3(0);
            bin_dens[it] = 0.;
            bin_sample_cnt[it] = int(0);
            bin_radii[it] = 0.;
        }

        const vec2 aspect = vec2(R.y/R.x,1.);
        if(true){
            col = vec3(0);
			
			// C = orig_col;
			if(
				is_far_seg && depth < focus_d ||
				is_near_seg && depth > focus_d
				){
				C *= 0.;
			}

            
            
            for (float i = RING_COUNT - 1.0; i >= 0.0; --i) {
				float li = i;
				if(is_near_seg){
					li = RING_COUNT - 1.0 - i;
				}
                float rad = (RING_SPACING) * li; float pol_cnt = max(RING_DENSITY * li, 1.0); float inc = 2.0 * pi / pol_cnt; float off = 0.1 * li; // polar offset

                float kern_coc = rad*pixel_coc; 

                bin_radii[int(i)] = kern_coc;
            
                // float radPx = kern_len * R.y;
            
                // float areaCirc = pi * radPx * radPx;
                // areaCirc = max(areaCirc,1.);
                // float areaPix = 1.;
            
                // //float alpha = areaPix/areaCirc*R.y/pi/3.14;
                // float alpha = areaPix/areaCirc;

                for (float j = 0.0; j < pol_cnt; ++j) {
                    float theta = off + j * inc;


                    vec2 kern = vec2(cos(theta), sin(theta));
                    vec2 kern_p = kern_coc * kern;
                    
                    vec2 suv = uv + kern_p*aspect; suv = clamp(suv,1./R.y,1. - 1./R.y);

                    float sample_depth = linearDepth(tex_(PC.idx_depth, suv).x, zNear, zFar);
                    float sample_coc = getCoc(sample_depth);

                    if (
                        (is_far_seg && sample_depth < focus_d) ||
                        (is_near_seg && sample_depth > focus_d)
                        // || sample_depth > zFar - 2.
                        ) {
                    } else {
                        vec3 samp = tex_(PC.idx_composite, suv).xyz;

                        
                        // binned 
                        if(true){
                            // if(is_near_seg){
                            //     // weight = sample_coc > pixel_coc ? 1. : pixel_coc;
                            //     weight = sample_coc > max_cock ? 1. : pixel_coc;
                            // } else {
                                // Far seg
							if(sample_coc >= kern_coc){
								int bin = int(sample_coc/pixel_coc*(float(RING_COUNT)*0.999));
								bin_accum[bin] += samp;
								// bin_dens[bin] += alpha;
								bin_sample_cnt[bin] += 1;
								// weight = sample_coc <= pixel_coc ? 1. : pixel_coc;
							}
                            // }
                        }

                        // ferris
                        if(false){
                            float weight;
                            if(is_near_seg){
                                // weight = sample_coc > pixel_coc ? 1. : pixel_coc;
                                weight = sample_coc > max_cock ? 1. : pixel_coc;
                            } else {
                                weight = sample_coc <= pixel_coc ? 1. : pixel_coc;
                            }
                            col += samp * weight;
                            tot += weight;
                        }
                        // MJP
                        if(false){
                            samp *= clamp(1. - (sample_coc - pixel_coc),0.,1.);
                            
                            // float shapeCurveAmt = 0.5;
                            // samp *= (1. - );
                            col += samp;
                            
                            tot += 1.;
                        }
                    }
                    // sample_cnt_total += 1.;
                }

                bin++;

            }

            // C.xyz = orig_col.xyz;
            // for(int it = 0; it < RING_COUNT; it++){
            for(int it = RING_COUNT - 1; it >= 0; it--){
				float inv_it = float(RING_COUNT - 1 - it);

                float sample_cnt = float(bin_sample_cnt[it]);
                vec3 acc = bin_accum[it];
                float dens = bin_dens[it];
                if(sample_cnt > 0.){
                // if(true){
					// float sample_coc = (RING_SPACING)*float(inv_it)*max_cock;
					float sample_coc = (RING_SPACING)*float(it)*pixel_coc;

					float radPx = sample_coc * R.y;
                    radPx = max(radPx,1.);
				
					// float areaCirc = pi * radPx * radPx*0.5 * 0.5;
					float areaCirc = 1.;
                    if(it == 0.){
                        areaCirc = 1.;
                    }
					// areaCirc = max(areaCirc,1.);
					float areaPix = 1.;
					float alpha = areaPix/areaCirc;

                    // C.xyz += acc/sample_cnt*alpha;
                    C.xyz = mix(C.xyz,acc/sample_cnt,alpha);

                    // C.xyz = mix(C.xyz, acc/dens, dens/sample_cnt*(sample_cnt/sample_cnt_total));
                    // C.xyz += acc/sample_cnt*alpha;
					// if(it == 0){
					// 	C.xyz
						
					// }
                    // C.xyz = mix(C.xyz,acc/sample_cnt,dens*1.);
                    // C.xyz = mix(C.xyz, acc/dens, dens/sample_cnt_total);
                }

                // bin_sample_cnt[it] = int(cnt);

                
            }
            // if(tot > 0.){
            //     C.xyz = col/tot;
            //     // C.xyz = col/float(it);
            // } else {
            //     C.xyz = col;
            // }
            
        }
            
        // C.xyz = col/(tot);


            // float x = r11()*2. - 1.;
            // float y = r11()*2. - 1.;
            // float x = mod(i,sample_cnt)/sample_cnt;
            // float y = floor(i/sample_cnt)/sample_cnt;
            // vec2 p = getCrytek(x, y);

    } else if(PC.seg >= 2){
        #define TEARDOWN_DOF true
        if (TEARDOWN_DOF){

            if(PC.seg == 2){
                float uFocusPoint = 2.;
                float uFocusScale = 0.5;
                float uMaxBlurSize = 20.0;
                float uRadScale = 3.2;
                    
                float centerDepth = tex_(int(PC.idx_depth),uv).x;
                centerDepth = linearDepth(depth, zNear, zFar);

                float centerSize = getCoc(depth);

                // vec3 color = uRunningOnFB ? texture(s_InputFB[0], uv).rgb : texture(s_InputColTex, uv).rgb;
                vec3 color = tex_(int(PC.idx_composite),uv).xyz;

                float tot = 1.0;

                float radius = uRadScale;
                for (float ang = 0.0; radius<uMaxBlurSize; ang += GOLDEN_ANGLE) {
                    vec2 tc = uv + vec2(cos(ang), sin(ang)) * 1./max(R.x,R.y) * radius * vec2(1.,R.x/R.y);

                    // vec3 sampleColor = uRunningOnFB ? texture(s_InputFB[0], tc).rgb : texture(s_InputColTex, tc).rgb;
                    // vec3 sampleColor = texture(s_InputFB[0], tc).rgb : texture(s_InputColTex, tc).rgb;
                    // float sampleDepth = getLinearDepth(tc) ;
                    // float sampleSize = getBlurSize(sampleDepth, focusPoint, focusScale);

                    vec3 sampleColor = tex_(PC.idx_composite, tc).xyz;
                    float sampleDepth = linearDepth(tex_(PC.idx_depth, tc).x, zNear, zFar);
                    float sampleSize = getCoc(sampleDepth);

                    if (sampleDepth > centerDepth)
                        sampleSize = clamp(sampleSize, 0.0, centerSize*2.0);

                    float m = smoothstep(radius-0.5, radius+0.5, sampleSize);
                    color += mix(color/tot, sampleColor, m);
                    tot += 1.0;
                    radius += uRadScale/radius;
                }
                C.xyz = color /= tot;
            } else {
                vec3 color = texFetch_(int(PC.idx_far),uv*R,0).xyz;
                float kern_sz = 4.;
                // for 

                color = tex_(int(PC.idx_far),uv).xyz;
                C.xyz = color;
            }

        } else {
             C *= 0.;
            // C += tex_(int(PC.idx_near),uv);
            C += tex_(int(PC.idx_far),uv);
            // C = vec4(1,0,0,0);
            // if(int(PC.idx_far) > 2000)
            // C = tex_(31,uv);
            // C.x += 1.;
            // C = vec4(depth*0.01);
            C.w = 1.;
        }
       // C = tex_(int(PC.idx_depth),uv);
    }

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