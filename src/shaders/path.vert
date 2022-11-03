
W_PC_DEF{
  UboObject ubo;
  uint8_t indices_buff_idx;
  uint8_t vertex_buff_idx;
}


// layout(location = 0) out vec3 vColor;
// layout(location = 1) out vec3 vNorm;
// layout(location = 2) out vec2 vUv;



struct Vertex {
  vec2 position;
};


W_BUFF_DEF VertexBuff{
  Vertex verts[];
};


// struct Index{
//   uint16_t idx;
// }

W_BUFF_DEF IndicesBuff{
  uint16_t data[];
};


void main() {
    // uint idx = PC.indices.data[gl_VertexIndex];
    uint idx = uint(IndicesBuff_get[uint(PC.indices_buff_idx)].data[gl_VertexIndex]);
    
    // idx = gl_VertexIndex;
    Vertex vert = VertexBuff_get[uint(PC.vertex_buff_idx)].verts[idx];

    // gl_Position = vec4(vert.position.xyz* 0.01/0.01, 1.0);
    vert.position *= 0.01;
    // vert.position += 1.;

    gl_Position = vec4(vert.position.xy,0, 1.0);

    gl_Position = PV * gl_Position;



    // fragColor = vec3(1,1,1);
    // vColor = vert.color.xyz;
    // vNorm = vert.normal.xyz;

    // vUv = vert.uv;
}
