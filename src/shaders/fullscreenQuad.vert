layout(location = 0) out vec2 vUv;


// Triangle strip
vec2 positions[4] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0,1.0)
);

void main() {
    vec2 pos = positions[gl_VertexIndex];

    gl_Position = vec4(pos, 0.0, 1.0);

    vUv = gl_Position.xy;
}
