

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
}

#include "utils.include"

vec4 hash;

// 4 out, 1 in...
vec4 hash41(float p) {
    vec4 p4 = fract(vec4(p) * vec4(.1031, .1030, .0973, .1099));
    p4 += dot(p4, p4.wzxy+33.33);
    return fract((p4.xxyz+p4.yzzw)*p4.zywx);
}


vec3 lambertNoTangent(in vec3 normal, in vec2 uv)
{
    float theta = 6.283185 * uv.x;

    uv.y = 2.0 * uv.y - 1.0;
    vec3 spherePoint = vec3(sqrt(1.0 - uv.y * uv.y) * vec2(cos(theta), sin(theta)), uv.y);
    return normalize(normal + spherePoint);
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

    
    const float ao_iters = 450.;
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
        oC.xyz = cube_map;
        oC.w = 1.;
        return;
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

    oC.w = 1.;

}
