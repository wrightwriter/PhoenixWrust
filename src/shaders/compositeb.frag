

layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 oC;

W_PC_DEF{
  UboObject ubo;
  u16 idx_hdr;
  u16 idx_brdf;
  u16 idx_galbedo;
  u16 idx_gnorm;
  u16 idx_gvel;
  u16 idx_depth;
  u16 idx_prev_frame;
  u16 idx_flame_tex;
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

const float PI = 3.14159265359;
// ----------------------------------------------------------------------------
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}   

void main() {

    //!! ---------- BEGIN

    // oC = tex(shared_textures[int(PC.idx_gbuff)-1],fract(vUv.xy));
    vec2 uv = vUv.xy;
    uv += 1.;
    vec2 uvn = uv*0.5;
    
    
    float aspect = R.x/R.y;

    vec3 vDir = vec3(vUv.x*aspect, vUv.y*-1, -1.0)*mat3(V);
    // vec3 vForward = vec3(0,0,-1)*mat3(V);

    vec3 cube_map = texCubeLod(int(PC.idx_hdr), vDir, 0.).rgb;




    vec4 albedo = tex_(PC.idx_galbedo, fract(uvn));

    // albedo = vec4(1);
    
    vec4 norm = tex_(PC.idx_gnorm, fract(uvn));
    float depth = tex_(PC.idx_depth, fract(uvn)).x;
    
    
    vec3 worldP = depthToWorld(depth, uvn, invV, invP);
    vec3 WorldN = (norm.xyz-0.5)*2.; // WorldN = normalize(WorldN);

    
    const float ao_iters = 50.;
    const float uAoRad = 0.1;
  
    float ao = 0.;
    
    // oC = norm.xyzz;
    // return;
    
    
    
    hash = hash41( 20. + hash41( vUv.x*15.2 ).x*20.+ hash41( vUv.y*15.2 ).x*20.  );
    depth = linearDepth(depth, zNear, zFar);

    if(depth < zFar - 0.1) {
        //!! ---------- AO
        for(float i = 0; i < ao_iters; i++){
            hash = hash41( 20. + i*15.56 + hash[int(i )%4]*14.5642 + float(frame%16)*0.);
            vec3 rayDir = lambertNoTangent(WorldN, hash.xy);

            vec4 rayNdcPos = PV * vec4( worldP + rayDir*uAoRad, 1. );
            rayNdcPos.xyz /= rayNdcPos.w;
            vec2 rayUvPos = rayNdcPos.xy * 0.5 + 0.5;

            rayNdcPos.z = linearDepth(rayNdcPos.z,zNear,zFar);

            float sampleDepth =tex_(PC.idx_depth, rayUvPos).x; 
            sampleDepth = linearDepth(sampleDepth,zNear,zFar);


            // float jumpCheck = smoothstep(1.,0.,abs(sampleDepth - rayNdcPos.z));
            float jumpCheck = smoothstep(1.5*uAoRad,1.5*uAoRad,abs(sampleDepth - rayNdcPos.z));

            float occ = 0.;
            if (sampleDepth < rayNdcPos.z + 0.2*uAoRad){
              occ += 1.;
            }
            occ = mix(occ,0.,jumpCheck);
            ao += occ;
        }

        ao /= ao_iters;
        ao = pow(ao,1.);
        ao = smoothstep( 0.,1., ao);
        ao = 1. - ao;
        // ao = 1.;


        const float metallic = 0.;
        const float roughness = 1.;

        vec3 WorldPos = worldP;
        vec3 N = WorldN;
        // N.y *= -1.;
        vec3 V = normalize(camPos - WorldPos);
        V.x *= -1.;
        vec3 R = reflect(-V, N); 
        
        vec3 F0 = vec3(0.04); 
        F0 = mix(F0, albedo.xyz, metallic);
        
        
        // reflectance equation
        vec3 Lo = vec3(0.0);
        for(int i = 0; i < 1; ++i) 
        {
            vec3 lightPos = normalize(vec3(-1,1,-2))*5.;
            vec3 lightCol = vec3(1,1,1)*1.;

            // calculate per-light radiance
            vec3 L = normalize(lightPos - WorldPos);
            vec3 H = normalize(V + L);
            float distance = length(lightPos - WorldPos);
            float attenuation = 1.0 / (distance * distance);
            vec3 radiance = lightCol * attenuation;

            // Cook-Torrance BRDF
            float NDF = DistributionGGX(N, H, roughness);   
            float G   = GeometrySmith(N, V, L, roughness);    
            vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);        
            
            vec3 numerator    = NDF * G * F;
            float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // + 0.0001 to prevent divide by zero
            vec3 specular = numerator / denominator;
            
             // kS is equal to Fresnel
            vec3 kS = F;

            vec3 kD = vec3(1.0) - kS;
            // multiply kD by the inverse metalness such that only non-metals 
            // have diffuse lighting, or a linear blend if partly metal (pure metals
            // have no diffuse light).
            kD *= 1.0 - metallic;	                
                
            // scale light by NdotL
            float NdotL = max(dot(N, L), 0.0);        

            // add to outgoing radiance Lo
            Lo += (kD * albedo.xyz / PI + specular) * radiance * NdotL; // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
        }
        vec3 F = fresnelSchlickRoughness(max(dot(N, V), 0.0), F0, roughness);
        vec3 kS = F;
        vec3 kD = 1.0 - kS;
        kD *= 1.0 - metallic;	  
        
        // vec3 irradiance = texture(irradianceMap, N).rgb;
        vec3 irradiance = texCube_(int(PC.idx_hdr)-1, N).rgb;
        vec3 diffuse      = irradiance * albedo.xyz;
        
        // sample both the pre-filter map and the BRDF lut and combine them together as per the Split-Sum approximation to get the IBL specular part.
        const float MAX_REFLECTION_LOD = 6.0;
        vec3 prefilteredColor = texCubeLod(int(PC.idx_hdr), R,  roughness * MAX_REFLECTION_LOD).rgb;    
        vec2 brdf  = tex_(PC.idx_brdf, vec2(max(dot(N, V), 0.0), roughness)).rg;
        vec3 specular = prefilteredColor * (F * brdf.x + brdf.y);
        
        // kD *= 0.;
        // diffuse *= 0.;
        // float ao = 1.;
        vec3 ambient = (kD * diffuse + specular) * ao;
        
        vec3 color = ambient + Lo;
        
        
        oC.xyz = color;
    } else {

        oC = cube_map.xyzz;
        oC = texCube_(int(PC.idx_hdr), vDir).rgbb;
        
    }

    // if(depth > zFar - 0.1){
    //     oC = vec4(0,0,0,0);

    // };

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
            
            // asdg
            

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
    // if(depth < zFar - 0.02) {
    if(true) {
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
            if (true){
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
