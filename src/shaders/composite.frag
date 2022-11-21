

layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 oC;

W_PC_DEF{
  UboObject ubo;
  uint8_t idx_galbedo;
  uint8_t idx_gnorm;
  uint8_t idx_gvel;
  uint8_t idx_depth;
  uint8_t idx_prev_frame;
  uint8_t idx_flame_tex;
}

#include "utils.include"



#define saturate(x) clamp(x,0.,1.) 

vec4 hash;

// 4 out, 1 in...
vec4 hash41(float p) {
    vec4 p4 = fract(vec4(p) * vec4(.1031, .1030, .0973, .1099));
    p4 += dot(p4, p4.wzxy+33.33);
    return fract((p4.xxyz+p4.yzzw)*p4.zywx);
}

vec3 randomDirection( ) {
    float phi16_ = tau * hash.y;
    float theta = acos( -1.0 + 2.0 * hash.z );

    return vec3(
        sin( theta ) * sin( phi16_ ),
        cos( theta ),
        sin( theta ) * cos( phi16_ )
    );
}

vec3 lambertNoTangent(in vec3 normal, in vec2 uv)
{
    float theta = 6.283185 * uv.x;

    uv.y = 2.0 * uv.y - 1.0;
    vec3 spherePoint = vec3(sqrt(1.0 - uv.y * uv.y) * vec2(cos(theta), sin(theta)), uv.y);
    return normalize(normal + spherePoint);
}


vec4 worldToView(vec3 world){
    vec4 aoP = vec4(world.xyz, 1);
    aoP = V * aoP;
    aoP = P * aoP;
    aoP.xyz /= aoP.w;
    aoP.xy += 1.;
    aoP.xy /= 2.;
    return aoP;
}



// 4 out, 4 in...
vec4 hash44(vec4 p4)
{
	p4 = fract(p4  * vec4(.1031, .1030, .0973, .1099));
    p4 += dot(p4, p4.wzxy+33.33);
    return fract((p4.xxyz+p4.yzzw)*p4.zywx);
}

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

vec3 rgb_to_ycocg(vec3 c) {
	return vec3(
		 .25 * c.r + .5 * c.g + .25 * c.b,
		 .5  * c.r            - .5  * c.b,
		-.25 * c.r + .5 * c.g - .25 * c.b
	);
}

// convert from YCoCg to RGB
vec3 ycocg_to_rgb(vec3 c) {
	// float tmp = c.x - c.z;	// tmp = Y   - Cg;
	// return vec3(
	// 	tmp + c.y,	// R   = tmp + Co;
	// 	c.x + c.z,	// G   = Y   + Cg;
	// 	tmp - c.y	// B   = tmp - Co;
	// );
    	return saturate(vec3(
			c.x + c.y - c.z,
			c.x + c.z,
			c.x - c.y - c.z
		));
}

vec4 clip_aabb(vec3 aabb_min, vec3 aabb_max, vec4 p, vec4 q)
{
#define FLT_EPS 0.001
#if USE_OPTIMIZATIONS
    // note: only clips towards aabb center (but fast!)
    vec3 p_clip = 0.5 * (aabb_max + aabb_min);
    vec3 e_clip = 0.5 * (aabb_max - aabb_min) + FLT_EPS;

    vec4 v_clip = q - float4(p_clip, p.w);
    vec3 v_unit = v_clip.xyz / e_clip;
    vec3 a_unit = abs(v_unit);
    float ma_unit = max(a_unit.x, max(a_unit.y, a_unit.z));

    if (ma_unit > 1.0)
        return float4(p_clip, p.w) + v_clip / ma_unit;
    else
        return q;// point inside aabb
#else
    vec4 r = q - p;
    vec3 rmax = aabb_max - p.xyz;
    vec3 rmin = aabb_min - p.xyz;

    const float eps = FLT_EPS;

    if (r.x > rmax.x + eps)
        r *= (rmax.x / r.x);
    if (r.y > rmax.y + eps)
        r *= (rmax.y / r.y);
    if (r.z > rmax.z + eps)
        r *= (rmax.z / r.z);

    if (r.x < rmin.x - eps)
        r *= (rmin.x / r.x);
    if (r.y < rmin.y - eps)
        r *= (rmin.y / r.y);
    if (r.z < rmin.z - eps)
        r *= (rmin.z / r.z);

    return p + r;
#endif
}

