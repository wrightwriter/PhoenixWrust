


#include "noises.include"
#include "raytrace.include"



W_PC_DEF{
  UboObject ubo;
  u16 ibl_idx;
  u8 stage;
  float roughness;
}

layout(location = 0) in vec2 vUv;
// layout(location = 1) in vec3 vCubeVec;
layout(location = 1) in vec2 vCuv;
layout(location = 2) in vec3 vCpos;

layout(location = 0) out vec4 C;


const vec2 invAtan = vec2(0.1591, 0.3183);
vec2 SampleSphericalMap(vec3 v)
{
    vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}


// vec3 getHemisphereUniformSample(vec3 n) {
//     float cosTheta = getRandom();
//     float sinTheta = sqrt(1. - cosTheta * cosTheta);

//     float phi = 2. * pi * getRandom();

//     // Spherical to cartesian
//     vec3 t = normalize(cross(n.yzx, n));
//     vec3 b = cross(n, t);

//     return (t * cos(phi) + b * sin(phi)) * sinTheta + n * cosTheta;
// }



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
// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
// efficient VanDerCorpus calculation.
float RadicalInverse_VdC(uint bits) 
{
     bits = (bits << 16u) | (bits >> 16u);
     bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
     bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
     bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
     bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
     return float(bits) * 2.3283064365386963e-10; // / 0x100000000
}
// ----------------------------------------------------------------------------
vec2 Hammersley(uint i, uint N)
{
	return vec2(float(i)/float(N), RadicalInverse_VdC(i));
}
// ----------------------------------------------------------------------------
vec3 ImportanceSampleGGX(vec2 Xi, vec3 N, float roughness)
{
	float a = roughness*roughness;
	
	float phi = 2.0 * PI * Xi.x;
	float cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));
	float sinTheta = sqrt(1.0 - cosTheta*cosTheta);
	
	// from spherical coordinates to cartesian coordinates - halfway vector
	vec3 H;
	H.x = cos(phi) * sinTheta;
	H.y = sin(phi) * sinTheta;
	H.z = cosTheta;
	
	// from tangent-space H vector to world-space sample vector
	vec3 up          = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
	vec3 tangent   = normalize(cross(up, N));
	vec3 bitangent = cross(N, tangent);
	
	vec3 sampleVec = tangent * H.x + bitangent * H.y + N * H.z;
	return normalize(sampleVec);
}



void main() {
  seed = 12512;

  vec2 uvn = vUv.xy*0.5 + 0.5;
  C = vec4(vUv.xy,0,1);

  if (int(PC.stage) == 0) {
    // Generate cubemap.
    vec2 uv = SampleSphericalMap(normalize(vCpos));
    C = tex_(int(PC.ibl_idx),uv);
  } else if (int(PC.stage) == 1) {
    // Generate blurred cubemap.
    C *= 0.;

    float sample_cnt = 4000.;

    vec3 dir = normalize(vCpos);
    
    vec3 normal = dir;
    
    vec3 up    = vec3(0.0, 1.0, 0.0);
    vec3 right = normalize(cross(up, normal));
    up         = normalize(cross(normal, right));

    float sampleDelta = 0.025;
    float nrSamples = 0.0; 

    for(float phi = 0.0; phi < 2.0 * pi; phi += sampleDelta)
    {
        for(float theta = 0.0; theta < 0.5 * pi; theta += sampleDelta)
        {
            // spherical to cartesian (in tangent space)
            vec3 tangentSample = vec3(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta));
            // tangent space to world
            vec3 sampleVec = tangentSample.x * right + tangentSample.y * up + tangentSample.z * normal;

            // irradiance += texture(environmentMap, sampleVec).rgb * cos(theta) * sin(theta);

            C += texCube_(int(PC.ibl_idx),sampleVec) * cos(theta) * sin(theta);
            nrSamples++;
        }
    }
    C = pi * C * (1.0 / float(nrSamples));
  } else if (int(PC.stage) == 2) {
    vec3 dir = normalize(vCpos);
    
    
    dir.y *= -1.;
    vec3 normal = dir;

    vec3 N = normal;
    
    // make the simplyfying assumption that V equals R equals the normal 
    vec3 R = N;
    vec3 V = R;

    const uint SAMPLE_COUNT = 1024u;
    vec3 prefilteredColor = vec3(0.0);
    float totalWeight = 0.0;
    
    for(uint i = 0u; i < SAMPLE_COUNT; ++i)
    {
        // generates a sample vector that's biased towards the preferred alignment direction (importance sampling).
        vec2 Xi = Hammersley(i, SAMPLE_COUNT);
        vec3 H = ImportanceSampleGGX(Xi, N, PC.roughness);
        vec3 L  = normalize(2.0 * dot(V, H) * H - V);

        float NdotL = max(dot(N, L), 0.0);
        if(NdotL > 0.0)
        {
            // sample from the environment's mip level based on roughness/pdf
            float D   = DistributionGGX(N, H, PC.roughness);
            float NdotH = max(dot(N, H), 0.0);
            float HdotV = max(dot(H, V), 0.0);
            float pdf = D * NdotH / (4.0 * HdotV) + 0.0001; 

            float resolution = 512.0; // resolution of source cubemap (per face)
            float saTexel  = 4.0 * PI / (6.0 * resolution * resolution);
            float saSample = 1.0 / (float(SAMPLE_COUNT) * pdf + 0.0001);

            float mipLevel = PC.roughness == 0.0 ? 0.0 : 0.5 * log2(saSample / saTexel); 
            
            // prefilteredColor += textureLod(environmentMap, L, mipLevel).rgb * NdotL;
            // prefilteredColor += textureLod(environmentMap, L, mipLevel).rgb * NdotL;
            // prefilteredColor += texCubeLod(PC.ibl_idx, L, mipLevel).rgb * NdotL;
            prefilteredColor += texCubeLod(PC.ibl_idx, L, mipLevel).rgb * NdotL;
            totalWeight      += NdotL;
        }
    }

    prefilteredColor = prefilteredColor / totalWeight;
    C = prefilteredColor.xyzz;
    
  }

  // if(vCuv.x == 0.){
  //   C = vec4(1,0,0,0);
  // }
  C.w = 1.;
}

