struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] uv: vec2<f32>;
	[[location(2)]] color: vec4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
	[[location(1)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	out.position = vec4<f32>(in.position, 1.0);
	out.uv = in.uv;
	out.color = in.color;

	return out;
}

[[group(0), binding(0)]]
var texture: texture_2d<f32>;

[[group(0), binding(1)]]
var sampler: sampler;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	let texture_color = textureSample(texture, sampler, in.uv);

	if (texture_color.a < 0.01) {
		discard; 
	}

	return texture_color * in.color;
}