void main() {

    //!! ---------- BEGIN

    // oC = tex(shared_textures[int(PC.idx_gbuff)-1],fract(vUv.xy));
    vec2 uv = vUv.xy;
    uv += 1.;
    vec2 uvn = uv*0.5;

    oC *= 0.0;
    // oC = 

    // oC = tex_(int(PC.idx_flame_tex)-6, fract(uvn)).rgbw*vec4(1);
    oC = uvn.xyxy;
    // oC = vec4(1,0,0,1);
    float br = oC.r;
    // oC = 0.5 + 0.5 * sin(exp(-oC.r*1.5)*vec4(3,2,4,1.)*3. + .0 - dot(vUv,vUv)*0.);

    // oC *= 1.-br;
    // oC = 1.-oC;

    // oC = abs(oC)/(1.+oC*1.);
    // oC = pow(oC,1./vec4(0.4545));
    // oC = smoothstep(0.,1.,oC);
    return;

    // oC = tex_(PC.idx_vid,fract(uvn));
    // oC = pow(oC,vec4(1./0.4545/0.4545));
    // return;
    // uv *= rot(T);
    // oC = imageLoad(shared_images[max(int(PC.idx_gbuff)-6,0)], ivec2(fract(uvn)*R));
    vec4 albedo = tex_(PC.idx_galbedo, fract(uvn));
    vec4 norm = tex_(PC.idx_gnorm, fract(uvn));
    float depth = tex_(PC.idx_depth, fract(uvn)).x;
    
    
    vec3 worldP = depthToWorld(depth, uvn, invV, invP);

    vec3 WorldN = (norm.xyz-0.5)*2.;
    WorldN = normalize(WorldN);
    
    oC = vec4(albedo);

    // oC = mix(oC, vec4(1),1.);

    //!! ---------- AO

    float uIters = 110.;
    float uAoRad = 0.2;

  
    float ao = 0.;
    hash = hash41( 20. + hash41( vUv.x*15.2 ).x*20.+ hash41( vUv.y*15.2 ).x*20.  );

    depth = linearDepth(depth, zNear, zFar);

    if(depth > zFar - 0.1){
        oC = vec4(0,0,0,0);

    };

    if(dot(albedo, albedo) > 0.00){
        for(float i = 0; i < uIters; i++){
            hash = hash41( 20. + i*15.56 + hash[int(i )%4]*14.5642 + float(frame%16)*0.);
            vec3 rayDir = lambertNoTangent(WorldN, hash.xy);
            // rayDir = mix(rayDir, WorldN,0.6);
            // if(dot(rayDir, WorldN)<0.001){
            //   rayDir *= -1.;
            // }

            vec4 rayNdcPos = PV * vec4( worldP + rayDir*uAoRad, 1. );
            rayNdcPos.xyz /= rayNdcPos.w;
            vec2 rayUvPos = rayNdcPos.xy * 0.5 + 0.5;

            rayNdcPos.z = linearDepth(rayNdcPos.z,zNear,zFar);

            float sampleDepth =tex(shared_textures[int(PC.idx_depth)-1], rayUvPos).x; 
            sampleDepth = linearDepth(sampleDepth,zNear,zFar);


            // float jumpCheck = smoothstep(1.,0.,abs(sampleDepth - rayNdcPos.z));
            float jumpCheck = smoothstep(1.5*uAoRad,1.5*uAoRad,abs(sampleDepth - rayNdcPos.z));

            float occ = 0.;
            if (sampleDepth < rayNdcPos.z + 0.1*uAoRad){
              occ += 1.;
            }
            occ = mix(occ,0.,jumpCheck);
            ao += occ;
        }

        ao /= uIters;
        ao = pow(ao,1.);
        ao = smoothstep( 0.,1., ao);
        

        ao = clamp(ao,0.,1.);
        oC *= 1.-ao;
    } 

    //!! ---------- SSR
    float ssrSteps = 0.;
    float ssrRange = 10.;
    float stepSz = ssrRange/ssrSteps;
    {
        vec3 p = worldP + WorldN*stepSz*5.; 

        vec3 rayDir = normalize(worldP - camPos*vec3(-1,1,1));

        rayDir = reflect(rayDir, WorldN);

        vec3 rayStep = rayDir*ssrRange/ssrSteps;
        
        for(float i = 0.; i < ssrSteps; i++){
            p += rayStep;

            vec4 rayNdcPos = PV * vec4( p, 1. );
            rayNdcPos.xyz /= rayNdcPos.w;
            rayNdcPos.z = linearDepth(rayNdcPos.z,zNear,zFar);
            vec2 rayUvPos = rayNdcPos.xy * 0.5 + 0.5;
            
            float sampleDepth = tex(shared_textures[int(PC.idx_depth)-1], rayUvPos).x; 

            sampleDepth = linearDepth(sampleDepth,zNear,zFar);


            if (sampleDepth < rayNdcPos.z + stepSz*2.){
                float fadeFac = smoothstep(0.,stepSz*8.,abs(sampleDepth - rayNdcPos.z));
                float fadeRange = smoothstep(0.,ssrSteps,i/ssrSteps);
                
                float fadeFacScreen = smoothstep(1.,0.,max(
                    abs(rayNdcPos.x * 0.5),
                    abs(rayNdcPos.y * 0.5)
                ));
                // fadeFacScreen = 1.;

                if(fadeFac<1.){
                    vec3 sampleAlbedo = tex(shared_textures[int(PC.idx_galbedo)-1], rayUvPos).xyz; 
                    oC.xyz = mix(
                        oC.xyz,
                        oC.xyz*sampleAlbedo.r,
                        fadeFacScreen*(1.-fadeFac)*(1.-fadeRange)
                    );
                }
                // oC *= 0.;
                break;
            //   occ += 1.;
            }
            
            
            
        }
        // rayDir = mix(rayDir, WorldN,0.6);
        // if(dot(rayDir, WorldN)<0.001){
        //   rayDir *= -1.;
        // }

        // vec4 rayNdcPos = PV * vec4( worldP + rayDir*uAoRad, 1. );
        // rayNdcPos.xyz /= rayNdcPos.w;
        // vec2 rayUvPos = rayNdcPos.xy * 0.5 + 0.5;

        // rayNdcPos.z = linearDepth(rayNdcPos.z,zNear,zFar);

        // float sampleDepth =tex(shared_textures[int(PC.idx_depth)-1], rayUvPos).x; // is this right?
        // sampleDepth = linearDepth(sampleDepth,zNear,zFar);


        // // float jumpCheck = smoothstep(1.,0.,abs(sampleDepth - rayNdcPos.z));
        // float jumpCheck = smoothstep(1.5*uAoRad,1.5*uAoRad,abs(sampleDepth - rayNdcPos.z));

        // float occ = 0.;
        // if (sampleDepth < rayNdcPos.z + 0.1*uAoRad){
        //   occ += 1.;
        // }
        // occ = mix(occ,0.,jumpCheck);
        // ao += occ;
        
    } 

    //!! ---------- POST 

    // tonemap
    oC = oC/(oC*1. + 1.4);
    oC *= 1.5;
    // oC *= 1.5;
    oC = mix(oC, smoothstep(0.,1.,oC),0.5);
    oC = mix(oC, smoothstep(0.,1.,oC),0.2);
    oC = mix(oC, smoothstep(0.,1.,oC),0.2);
    oC = mix(oC, smoothstep(0.,1.,oC),0.2);
    oC = mix(oC, smoothstep(0.,1.,oC),0.2);
    oC = saturate(oC);


    //!! ---------- TAA
    if(depth < zFar - 0.02) {
        mat4 proj = Pprev;

        vec2 h = HammersleyNorm(int(frame)%16, 16);
        // uvn -= h/R*1.;
        

        vec4 rayNdcPos =  PVprev * vec4( worldP, 1. );
        rayNdcPos.xyz /= rayNdcPos.w;
        vec2 rayUvPos = rayNdcPos.xy * 0.5 + 0.5;
        // rayUvPos = saturate(rayUvPos);

        vec2 luv = rayUvPos;
        
        if(
            any( lessThan(luv, vec2(0))) || 
            any( greaterThan(luv, vec2(1)))
        ){
            // outside of frustum
        } else {
            float neigh_sz = 1.;
            vec2 buv = uvn;

            vec4 prev_frame = tex_(PC.idx_prev_frame, luv);

            // vec3 center = tex_(PC.idx_prev_frame, buv).xyz;
            vec3 center = oC.xyz;
            
            vec3 ne = tex_(PC.idx_prev_frame, buv + neigh_sz*vec2(1,1)/R).xyz;
            vec3 sw = tex_(PC.idx_prev_frame, buv - neigh_sz*vec2(1,1)/R).xyz;
            vec3 ns = tex_(PC.idx_prev_frame, buv + neigh_sz*vec2(-1,1)/R).xyz;
            vec3 se = tex_(PC.idx_prev_frame, buv + neigh_sz*vec2(1,-1)/R).xyz;
            
            vec3 e = tex_(PC.idx_prev_frame, buv + neigh_sz*vec2(1,0)/R).xyz;
            vec3 w = tex_(PC.idx_prev_frame, buv - neigh_sz*vec2(1,0)/R).xyz;
            vec3 n = tex_(PC.idx_prev_frame, buv + neigh_sz*vec2(0,1)/R).xyz;
            vec3 s = tex_(PC.idx_prev_frame, buv - neigh_sz*vec2(0,1)/R).xyz;
            
            prev_frame.xyz = rgb_to_ycocg(prev_frame.xyz);
            center = rgb_to_ycocg(center);
            ne = rgb_to_ycocg(ne);
            sw = rgb_to_ycocg(sw);
            ns = rgb_to_ycocg(ns);
            se = rgb_to_ycocg(se);
            e = rgb_to_ycocg(e);
            w = rgb_to_ycocg(w);
            n = rgb_to_ycocg(n);
            s = rgb_to_ycocg(s);

            vec3 boxMin = min( ne, min( sw, min(se, ns)));
            vec3 boxMax = max( ne, max( sw, max(se, ns)));
            boxMin = min(min(min(e,min(w,min(n,s))), center), boxMin);
            boxMax = max(max(max(e,max(w,max(n,s))), center), boxMax);
            
            vec3 avg = (ne + sw + ns + se + e + w + n + s + center)/9.;

            // filter
            if(true){
                vec3 cmin5 = min(e, min(s, min(w, min(n, center))));
                vec3 cmax5 = max(e, max(s, max(w, max(n, center))));
                vec3 cavg5 = (n + w + s + e + center) / 5.0;
                boxMin = 0.5 * (boxMin + cmin5);
                boxMax = 0.5 * (boxMax + cmax5);
                avg = 0.5 * (avg + cavg5);
            }
            
            // shrink chroma minmax
            if (true){
                vec2 chroma_extent = vec2(0.25 * 0.5 * (boxMax.r - boxMin.r));
                vec2 chroma_center = center.gb;
                boxMin.yz = chroma_center - chroma_extent;
                boxMax.yz = chroma_center + chroma_extent;
                avg.yz = chroma_center;
                
            }

            if(true){
                prev_frame.xyz = clip_aabb(
                    boxMin, boxMax, 
                    clamp(avg, boxMin, boxMax).xyzz, 
                    prev_frame.xyzz).xyz;
            }
            if (false){
                prev_frame.xyz = clamp(prev_frame.xyz, boxMin, boxMax);
            }

            // luma feedback
            float lum0 = center.r;
            float lum1 = prev_frame.r;

            float _FeedbackMin = 0.7;
            float _FeedbackMax = 0.96;
            
            float unbiased_diff = abs(lum0 - lum1) / max(lum0, max(lum1, 0.2));
            float unbiased_weight = 1.0 - unbiased_diff;
            float unbiased_weight_sqr = unbiased_weight * unbiased_weight;

            float k_feedback = mix(_FeedbackMin, _FeedbackMax, unbiased_weight_sqr);
            
            if(false){
                oC.xyz = mix(center.xyz,prev_frame.xyz,k_feedback);
                oC.xyz = ycocg_to_rgb(oC.xyz);
            }
            if(true){
                oC.xyz = mix(center.xyz,prev_frame.xyz,0.8);
                oC.xyz = ycocg_to_rgb(oC.xyz);
            }

            // oC.xyz = ycocg_to_rgb(oC.xyz);
            // prev_frame.xyz = ycocg_to_rgb(prev_frame.xyz);

            // oC.xyz = mix(oC.xyz,prev_frame.xyz,0.8);
            
            // oC.x = clamp(oC.x,0.,0.1);


        }
        // vec3 st = vec3(1,1,0)/R.xyx;
        



// vec3 NearColor1 = textureLodOffset(CurrentBuffer, UV, 0.0, ivec2(0, 1));
// vec3 NearColor2 = textureLodOffset(CurrentBuffer, UV, 0.0, ivec2(-1, 0));
// vec3 NearColor3 = textureLodOffset(CurrentBuffer, UV, 0.0, ivec2(0, -1));

// vec3 BoxMin = min(CurrentSubpixel, min(NearColor0, min(NearColor1, min(NearColor2, NearColor3))));
// vec3 BoxMax = max(CurrentSubpixel, max(NearColor0, max(NearColor1, max(NearColor2, NearColor3))));;

// History = clamp(History, BoxMin, BoxMax);

        // rayUvPos *= 2.;


        // oC = prev_frame;
    }

}
