struct VertexInput {
	[[location(0)]]
	position: vec4<f32>;
	[[location(1)]]
	color: vec4<f32>;
};

struct VertexOutput {
	[[builtin(position)]]
	position: vec4<f32>;
	[[location(0)]]
	color: vec4<f32>;
};

[[stage(vertex)]]
fn vert(in: VertexInput) -> VertexOutput {
	return VertexOutput(in.position, in.color);
}

[[stage(fragment)]]
fn frag(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return in.color;
}