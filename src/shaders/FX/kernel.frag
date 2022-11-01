

W_UBO_DEF{
	float u_type;
	float u_edgeType;
	float u_kernSz;
	float u_sharpenAmt;
	float u_sharpenBias;
	float u_blurCenterBias;
	float u_blurEdgeBias;
}

//


W_PC_DEF{
  UboObject ubo;
  uint8_t s_InputColTex;
}

//

#define u_resolution R


layout(location = 0) in vec2 uVar;
layout(location = 0) out vec4 C;


#include "utils.include"


// #define texture(a,b) textureSmooth(a,b)

float u_type;
float u_edgeType;
float u_kernSz;
float u_sharpenAmt;
float u_sharpenBias;
float u_blurCenterBias;
float u_blurEdgeBias;

#define SHARPEN 0.
#define BLUR 1.
#define ANISOTROPIC_BLUR 2.
#define EDGE 4.


#define KAYYALI_NESW 1.
#define KAYYALI_SENW 2.
#define PREWITT 3.
#define ROBERTSCROSS 4.
#define SCHARR 5.
#define SOBEL 6.


const vec2[] offset = vec2[9](
    vec2(-1, -1),
    vec2(0.0, -1),
    vec2(1, -1),

    vec2(-1, 0.0),
    vec2(0.0, 0.0),
    vec2(1, 0.0),

    vec2(-1, 1),
    vec2(0.0, 1),
    vec2(1, 1)
);




//options are edge, colorEdge, or trueColorEdge
#define EDGE_FUNC trueColorEdge


const float[] kayyali_NESW = float[9](-6.0, 0.0, 6.0,
							   0.0, 0.0, 0.0,
							   6.0, 0.0, -6.0);

const float[] kayyali_SENW = float[9](6.0, 0.0, -6.0,
							   0.0, 0.0, 0.0,
							   -6.0, 0.0, 6.0);

// Prewitt masks (see http://en.wikipedia.org/wiki/Prewitt_operator)
const float[] prewittKernelX = float[9](-1.0, 0.0, 1.0,
								 -1.0, 0.0, 1.0,
								 -1.0, 0.0, 1.0);

const float[] prewittKernelY = float[9](1.0, 1.0, 1.0,
								 0.0, 0.0, 0.0,
								 -1.0, -1.0, -1.0);

// Roberts Cross masks (see http://en.wikipedia.org/wiki/Roberts_cross)
const float[] robertsCrossKernelX = float[9](1.0, 0.0, 0.0,
									  0.0, -1.0, 0.0,
									  0.0, 0.0, 0.0);
const float[] robertsCrossKernelY = float[9](0.0, 1.0, 0.0,
									  -1.0, 0.0, 0.0,
									  0.0, 0.0, 0.0);

// Scharr masks (see http://en.wikipedia.org/wiki/Sobel_operator#Alternative_operators)
const float[] scharrKernelX = float[9](3.0, 10.0, 3.0,
								0.0, 0.0, 0.0,
								-3.0, -10.0, -3.0);

const float[] scharrKernelY = float[9](3.0, 0.0, -3.0,
								10.0, 0.0, -10.0,
								3.0, 0.0, -3.0);

// Sobel masks (see http://en.wikipedia.org/wiki/Sobel_operator)
const float[] sobelKernelX = float[9](1.0, 0.0, -1.0,
							   2.0, 0.0, -2.0,
							   1.0, 0.0, -1.0);

const float[] sobelKernelY = float[9](-1.0, -2.0, -1.0,
							   0.0, 0.0, 0.0,
							   1.0, 2.0, 1.0);
float convolve(float[9] kernel, vec2 vUv, vec2 st) {
	float result = 0.0;
	for (int i = 0; i < 3; i++) {
		for (int j = 0; j < 3; j++) {
            vec2 it = vec2(i,j);
            float image = tex_(PC.s_InputColTex, vUv + (vec2(it) - 3./2.)*st).x;
			result += kernel[i + j * 3]*image;
		}
	}
	return result;
}

