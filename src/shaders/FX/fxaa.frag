

W_UBO_DEF{
  float lumaThreshold;
  float maxSpan;
  float directionReduceMultiplier;
  float directionReduceMinimum;
}

//


W_PC_DEF{
  UboObject ubo;
  uint16_t s_InputFB;
}

//



layout(location = 0) in vec2 uVar;
layout(location = 0) out vec4 fragColor;



void main(void) {
  float lumaThreshold = -0.;
  float maxSpan = 2.;
  float directionReduceMultiplier = 1.;
  float directionReduceMinimum = 0.;


	vec2 coords = U/R;


    const int u_showEdges = 0;
    const int u_fxaaOn = 1;

    vec2 u_texelStep = 1.0 / texSz(PC.s_InputFB);
    vec3 rgbM = min(vec3(1), tex_(PC.s_InputFB, coords).rgb);

// 	// Possibility to toggle FXAA on and off.
	if (u_fxaaOn == 0) {
		fragColor = vec4(rgbM, 1.0);
		return;
	}

// 	// Sampling neighbour texels. Offsets are adapted to OpenGL texture coordinates.
    vec3 rgbNW = min(vec3(1),tex_(PC.s_InputFB, coords+ vec2(-1, 1)*u_texelStep).rgb);
    vec3 rgbNE = min(vec3(1),tex_(PC.s_InputFB, coords+ vec2(1, 1)*u_texelStep).rgb);
    vec3 rgbSW = min(vec3(1),tex_(PC.s_InputFB, coords+ vec2(-1, -1)*u_texelStep).rgb);
    vec3 rgbSE = min(vec3(1),tex_(PC.s_InputFB, coords+ vec2(1, -1)*u_texelStep).rgb);

// 	// see http://en.wikipedia.org/wiki/Grayscale
	const vec3 toLuma = vec3(0.299, 0.587, 0.114);

// 	// Convert from RGB to luma.
	float lumaNW = dot(rgbNW, toLuma);
	float lumaNE = dot(rgbNE, toLuma);
	float lumaSW = dot(rgbSW, toLuma);
	float lumaSE = dot(rgbSE, toLuma);
	float lumaM = dot(rgbM, toLuma);

// 	// Gather minimum and maximum luma.
	float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
	float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

// 	// If contrast is lower than a maximum threshold ...
	if (lumaMax - lumaMin < lumaMax * lumaThreshold)
	{
		// ... do no AA and return.
		fragColor = vec4(rgbM, 1.0);

		return;
	}

// 	// Sampling is done along the gradient.
	vec2 samplingDirection;
	samplingDirection.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    samplingDirection.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));

//     // Sampling step distance depends on the luma: The brighter the sampled texels, the smaller the final sampling step direction.
//     // This results, that brighter areas are less blurred/more sharper than dark areas.
    float samplingDirectionReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * 0.25 * directionReduceMultiplier, directionReduceMinimum);

// 	// Factor for norming the sampling direction plus adding the brightness influence.
	float minSamplingDirectionFactor = 1.0 / (min(abs(samplingDirection.x), abs(samplingDirection.y)) + samplingDirectionReduce);

//     // Calculate final sampling direction vector by reducing, clamping to a range and finally adapting to the texture size.
    samplingDirection = clamp(samplingDirection * minSamplingDirectionFactor, vec2(-maxSpan, -maxSpan), vec2(maxSpan, maxSpan)) * u_texelStep;

// 	// Inner samples on the tab.
	vec3 rgbSampleNeg = min(vec3(1),tex_(PC.s_InputFB, coords + samplingDirection * (1.0/3.0 - 0.5)).rgb);
	vec3 rgbSamplePos = min(vec3(1),tex_(PC.s_InputFB, coords + samplingDirection * (2.0/3.0 - 0.5)).rgb);

	vec3 rgbTwoTab = (rgbSamplePos + rgbSampleNeg) * 0.5;

// 	// Outer samples on the tab.
	vec3 rgbSampleNegOuter = min(vec3(1),tex_(PC.s_InputFB, coords + samplingDirection * (0.0/3.0 - 0.5)).rgb);
	vec3 rgbSamplePosOuter = min(vec3(1),tex_(PC.s_InputFB, coords + samplingDirection * (3.0/3.0 - 0.5)).rgb);

	vec3 rgbFourTab = (rgbSamplePosOuter + rgbSampleNegOuter) * 0.25 + rgbTwoTab * 0.5;

	// Calculate luma for checking against the minimum and maximum value.
	float lumaFourTab = dot(rgbFourTab, toLuma);

	// Are outer samples of the tab beyond the edge ...
	if (lumaFourTab < lumaMin || lumaFourTab > lumaMax)
	{
		// ... yes, so use only two samples.
		fragColor = vec4(rgbTwoTab, 1.0);
//		fragColor.r = 1.0;
	}
	else
	{
		// ... no, so use four samples.
		fragColor = vec4(rgbFourTab, 1.0);
//		fragColor.g = 1.0;
	}

// 	// Show edges for debug purposes.
// 	if (u_showEdges != 0)
// 	{
// 		fragColor.r = 1.0;
// 	}
}
