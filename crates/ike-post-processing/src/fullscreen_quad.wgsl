var<private> VERTICES: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
	vec2<f32>(-1.0, -1.0),
	vec2<f32>(-1.0, 1.0),
	vec2<f32>(1.0, -1.0),
	vec2<f32>(-1.0, 1.0),
	vec2<f32>(1.0, -1.0),
	vec2<f32>(1.0, 1.0),
);

struct VertexOutput {
	[[builtin(position)]]
	position: vec4<f32>;
	[[location(0)]]
	uv: vec2<f32>;
};

[[stage(vertex)]]
fn vert([[builtin(vertex_index)]] index: u32) -> VertexOutput {
	var out: VertexOutput;

	let vertex = VERTICES[index];

	out.position = vec4<f32>(vertex, 0.0, 1.0);

	let flip_correction = vec2<f32>(0.5, -0.5);
	out.uv = vertex * flip_correction + 0.5;

	return out;
}