//helper function for colorEdge()
float convolveComponent(float[9] kernelX, float[9] kernelY, vec2 vUv, vec2 st) {
	vec2 result;
	result.x = convolve(kernelX, vUv, st);
	result.y = convolve(kernelY, vUv, st);
	return clamp(length(result), 0.0, 255.0);
}

// https://www.shadertoy.com/view/ldsSWr
vec4 getEdge(vec2 vUv, vec2 st){
    vec4 sum = vec4(0);


    if (u_edgeType == KAYYALI_NESW){
        sum += convolveComponent( kayyali_NESW, kayyali_NESW, vUv, st);
    } else if (u_edgeType == KAYYALI_SENW){
        sum += convolveComponent( kayyali_SENW, kayyali_SENW, vUv, st);
    } else if (u_edgeType == PREWITT ){
        sum += convolveComponent( prewittKernelX, prewittKernelY, vUv, st);
    } else if (u_edgeType == ROBERTSCROSS ){
        sum += convolveComponent( robertsCrossKernelX, robertsCrossKernelY, vUv, st);
    } else if (u_edgeType == SOBEL){
        sum += convolveComponent( sobelKernelX, sobelKernelY, vUv, st);
    } else if (u_edgeType == SCHARR){
        sum += convolveComponent( scharrKernelX, scharrKernelY, vUv, st);
    }

    return sum;
}

#define BLUR_RATIO 0.		// ratio of the original pixel value to the blurred value
#define SHARPNESS 1.		// sharpness of the blur kernel, 0.0 gives a uniform distribution
#define VECTOR_SHARPEN 0. // sharpens the vector field

// https://www.shadertoy.com/view/ldcSDB
vec4 getAnisotropicBlur(vec2 vUv, vec2 st){
    float step_x = st.x;
    float step_y = st.y;
    vec2 n  = vec2(0.0, step_y);
    vec2 ne = vec2(step_x, step_y);
    vec2 e  = vec2(step_x, 0.0);
    vec2 se = vec2(step_x, -step_y);
    vec2 s  = vec2(0.0, -step_y);
    vec2 sw = vec2(-step_x, -step_y);
    vec2 w  = vec2(-step_x, 0.0);
    vec2 nw = vec2(-step_x, step_y);

    vec2 ab =    tex_(PC.s_InputColTex, fract(vUv)).xy;
    vec2 ab_n =  tex_(PC.s_InputColTex, fract(vUv+n)).xy;
    vec2 ab_e =  tex_(PC.s_InputColTex, fract(vUv+e)).xy;
    vec2 ab_s =  tex_(PC.s_InputColTex, fract(vUv+s)).xy;
    vec2 ab_w =  tex_(PC.s_InputColTex, fract(vUv+w)).xy;
    vec2 ab_nw = tex_(PC.s_InputColTex, fract(vUv+nw)).xy;
    vec2 ab_sw = tex_(PC.s_InputColTex, fract(vUv+sw)).xy;
    vec2 ab_ne = tex_(PC.s_InputColTex, fract(vUv+ne)).xy;
    vec2 ab_se = tex_(PC.s_InputColTex, fract(vUv+se)).xy;

    const float _K0 = -20.0/6.0; // center weight
    const float _K1 = 4.0/6.0;   // edge-neighbors
    const float _K2 = 1.0/6.0;   // vertex-neighbors

    // laplacian
    vec2 lapl  = _K0*ab + _K1*(ab_n + ab_e + ab_w + ab_s) + _K2*(ab_nw + ab_sw + ab_ne + ab_se);

    ab += -VECTOR_SHARPEN * lapl;


    vec3 im =    tex_(PC.s_InputColTex, fract(vUv)).xyz;
    vec3 im_n =  tex_(PC.s_InputColTex, fract(vUv+n)).xyz;
    vec3 im_e =  tex_(PC.s_InputColTex, fract(vUv+e)).xyz;
    vec3 im_s =  tex_(PC.s_InputColTex, fract(vUv+s)).xyz;
    vec3 im_w =  tex_(PC.s_InputColTex, fract(vUv+w)).xyz;
    vec3 im_nw = tex_(PC.s_InputColTex, fract(vUv+nw)).xyz;
    vec3 im_sw = tex_(PC.s_InputColTex, fract(vUv+sw)).xyz;
    vec3 im_ne = tex_(PC.s_InputColTex, fract(vUv+ne)).xyz;
    vec3 im_se = tex_(PC.s_InputColTex, fract(vUv+se)).xyz;

    // a gaussian centered around the point at 'ab'
    #define e(x,y) exp(-SHARPNESS * dot(vec2(x,y) - ab, vec2(x,y) - ab))

    float D_c =  e( 0.0, 0.0);
    float D_e =  e( 1.0, 0.0);
    float D_w =  e(-1.0, 0.0);
    float D_n =  e( 0.0, 1.0);
    float D_s =  e( 0.0,-1.0);
    float D_ne = e( 1.0, 1.0);
    float D_nw = e(-1.0, 1.0);
    float D_se = e( 1.0,-1.0);
    float D_sw = e(-1.0,-1.0);

    // normalize the blur kernel
    float dn = D_c + D_e + D_w + D_n + D_s + D_ne + D_nw + D_se + D_sw;

    vec3 blur_im = (D_c*im
        + im_n*D_n + im_ne*D_ne
        + im_e*D_e + im_se*D_se
        + im_s*D_s + im_sw*D_sw
        + im_w*D_w + im_nw*D_nw) / dn;

    return vec4(clamp(BLUR_RATIO * im + (1.0 - BLUR_RATIO) * blur_im, 0.0, 1.0), 0.0);
}

