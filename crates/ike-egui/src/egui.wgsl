struct VertexInput {
	[[location(0)]] position: vec2<f32>;
	[[location(1)]] uv: vec2<f32>;
	[[location(2)]] color: vec4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
	[[location(1)]] color: vec4<f32>;
};

[[block]]
struct Uniforms {
	size: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[group(0), binding(1)]]
var texture: texture_2d<f32>;

[[group(0), binding(2)]]
var sampler: sampler;

fn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
	let cutoff = srgb < vec3<f32>(10.31475);
	let lower = srgb / vec3<f32>(3294.6);
	let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));
	return mix(higher, lower, vec3<f32>(cutoff));
}

[[stage(vertex)]]
fn main(input: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	let color = vec4<f32>(linear_from_srgb(input.color.rgb * 255.0), input.color.a);

	out.position = vec4<f32>(
		2.0 * input.position.x / uniforms.size.x - 1.0,
		1.0 - 2.0 * input.position.y / uniforms.size.y,
		0.0,
		1.0,
	); 
	out.uv = input.uv;
	out.color = color;

	return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return in.color * textureSample(texture, sampler, in.uv);
}
