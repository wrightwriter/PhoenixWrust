
W_PC_DEF{
  UboObject ubo;
}


// layout(location = 0) out vec3 vColor;
// layout(location = 1) out vec3 vNorm;
layout(location = 0) out vec2 vUv;


struct Vertex {
  vec2 position;
};


// W_aBUFF_DEF VertexBuff{
//   Vertex verts[];
// };



// Triangle strip
vec2 positions[4] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0,1.0)
);


W_BUFF_DEF IndicesBuff{
  uint16_t data[];
};


void main() {
    vec2 pos = positions[gl_VertexIndex];

    vUv = pos;
    
    pos.y *= 0.33;

    pos.y += 1.5;


    gl_Position = vec4(pos*1.,0, 1.0);


    gl_Position = PV * gl_Position;



    // fragColor = vec3(1,1,1);
    // vColor = vert.color.xyz;
    // vNorm = vert.normal.xyz;

}
