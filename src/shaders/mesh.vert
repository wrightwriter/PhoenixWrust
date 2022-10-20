

struct Vertex {
  vec4 position;
  vec4 normal;
  vec4 color;
  vec2 uvs;
  // vec2 pad;
  // vec2 pada;
  // vec2 padc;
  // vec2 padd;
};


// Wa_BDA_DEF BuffVerts{
//   Vertex data[];
// };

// Wa_BDA_DEF BuffIndices{
//   uint data[];
// };



W_PC_DEF{
  UboObject ubo;
  // BuffIndices indices;
  // BuffVerts verts;
  uint8_t a;
}


layout(location = 0) out vec3 vColor;
layout(location = 1) out vec3 vNorm;
// layout(location = 2) out vec3 vNorm;


W_BUFF_DEF VertexBuff{
  Vertex verts[];
};

W_BUFF_DEF IndicesBuff{
  uint data[];
};


void main() {
    // uint idx = PC.indices.data[gl_VertexIndex];
    uint idx = IndicesBuff_get[1].data[gl_VertexIndex];
    
    Vertex vert = VertexBuff_get[0].verts[idx];


    gl_Position = vec4(vert.position.xyz, 1.0);

    gl_Position = PV * gl_Position;


    // fragColor = vec3(1,1,1);
    vColor = vert.color.xyz;
    vNorm = vert.normal.xyz;
}
