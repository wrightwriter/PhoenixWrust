


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
  // uint8_t indices_buff_a_idx;
  // uint8_t indices_buff_b_idx;
  uint16_t indices_buff_idx;
  uint16_t vertex_buff_idx;
  // BuffIndices indices;
  // BuffVerts verts;
}


layout(location = 0) out vec3 vColor;
layout(location = 1) out vec3 vNorm;
layout(location = 2) out vec2 vUv;
layout(location = 3) out vec2 vVel;
// layout(location = 4) out mat3 vTBN;
// layout(location = 4) out vec3 vT;
// layout(location = 5) out vec3 vB;
layout(location = 6) out vec3 vPos;
// layout(location = 2) out vec3 vNorm;


W_BUFF_DEF VertexBuff{
  Vertex verts[];
};

W_BUFF_DEF IndicesBuff{
  uint data[];
};

void main() {
    // uint idx = PC.indices.data[gl_VertexIndex];
    // uint indices_idx = 0u|| (uint(PC.indices_buff_a_idx) << 8u) || (uint(PC.indices_buff_b_idx) << 8u);
    // uint indices_idx = 0u || (uint(PC.indices_buff_a_idx) << 8) ;
    // uint indices_idx = 0u;
    // indices_idx = 0u | (uint(PC.indices_buff_a_idx) << 8) ;
    // indices_idx |= (uint(PC.indices_buff_b_idx)) ;

    // uint indices_idx = concatu8(PC.indices_buff_a_idx, PC.indices_buff_b_idx);
    uint indices_idx = uint(PC.indices_buff_idx);
    uint vertex_buff_idx = uint(PC.vertex_buff_idx);


    uint idx = IndicesBuff_get[indices_idx].data[gl_VertexIndex];
    Vertex vert = VertexBuff_get[vertex_buff_idx].verts[idx];
    vNorm = vert.normal.xyz;
    vUv = vert.uv;

    uint idx_base = (gl_VertexIndex/3)*3;

    uint idx_a = IndicesBuff_get[indices_idx].data[idx_base];
    uint idx_b = IndicesBuff_get[indices_idx].data[idx_base + 1];


    Vertex vert_base = VertexBuff_get[vertex_buff_idx].verts[idx_a];
    Vertex vert_next = VertexBuff_get[vertex_buff_idx].verts[idx_b];
    // Vertex vert_last = VertexBuff_get[vertex_buff_idx].verts[idx_base + (idx-2)%3];
    
    // vec3 t = (vert_next.position - vert.position);
    // vec3 n = vert_base.normal.xyz;
    // vec3 b = (cross(n,t));
    
    // vT = t;
    // vB = b;
    
    

    // vert.position.xyz *= 100000.;
    // vert.position.xyz *= 0.01;
    
    // Vertex vert_next = VertexBuff_get[vertex_buff_idx].verts[idx];


    
    


    // idx = gl_VertexIndex;
    // uint indices_idx = 0;

    // uint aaaaa = uint(PC.indices_buff_idx);

    
    // vert.position *= 0.01;
    // vert.position *= 0.01;

    int steps = 16;


    vec4 pos_prev = Pprev * Vprev * vec4(vert.position.xyz, 1.0);
    pos_prev.xyz /= pos_prev.w;
    
    vec4 pos_curr = P * V * vec4(vert.position.xyz,1);
    pos_curr.xyz /= pos_curr.w;
    
    vec2 vel = pos_curr.xy - pos_prev.xy;
    // vel += 1.;
    // vel *= 0.5;
    
    vVel = vel;

    // pos_prev - pos_curr;



    mat4 proj = P;
    vec2 h = HammersleyNorm(int(frame)%steps, steps);

    h = halton_2_3(3, int(frame)%steps);

    // h = h*2. - 2.;
    // h *= 10.;
  
    // fragCoord += h*0.75;
    
    proj[2][0] += h.x/R.x*1.;
    proj[2][1] += h.y/R.y*1.;

    vPos = vert.position.xyz;
    gl_Position = vec4(vert.position.xyz, 1.0);
    gl_Position = proj * V * gl_Position;

}
