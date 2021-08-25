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
	screen_size: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	out.position = vec4<f32>(
		in.position / uniforms.screen_size * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 
		0.0, 
		1.0
	);
	out.uv = in.uv;
	out.color = in.color;

	return out;
}

[[group(1), binding(0)]]
var texture: texture_2d<f32>;

[[group(1), binding(1)]]
var sampler: sampler;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return textureSample(texture, sampler, in.uv) * in.color;
}