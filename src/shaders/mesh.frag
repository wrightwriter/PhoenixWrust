
W_PC_DEF{
  UboObject ubo;
  // uint8_t indices_buff_idx;
  // uint8_t vertex_buff_idx;
  uint16_t indices_buff_idx;
  uint16_t vertex_buff_idx;
  uint16_t diffuse_tex_idx;
  uint16_t normal_tex_idx;
  uint16_t metallic_roughness_tex_idx;
  uint16_t occlusion_tex_idx;

  // BuffIndices indices;
  // BuffVerts verts;
}


layout(location = 0) in vec3 vColor;
layout(location = 1) in vec3 vNorm;
layout(location = 2) in vec2 vUv;
layout(location = 3) in vec2 vVel;
// layout(location = 4) flat in mat3 vTBN;
// layout(location = 4) in vec3 vT;
// layout(location = 5) in vec3 vB;
layout(location = 6) in vec3 vPos;


layout(location = 0) out vec4 oC;
layout(location = 1) out vec4 oGNorm;
layout(location = 2) out vec4 oGVel;

// layout(location = 2) out vec4 oGPotato;
mat3 cotangent_frame( vec3 N, vec3 p, vec2 uv )
{
    // get edge vectors of the pixel triangle
    vec3 dp1 = dFdx( p );
    vec3 dp2 = dFdy( p );
    vec2 duv1 = dFdx( uv );
    vec2 duv2 = dFdy( uv );


    // solve the linear system
    vec3 dp2perp = cross( dp2, N );
    vec3 dp1perp = cross( N, dp1 );
    vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
    vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;

    // construct a scale-invariant frame 
    float invmax = inversesqrt( max( dot(T,T), dot(B,B) ) );
    return mat3( T * invmax, B * invmax, N );
}

void main() {
    vec3 n = vNorm;
    vec2 uv = U.xy/R.xy;
    
    vec4 albedo = vec4(1.);
    if(PC.diffuse_tex_idx != 0)
        albedo = texMip(int(PC.diffuse_tex_idx), (vUv));
    // albedo = vec4(1);
    vec4 normal_map = vec4(0,0,1,0);

    if(PC.normal_tex_idx != 0)
        normal_map = texMip(int(PC.normal_tex_idx), (vUv));
    // albedo = normal_map;
    // albedo = normal_map;
    // vec4 occlusion_map = tex(shared_textures[int(PC.occlusion_tex_idx)-1], (vUv));
    // vec4 metallic_roughness_map = tex(shared_textures[int(PC.metallic_roughness_tex_idx)-1], (vUv));
    // vec4 albedo = vec4(1,0,0,0);
    
    if(abs(dot(sin(vPos*10.),sin(vPos*10.))) < 0.01){
        albedo = vec4(10);
    } else {
        albedo = vec4(0,0,0,1);
        albedo.xyz += 0.5;
        
    }

    oC = pow(abs(albedo),vec4(1./0.4545));

    // oC = vNorm.xyzx*0.5 + 0.5;
    // oC = albedo.xyzz;
    oC.w = 1.;
    
    vec3 norm = vNorm;

    mat3 TBN = cotangent_frame( n, vPos, uv);
    
if(
    false
    ){

    // norm = (normal_map.xyz*2. - 1.)*1.;
    norm = normal_map.xyz*2.-1.;
    norm = normalize(norm);
    norm *= transpose(TBN);
    // norm = normalize(norm);
    
    // oC = normal_map.xyzz;
}


    // norm = (normal_map.xyz*2. - 1.)*1.;
    // norm = normal_map.xyz;
    // norm *= transpose(TBN);

    // norm = normalize(norm);
    
    // norm = mix(norm);


// norm = normalize(norm);

    oGNorm = vec4(clamp(norm*0.5 + 0.5,0.,1.),1);
    
    oGVel = vVel.xyxy;
    oGVel.w = 1.;
}
