

// Triangle strip
vec2 positions[4] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0,1.0)
);



W_PC_DEF{
  UboObject ubo;
  uint8_t potato;
}


layout(location = 0) out vec2 vUv;


void main() {
    gl_Layer = int(gl_InstanceIndex);

    // if(gl_Layer == 0){

    // }

    vec2 pos = positions[gl_VertexIndex];

    gl_Position = vec4(pos, 0.0, 1.0);

    vUv = gl_Position.xy;
}

