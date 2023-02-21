


#include "utils.include"



struct Vertex {
  vec3 position;
  vec3 normal;
  vec3 tangent;
  vec4 color;
  vec2 uv;
  // vec2 uvsb;
};



// Wa_BDA_DEF BuffVerts{
//   Vertex data[];
// };

// Wa_BDA_DEF BuffIndices{
//   uint data[];
// };



W_PC_DEF{
  UboObject ubo;
  u16 idx_composite;
  u16 idx_depth;
}

layout(location = 0) out vec2 vUv;
layout(location = 1) out float vAlpha;
layout(location = 2) out vec3 vCol;
// layout(location = 2) out float alpha;

double focus_d = 2.;
const double coc_rad = 0.02;
const double max_cock = 0.04;


#define getCoc(d) min(abs(d - focus_d)*coc_rad,max_cock)




// Triangle strip
vec2 positions[4] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0,1.0)
);

void main() {
  vec2 sq = positions[gl_VertexIndex];
  
  focus_d += sin(T*1.)*0.4;
  
  vec2 pos = vec2(
    gl_InstanceIndex % int(R.x),
    gl_InstanceIndex / int(R.x)
  );

  // vec2 uv = (pos+ 0.5)/R;

  // vCol = tex_(PC.idx_composite, uv).xyz;
  vCol = texFetch_(PC.idx_composite, pos,0).xyz;
// texFetch_(t,l,lod)

  double depth = texFetch_(PC.idx_depth,pos,0).x;
  depth = linearDepth(depth, zNear, zFar);


  double coc_screen_sz = getCoc(depth);

  // float pixel_coc = float(max(coc_screen_sz*double(R.y),1.33));
  // pixel_coc = mix(1.33,pixel_coc,smoothstep(0.,1.,pixel_coc));
  

  float pixel_coc = float(max(coc_screen_sz*double(R.y),1.0));

  float coc_area = pi * pixel_coc * pixel_coc*0.5*0.5;
  

  if(depth > zFar - 0.01){
    pixel_coc = 20.;
  }

  // float coc_area = pixel_coc * pixel_coc;

coc_area = mix(pi,coc_area,smoothstep(0.,2.,pixel_coc ));
  // if(pixel_coc < 3.){
  // coc_area = 2.;
    
  // }

  vAlpha = 1./coc_area;
  

  
  // pixel_coc = 1.;
  
    
  

  // coc_area = mix(1.,coc_area,smoothstep(1.5,2.5,pixel_coc));

  // if(vAlpha > pi/3. + 0.1){
  //   vAlpha = 1.;
      
  // }
  
  vUv = sq;
  // sq *= vec2(R.y/R.x*1.,1.);
  // sq *= vec2(1.,R.y/R.x*1.5);


  sq *= 1.;
  // pos += mix(
  //   // sq*0.51,
  //   sq*0.5001,
  //   sq*0.5001,
  //   smoothstep(1.,2.,pixel_coc)
  // ) *pixel_coc*1. + 0.5;
  // pos += sq*0.500*pixel_coc + 0.;
  pos += sq*0.5*pixel_coc*3. + 0.;

  pos /= R; // 0 to 1
  pos = pos*2. - 1.; // -1 to 1
  

  if(depth < focus_d){
    // pos += 100.;
  }
  // pos += 0.5/R;
  // pos += 0.5/R;

  // pos += (sq*0.5)*pixel_coc;

  

  
    
  gl_Position = vec4(vec3(pos,0.01), 1.0);
}
