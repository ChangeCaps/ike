struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] uv: vec2<f32>;
	[[location(3)]] color: vec4<f32>;
	[[location(4)]] transform_0: vec4<f32>;
	[[location(5)]] transform_1: vec4<f32>;
	[[location(6)]] transform_2: vec4<f32>;
	[[location(7)]] transform_3: vec4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] normal: vec3<f32>;
	[[location(1)]] color: vec4<f32>;
};

[[block]]
struct Uniforms {
	view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]] 
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	let transform = mat4x4<f32>(
		in.transform_0,
		in.transform_1,
		in.transform_2,
		in.transform_3
	);

	out.position = uniforms.view_proj * transform * vec4<f32>(in.position, 1.0);
	out.normal = normalize((transform * vec4<f32>(in.normal, 0.0)).xyz);
	out.color = in.color;

	return out;
}

struct FragmentInput {
	[[builtin(front_facing)]] front: bool;
	[[location(0)]] normal: vec3<f32>; 
	[[location(1)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn main(in: FragmentInput) -> [[location(0)]] vec4<f32> {
	let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.0));

	var normal: vec3<f32>;

	if (in.front) {
		normal = in.normal;
	} else {
		normal = -in.normal; 
	}

	var diffuse: f32;

	if (dot(light_dir, in.normal) > .0) {
		diffuse = 1.0;
	} else {
		diffuse = 0.7;
	}

	return vec4<f32>(in.color.rgb * diffuse, in.color.a);
}