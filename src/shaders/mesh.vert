

struct Vertex {
  vec4 position;
  vec4 normal;
  vec4 color;
  vec2 uvs;
};


W_BDA_DEF BuffVerts{
    Vertex data[];
};

W_BDA_DEF BuffIndices{
    uint data[];
};



W_PC_DEF{
  UboObject ubo;
  BuffIndices indices;
  BuffVerts verts;
}


layout(location = 0) out vec3 fragColor;


void main() {
    uint idx = PC.indices.data[gl_VertexIndex];

    Vertex vert = PC.verts.data[idx];

    gl_Position = vec4(vert.position.xyz, 1.0);

    gl_Position = V * gl_Position;
    gl_Position = P * gl_Position;

    
    // texture(sampler2D(shared_textures[0], shared_samplers[0]),vec2(0));
    // texture(sampler2D(shared_textures[0], shared_samplers[0]),vec2(0));
    tex(shared_textures[0], vec2(0));


    // fragColor = vec3(1,1,1);
    fragColor = vert.color.xyz;
}