float[9] getSharpenKernel(float coeffScale, float sharpenAmt){
    float coeffA = -coeffScale/4.*sharpenAmt;
    float coeffB = -(1.-coeffScale)/4.*sharpenAmt;
    return float[9](
        coeffA, coeffB, coeffA,
        coeffB, sharpenAmt + 1., coeffB,
        coeffA, coeffB, coeffA
    );
}


float[9] getBlurKernel(float coeffScale, float centerWeight){
    float notCenterWeight = 1. - centerWeight;

    float coeffA = notCenterWeight*coeffScale/4.;
    float coeffB = notCenterWeight*(1.-coeffScale)/4.;
    return float[9](
        coeffA, coeffB, coeffA,
        coeffB, centerWeight, coeffB,
        coeffA, coeffB, coeffA
    );
}

vec4 get(vec2 uv, float st){
    vec2 step = st / u_resolution.xy;
    float ratio = u_resolution.x/u_resolution.y;
    // if(ratio > 1.){
    //     step.y /= ratio;
    // } else {
    //     step.x *= ratio; // ?
    // }
    vec2 coord = uv;

    vec4 sum = vec4(0);

    if(u_type == ANISOTROPIC_BLUR){

        sum = getAnisotropicBlur(uv, step);

    } else if(u_type == EDGE){

        sum = getEdge(uv, step);

    } else {
        for (int i = 0; i < 9; i++) {
            vec4 color = tex_(PC.s_InputColTex, coord + offset[i]*step);
            if(u_type == SHARPEN){
                float[9] kernelSharpen = getSharpenKernel(u_sharpenBias, u_sharpenAmt);
                sum += color * kernelSharpen[i];
            } else if(u_type == BLUR){
                float[9] kernelBlur = getBlurKernel(u_blurEdgeBias, u_blurCenterBias);
                sum += color * kernelBlur[i];
            }
        }
    }


    return sum;
}

void main() {
	// u_type = 0.;
	u_type = PC.ubo.u_type;
	u_edgeType = PC.ubo.u_edgeType;
	u_kernSz = PC.ubo.u_kernSz;
	u_sharpenAmt = PC.ubo.u_sharpenAmt;
	u_sharpenBias = PC.ubo.u_sharpenBias;
	u_blurCenterBias = PC.ubo.u_blurCenterBias;
	u_blurEdgeBias = PC.ubo.u_blurEdgeBias;

  vec2 uv = uVar*0.5 + 0.5;
  C = get(uv, u_kernSz);
  C = clamp(C, 0., 1.);
}