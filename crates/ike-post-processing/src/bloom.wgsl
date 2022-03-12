struct Uniforms {
	threshold: f32;
	knee: f32;
	scale: f32;
};

[[group(0), binding(0)]]
var source: texture_2d<f32>;

[[group(0), binding(1)]]
var additional: texture_2d<f32>;

[[group(0), binding(2)]]
var source_sampler: sampler;

[[group(0), binding(3)]]
var<uniform> uniforms: Uniforms;

//fn textureSample(uv: vec2<f32>) -> vec4<f32> {
//	let iuv = vec2<i32>(round(uv));	
//	let f = fract(uv);
//
//	let a = textureLoad(source, iuv + vec2<i32>(0, 0), 0);
//	let b = textureLoad(source, iuv + vec2<i32>(1, 0), 0);
//	let c = textureLoad(source, iuv + vec2<i32>(0, 1), 0);
//	let d = textureLoad(source, iuv + vec2<i32>(1, 1), 0); 
//
//	let a = mix(a, b, f.x);
//	let b = mix(c, d, f.x);
//
//	return mix(a, b, f.y); 
//}

fn quadratic_threshold(color: vec4<f32>, threshold: f32, curve: vec3<f32>) -> vec4<f32> {
	let br = max(max(color.r, color.g), color.b);

	var rq = clamp(br - curve.x, 0.0, curve.y);
	rq = curve.z * rq * rq;

	return color * max(rq, br - threshold) / max(br, 0.0001); 
}

[[stage(fragment)]]
fn filter([[location(0)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
    var color = textureSample(source, source_sampler, uv);

	let curve = vec3<f32>(
		uniforms.threshold - uniforms.knee,
		uniforms.knee * 2.0,
		0.25 / uniforms.knee,
	);
	
	color = quadratic_threshold(color, uniforms.threshold, curve);	

	// we don't want negative colors
	color = max(color, vec4<f32>(0.00001));

	return color;
}

fn sample_3x3_tent(tex: texture_2d<f32>, uv: vec2<f32>, scale: vec2<f32>) -> vec4<f32> {
	let d = vec4<f32>(1.0, 1.0, -1.0, 0.0);

	var s: vec4<f32> = textureSample(tex, source_sampler, uv - d.xy * scale);
	s = s + textureSample(tex, source_sampler, uv - d.wy * scale) * 2.0;
	s = s + textureSample(tex, source_sampler, uv - d.zy * scale);
                        
	s = s + textureSample(tex, source_sampler, uv + d.zw * scale) * 2.0;
	s = s + textureSample(tex, source_sampler, uv       ) * 4.0;
	s = s + textureSample(tex, source_sampler, uv + d.xw * scale) * 2.0;
                        
	s = s + textureSample(tex, source_sampler, uv + d.zy * scale);
	s = s + textureSample(tex, source_sampler, uv + d.wy * scale) * 2.0;
	s = s + textureSample(tex, source_sampler, uv + d.xy * scale);

	return s / 16.0;
}

[[stage(fragment)]]
fn down_sample([[location(0)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
	let texel_size = 1.0 / vec2<f32>(textureDimensions(source));

    let scale = texel_size * 2.0 * uniforms.scale;
	
	let a = textureSample(source, source_sampler, uv + vec2<f32>(-1.0, -1.0) * scale);
	let b = textureSample(source, source_sampler, uv + vec2<f32>( 0.0, -1.0) * scale);
	let c = textureSample(source, source_sampler, uv + vec2<f32>( 1.0, -1.0) * scale);
	let d = textureSample(source, source_sampler, uv + vec2<f32>(-0.5, -0.5) * scale);
	let e = textureSample(source, source_sampler, uv + vec2<f32>( 0.5, -0.5) * scale);
	let f = textureSample(source, source_sampler, uv + vec2<f32>(-1.0,  0.0) * scale);
	let g = textureSample(source, source_sampler, uv + vec2<f32>( 0.0,  0.0) * scale);
	let h = textureSample(source, source_sampler, uv + vec2<f32>( 1.0,  0.0) * scale);
	let i = textureSample(source, source_sampler, uv + vec2<f32>(-0.5,  0.5) * scale);
	let j = textureSample(source, source_sampler, uv + vec2<f32>( 0.5,  0.5) * scale);
	let k = textureSample(source, source_sampler, uv + vec2<f32>(-1.0,  1.0) * scale);
	let l = textureSample(source, source_sampler, uv + vec2<f32>( 0.0,  1.0) * scale);
	let m = textureSample(source, source_sampler, uv + vec2<f32>( 1.0,  1.0) * scale);

	let div = (1.0 / 4.0) * vec2<f32>(0.5, 0.125);

	var o = (d + e + i + j) * div.x;
	o = o + (a + b + g + f) * div.y;
	o = o + (b + c + h + g) * div.y;
	o = o + (f + g + l + k) * div.y;
	o = o + (g + h + m + l) * div.y;

	return o;
}

[[stage(fragment)]]
fn up_sample([[location(0)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
	let texel_size = 1.0 / vec2<f32>(textureDimensions(source));

    let scale = uniforms.scale * 0.95 * uniforms.scale;

	let up = sample_3x3_tent(source, uv, texel_size);
	let color = textureSample(additional, source_sampler, uv) + up;

	return color;
}

[[stage(fragment)]]
fn apply([[location(0)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
	let texel_size = 1.0 / vec2<f32>(textureDimensions(additional));

	let source_color = textureSample(source, source_sampler, uv);
	let additional_color = sample_3x3_tent(additional, uv, texel_size);

	return source_color + additional_color;
}