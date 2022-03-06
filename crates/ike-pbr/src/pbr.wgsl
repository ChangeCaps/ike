struct VertexInput {
	[[location(0)]]
	position: vec3<f32>;
	[[location(1)]]
	normal: vec3<f32>;
};

struct VertexOutput {
	[[builtin(position)]]
	position: vec4<f32>;
	[[location(0)]]
	w_position: vec3<f32>;
	[[location(1)]]
	w_normal: vec3<f32>;
};

struct Object {
	transform: mat4x4<f32>;
	view_proj: mat4x4<f32>;	
	position: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> object: Object;

[[stage(vertex)]]
fn vert(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;	

	let position = object.transform * vec4<f32>(in.position, 1.0);
	out.position = object.view_proj * position;
	out.w_position = position.xyz;
	let normal = object.transform * vec4<f32>(in.normal, 0.0);
	out.w_normal = normal.xyz;

	return out;
}

struct Material {
	base_color: vec4<f32>;
};

struct FragmentInput {
	[[location(0)]]
	w_position: vec3<f32>;
	[[location(1)]]
	w_normal: vec3<f32>;
};

[[stage(fragment)]]
fn frag(in: FragmentInput) -> [[location(0)]] vec4<f32> {
	var color = vec3<f32>(0.0);

	var light = vec3<f32>(0.0);

	return vec4<f32>(1.0);
}
