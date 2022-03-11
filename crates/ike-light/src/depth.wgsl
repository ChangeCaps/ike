struct Mesh {
	view_proj: mat4x4<f32>;	
	position: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> mesh: Mesh;

struct VertexInput {
	[[location(0)]] 
	position: vec3<f32>;
	[[location(1)]]
	transform_0: vec4<f32>;
	[[location(2)]]
	transform_1: vec4<f32>;
	[[location(3)]]
	transform_2: vec4<f32>;
	[[location(4)]]
	transform_3: vec4<f32>;
};

[[stage(vertex)]]
fn vert(in: VertexInput) -> [[builtin(position)]] vec4<f32> {
	let transform = mat4x4<f32>(
		in.transform_0,
		in.transform_1,
		in.transform_2,
		in.transform_3,
	);

	return mesh.view_proj * transform * vec4<f32>(in.position, 1.0);
}