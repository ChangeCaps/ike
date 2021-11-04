struct VertexInput {
	[[builtin(vertex_index)]] index: u32;
	[[location(0)]] transform_0: vec4<f32>;
	[[location(1)]] transform_1: vec4<f32>;
	[[location(2)]] transform_2: vec4<f32>;
	[[location(3)]] transform_3: vec4<f32>; 
	[[location(4)]] color: vec4<f32>;
	[[location(5)]] depth: vec2<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
	var vertices = array<vec3<f32>, 6>(
		vec3<f32>(-1.0, -1.0, 0.0),
		vec3<f32>(1.0, -1.0, 0.0),
		vec3<f32>(-1.0, 1.0, 0.0),
		vec3<f32>(1.0, -1.0, 0.0),
		vec3<f32>(-1.0, 1.0, 0.0),
		vec3<f32>(1.0, 1.0, 0.0),
	); 

	let transform = mat4x4<f32>(
		in.transform_0,
		in.transform_1,
		in.transform_2,
		in.transform_3,
	);

	let vertex = vertices[in.index];
	let pos = transform * vec4<f32>(vertex, 1.0);

	return VertexOutput(vec4<f32>(pos.x, pos.y, in.depth.x + in.depth.y * vertex.y, pos.w), in.color); 
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return in.color;
}