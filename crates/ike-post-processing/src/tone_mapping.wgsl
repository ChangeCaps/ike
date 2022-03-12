struct Uniforms {
	tone_mapping: u32;
};

[[group(0), binding(0)]]
var hdr_target: texture_2d<f32>;

[[group(0), binding(1)]]
var hdr_target_sampler: sampler;

[[group(0), binding(2)]]
var<uniform> uniforms: Uniforms;

fn saturate(x: vec3<f32>) -> vec3<f32> {
	return clamp(x, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn aces(x: vec3<f32>) -> vec3<f32> {
	let a = 2.51;
	let b = 0.03;
	let c = 2.43;
	let d = 0.59;
	let e = 0.14;
	return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}

[[stage(fragment)]]
fn frag([[location(0)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
	let color = textureSample(hdr_target, hdr_target_sampler, uv);

	let tone_mapped = aces(color.rgb);

	let gamma_corrected = pow(tone_mapped, vec3<f32>(1.0/2.2));

	return vec4<f32>(gamma_corrected, 1.0);
}