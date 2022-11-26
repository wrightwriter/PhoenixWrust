

W_PC_DEF{
  UboObject ubo;
}


// Triangle strip
vec2 positions[4] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0,1.0)
);


layout(location = 0) out vec2 vUv;
layout(location = 1) out vec2 vCuv;
layout(location = 2) out vec3 vCpos;

void main() {
    gl_Layer = int(gl_InstanceIndex);
    vec2 pos = positions[gl_VertexIndex];


    vec2 uvn = pos*0.5 + 0.5;
    
    
    int l = gl_InstanceIndex;
    vCuv = uvn;

    vCuv.x /= 4;
    
    vCpos = vec3(pos, -1);

    if(l == 0){
      // rightmost
      vCuv.x += 2./4.;
      vCpos.xz *= rot(tau*2./4.);
    } else if(l == 1){
      // second 
      vCuv.x += 3./4.;
    } else if(l == 4){
      // first
      vCpos.xz *= rot(tau*1./4.);
    } else if(l == 5){
      // third
      vCuv.x += 1./4.;
      vCpos.xz *= rot(tau*3./4.);
    } else if(l == 2){
      vCpos.yz *= rot(tau*-1./4.);
      vCpos.xz *= rot(tau*-3./4.);
    } else if(l == 3){
      vCpos.yz *= rot(tau*1./4.);
      vCpos.xz *= rot(tau*-3./4.);
    }
    gl_Position = vec4(pos, 0., 1.0);
    vUv = (pos.xy);
}

