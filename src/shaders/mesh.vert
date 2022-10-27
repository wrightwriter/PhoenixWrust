




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
  uint8_t indices_buff_idx;
  uint8_t vertex_buff_idx;
  // BuffIndices indices;
  // BuffVerts verts;
}


layout(location = 0) out vec3 vColor;
layout(location = 1) out vec3 vNorm;
layout(location = 2) out vec2 vUv;
// layout(location = 2) out vec3 vNorm;


W_BUFF_DEF VertexBuff{
  Vertex verts[];
};

W_BUFF_DEF IndicesBuff{
  uint data[];
};


void main() {
    // uint idx = PC.indices.data[gl_VertexIndex];
    uint idx = IndicesBuff_get[uint(PC.indices_buff_idx)].data[gl_VertexIndex];
    
    // idx = gl_VertexIndex;
    Vertex vert = VertexBuff_get[uint(PC.vertex_buff_idx)].verts[idx];



    // gl_Position = vec4(vert.position.xyz* 0.01/0.01, 1.0);
    gl_Position = vec4(vert.position.xyz* 0.01/0.01, 1.0);

    gl_Position = PV * gl_Position;



    // fragColor = vec3(1,1,1);
    // vColor = vert.color.xyz;
    vNorm = vert.normal.xyz;

    vUv = vert.uv;
}
