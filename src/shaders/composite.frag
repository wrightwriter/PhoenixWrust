
#include "utils.include"

layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 oC;

W_PC_DEF{
  UboObject ubo;
  uint8_t idx_galbedo;
  uint8_t idx_gnorm;
  uint8_t idx_depth;
  uint8_t idx_prev_frame;
  uint8_t idx_font;
}

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

void main() {

    //!! ---------- BEGIN

    // oC = tex(shared_textures[int(PC.idx_gbuff)-1],fract(vUv.xy));
    vec2 uv = vUv.xy;
    uv += 1.;
    vec2 uvn = uv*0.5;
    // uv *= rot(T);
    // oC = imageLoad(shared_images[max(int(PC.idx_gbuff)-6,0)], ivec2(fract(uvn)*R));
    vec4 albedo = tex_(PC.idx_galbedo, fract(uvn));
    vec4 norm = tex_(PC.idx_gnorm, fract(uvn));
    float depth = tex_(PC.idx_depth, fract(uvn)).x;
    

    if(tex_(PC.idx_depth, uvn).x == 1.){
        oC.xyz = albedo.xyz;
        oC.a = 1.;
        return;
    }
    
    vec3 worldP = depthToWorld(depth, uvn, invV, invP);

    vec3 WorldN = (norm.xyz-0.5)*2.;
    WorldN = normalize(WorldN);
    

    // WorldN.y *= -1.;
    
  
    // oC = vec4(norm);
    // oC = vec4(0);


    // oC += sin(depth*5.)*0.5 + 0.5;
    // C += sin(worldP.xyzz*20.)*0.5 + 0.5;
    // oC = vec4(1);
    oC = vec4(albedo);
    // oC = mix(oC, vec4(1),1.);

    //!! ---------- AO

    float uIters = 70.;
    float uAoRad = 0.2;

  
    float ao = 0.;
    hash = hash41( 20. + hash41( vUv.x*15.2 ).x*20.+ hash41( vUv.y*15.2 ).x*20.  );

    depth = linearDepth(depth, zNear, zFar);

    for(float i = 0; i < uIters; i++){
        hash = hash41( 20. + i*15.56 + hash[int(i)%4]*4.5642 );
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
    

    oC *= 1.-ao;
    // oC *= 0.02;

    //!! ---------- SSR
    float ssrSteps = 30.;
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
    oC = mix(oC, smoothstep(0.,1.,oC),0.5);


    // gamma


    oC = max(pow(oC,vec4(0.454545)),.0);
    oC.w = 1.;
    
    // oC = vec4(1,0,0,1);


    //!! ---------- TAA
    
    {
        vec4 r = hash44(vec4(vUv*200., mod(T,10000.),mod(T+ 2000.,10000.)));
        vec2 offs = vec2(sin(r.x*tau),cos(r.x*tau))*sqrt(r.y);

        vec2 luv = uvn + offs/min(R.x,R.y)*0.0;
        // vec4 rd = vec4(uvn,0,0);
        vec4 rayNdcPos = PV * vec4( worldP, 1. );
        rayNdcPos.xyz /= rayNdcPos.w;
        vec2 rayUvPos = rayNdcPos.xy * 0.5 + 0.5;
        // rayUvPos *= 2.;


        vec4 prev_frame = tex_(PC.idx_prev_frame, fract(rayUvPos));
        
        // oC = mix(oC,prev_frame,0.9);



    }
    // oC = tex_(PC.idx_font, fract(vUv));
    oC = norm;
    
    // vec3 flipped_texCoords = vec3(texCoords.x, 1.0 - texCoords.y, texCoords.z);
    // vec2 pos = flipped_texCoords.xy;
    // vec3 sample = texture(msdf, flipped_texCoords).rgb;
    ivec2 sz = texSz(PC.idx_font).xy;
    float dx = dFdx(vUv.x) * sz.x; 
    float dy = dFdy(vUv.y) * sz.y;
    float toPixels = 8.0 * inversesqrt(dx * dx + dy * dy);
    float sigDist = median(oC.r, oC.g, oC.b);
    float w = fwidth(sigDist);
    float opacity = smoothstep(0.5 - w, 0.5 + w, sigDist);    

    // oC = vec4(opacity);
    


    // oC = vec4(vUv.xyx + sin(float(frame)), 1.0);
}
