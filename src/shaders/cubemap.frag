




W_PC_DEF{
  UboObject ubo;
  uint16_t ibl_idx;
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


void main() {
  vec2 uvn = vUv.xy*0.5 + 0.5;
  C = vec4(vUv.xy,0,1);
  {
    vec2 uv = SampleSphericalMap(normalize(vCpos)); // make sure to normalize localPos
    // C = tex_(int(PC.ibl_idx)+1,uv);
    C = tex_(int(PC.ibl_idx)+1,uv);
    
  }
  if(vCuv.x == 0.){
    C = vec4(1,0,0,0);
  }
  C.w = 1.;
}

