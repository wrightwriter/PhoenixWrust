
layout(location = 0) in vec2 vUv;
layout(location = 0) out vec4 oC;

W_PC_DEF{
  UboObject ubo;
  uint8_t idx_gbuff;
  uint8_t vertex_buff_idx;
  // BuffIndices indices;
  // BuffVerts verts;
}

// 

void main() {
    // oC = tex(shared_textures[int(PC.idx_gbuff)-1],fract(vUv.xy));
    oC = imageLoad(shared_images[max(int(PC.idx_gbuff)-1,0)], ivec2(fract(vUv.xy)*R));

    // oC = vec4(vUv.xyx + sin(float(frame)), 1.0);
}
