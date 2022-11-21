


W_PC_DEF{
  UboObject ubo;
  uint8_t potato;
}

layout(location = 0) in vec2 vUv;

layout(location = 0) out vec4 C;


void main() {

  // gl_Layer = 1;
  C = vec4(vUv.xy,0,1);
}

