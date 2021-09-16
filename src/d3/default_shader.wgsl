let max_point_lights: u32 = 64u32;

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
	[[location(0)]] w_position: vec4<f32>;
	[[location(1)]] w_normal: vec3<f32>;
	[[location(2)]] color: vec4<f32>;
};

struct PointLight {
	position: vec3<f32>;
	intensity: f32;
	color: vec4<f32>;
};

[[block]]
struct Uniforms {
	view_proj: mat4x4<f32>;
	point_light_count: u32;
	point_lights: array<PointLight, max_point_lights>;
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

	out.w_position = transform * vec4<f32>(in.position, 1.0);
	out.position = uniforms.view_proj * transform * vec4<f32>(in.position, 1.0);
	out.w_normal = normalize((transform * vec4<f32>(in.normal, 0.0)).xyz);
	out.color = in.color;

	return out;
}

struct FragmentInput {
	[[builtin(front_facing)]] front: bool;
	[[location(0)]] position: vec4<f32>;
	[[location(1)]] normal: vec3<f32>; 
	[[location(2)]] color: vec3<f32>;
};

[[stage(fragment)]]
fn main(in: FragmentInput) -> [[location(0)]] vec4<f32> {
	var normal: vec3<f32>;

	if (in.front) {
		normal = in.normal;
	} else {
		normal = -in.normal; 
	}

	var color: vec3<f32> = vec3<f32>(0.0);	

	for (var i: u32 = 0u32; i < uniforms.point_light_count; i = i + 1u32) {
		let point_light = uniforms.point_lights[i];

		let light_dir = normalize(point_light.position - in.position.xyz);

		let diffuse = max(dot(light_dir, normal), 0.0);
		let diffuse_strength = 1.0 / pow(distance(point_light.position, in.position.xyz), 2.0) * point_light.intensity;

		color = color + diffuse * diffuse_strength * in.color;
	}

	return vec4<f32>(color, 1.0);
}