struct Mesh {
	transform: mat4x4<f32>;
	view_proj: mat4x4<f32>;	
	position: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> mesh: Mesh;

[[stage(vertex)]]
fn vert([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
	return mesh.view_proj * mesh.transform * vec4<f32>(position, 1.0);
}