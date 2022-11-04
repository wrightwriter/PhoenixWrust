
W_PC_DEF{
  UboObject ubo;
  uint8_t font_metadata_idx;
  uint8_t font_atlas_idx;
}


// layout(location = 0) in vec3 vColor;
// layout(location = 1) in vec3 vNorm;
layout(location = 0) in vec2 vUv;

layout(location = 0) out vec4 oC;
layout(location = 1) out vec4 oGNorm;
// layout(location = 2) out vec4 oGPotato;


struct Glyph{
  int unicode;
  float advance;

  float plane_bound_left;
  float plane_bound_bot;
  float plane_bound_right;
  float plane_bound_top;

  float atlas_bound_left;
  float atlas_bound_bot;
  float atlas_bound_right;
  float atlas_bound_top;
};


W_BUFF_DEF GlyphBuff{
  Glyph data[];
};


Glyph getGlyph(int idx){
  return GlyphBuff_get[uint(PC.font_metadata_idx)].data[idx];
}
float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}


void main() {
    // vec3 n = vNorm;
    // vec2 uv = U.xy/R.xy;
    
    vec2 uv = vUv;
    uv += 1.;
    uv *= 0.5;
    
    // vec4 albedo = vec4(vUv.xyx,1);
    vec4 albedo = vec4(0);

    const int let_cnt = 3;
    int[] letters = int[](CH_A,CH_B,CH_W);
    
    int id = int(floor(uv.x*float(let_cnt)));
    uv.x = fract(uv.x*float(let_cnt));
    
    

    // ivec2 sz = texSz(PC.idx_font).xy;
    // float dx = dFdx(vUv.x) * sz.x; 
    // float dy = dFdy(vUv.y) * sz.y;
    // float toPixels = 8.0 * inversesqrt(dx * dx + dy * dy);
    // float w = fwidth(sigDist);
    
    // Glyph glyph = getGlyph(letters[id] + int(fract(T*0.2)*9));
    Glyph glyph = getGlyph(letters[id]);
    
    vec2 canvas_sz = texSz(PC.font_atlas_idx);

    vec2 glyph_sz = vec2(
      glyph.atlas_bound_right - glyph.atlas_bound_left,
      glyph.atlas_bound_top - glyph.atlas_bound_bot
    )/canvas_sz;

    vec2 glyph_sz_b = vec2(
      glyph.plane_bound_right - glyph.plane_bound_left,
      glyph.plane_bound_top - glyph.plane_bound_bot
    );

  // float plane_bound_left;
  // float plane_bound_bot;
  // float plane_bound_right;
  // float plane_bound_top;

    // uv *= 0.4 + sin(T)*0.2;

    uv *= glyph_sz*1.;

    uv /= glyph_sz_b*1.;


    vec2 luv = uv;

    
    // luv.y = 1.-luv.y ;

    luv.x += glyph.atlas_bound_left/canvas_sz.x;
    luv.y += glyph.atlas_bound_bot/canvas_sz.y;



    
    // luv *= 0.2;
    
    
    if(
      luv.x < glyph.atlas_bound_right/canvas_sz.x &&
      luv.x > glyph.atlas_bound_left/canvas_sz.x &&
      luv.y < glyph.atlas_bound_top/canvas_sz.y &&
      luv.y > glyph.atlas_bound_bot/canvas_sz.y &&
      true
      ){
      
      albedo = tex_(int(PC.font_atlas_idx), luv);
    }
    

// luv *= 20.;
    // luv *= glyph.atlas_bound_l
    
    
    // albedo += uv.y;

    // albedo = uv.xyxy;
    // albedo = (vUv.xyxy + 1.)/2. - albedo;
    
    float w = 0.1;

    float sigDist = median(albedo.r, albedo.g, albedo.b);
    float opacity = smoothstep(0.5 - w, 0.5 + w, sigDist);    
    
    albedo = vec4(opacity);
    // albedo = uv.xyxy;
    

    
    




    oC.xyz = albedo.xyz;
    oC.w = 1.;
    
    

    // oGNorm = vec4(clamp(vNorm.xyz*0.5 + 0.5,0.,1.),1);
    oGNorm = vec4(0,0,1,1);
    // oGNorm = vec4(1,0,0,1);
}
