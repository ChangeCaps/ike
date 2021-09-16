var vertices: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
	vec3<f32>(-1.0, -1.0, 0.0),
	vec3<f32>(1.0, -1.0, 0.0),
	vec3<f32>(-1.0, 1.0, 0.0),
	vec3<f32>(1.0, -1.0, 0.0),
	vec3<f32>(-1.0, 1.0, 0.0),
	vec3<f32>(1.0, 1.0, 0.0),
);

struct VertexInput {
	[[builtin(vertex_index)]] index: u32;
	[[location(0)]] transform_0: vec4<f32>;
	[[location(1)]] transform_1: vec4<f32>;
	[[location(2)]] transform_2: vec4<f32>;
	[[location(3)]] transform_3: vec4<f32>; 
};

[[stage(vertex)]]
fn main(in: VertexInput) -> [[builtin(position)]] vec4<f32> {
	let transform = mat4x4<f32>(
		in.transform_0,
		in.transform_1,
		in.transform_2,
		in.transform_3,
	);

	let pos = transform * vec4<f32>(vertices[in.index], 1.0);

	return vec4<f32>(pos.x, pos.y, 0.0, pos.w); 
}

[[stage(fragment)]]
fn main() -> [[location(0)]] vec4<f32> {
	return vec4<f32>(1.0);
}