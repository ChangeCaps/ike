struct VertexInput {
	[[location(0)]]
	position: vec3<f32>;
	[[location(1)]]
	normal: vec3<f32>;
	[[location(2)]]
	uv: vec2<f32>;
};

struct VertexOutput {
	[[builtin(position)]]
	position: vec4<f32>;
	[[location(0)]]
	w_position: vec3<f32>;
	[[location(1)]]
	w_normal: vec3<f32>;
	[[location(2)]]
	uv: vec2<f32>;
};

struct Mesh {
	transform: mat4x4<f32>;
	view_proj: mat4x4<f32>;	
	camera_position: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> mesh: Mesh;

struct Material {
	base_color: vec4<f32>;
	metallic: f32;
	roughness: f32;
	reflectance: f32;
	emission: vec4<f32>;
};

[[group(1), binding(0)]]
var<uniform> material: Material;

[[group(1), binding(1)]]
var base_color_texture: texture_2d<f32>;
[[group(1), binding(2)]]
var base_color_sampler: sampler;

[[group(1), binding(1)]]
var metallic_roughness_texture: texture_2d<f32>;
[[group(1), binding(2)]]
var metallic_roughness_sampler: sampler;

[[group(1), binding(1)]]
var emission_texture: texture_2d<f32>;
[[group(1), binding(2)]]
var emission_sampler: sampler;

[[group(1), binding(1)]]
var normal_map_texture: texture_2d<f32>;
[[group(1), binding(2)]]
var normal_map_sampler: sampler;

let MAX_DIRECTIONAL_LIGHTS = 8;

struct DirectionalLight {
	view_proj: mat4x4<f32>;
	color: vec4<f32>;
	dir_to_light: vec3<f32>;
};

struct Lights {
	directional_light_count: u32;
	directional_lights: array<DirectionalLight, MAX_DIRECTIONAL_LIGHTS>;
};

[[group(2), binding(0)]]
var<uniform> lights: Lights;

[[group(2), binding(1)]]
var d_shadow_maps: texture_2d_array<f32>;

[[group(3), binding(2)]]
var shadow_map_sampler: sampler_comparison;

[[stage(vertex)]]
fn vert(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;	

	let position = mesh.transform * vec4<f32>(in.position, 1.0);
	out.position = mesh.view_proj * position;
	out.w_position = position.xyz;

	let normal = mesh.transform * vec4<f32>(in.normal, 0.0);
	out.w_normal = normal.xyz;

	out.uv = in.uv;

	return out;
}

struct Material {
	base_color: vec4<f32>;
};

struct FragmentInput {
	[[builtin(front_facing)]]
	front_facing: bool;
	[[location(0)]]
	w_position: vec3<f32>;
	[[location(1)]]
	w_normal: vec3<f32>;
	[[location(2)]]
	uv: vec2<f32>;
};

let PI = 3.1415926535;

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
	let a = roughness * roughness;
	let a2 = a * a;
	let n_dot_h2 = n_dot_h * n_dot_h;

	var denom = (n_dot_h2 * (a2 - 1.0) + 1.0);
	denom = PI * denom * denom;

	return a2 / denom;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
	let r = (roughness + 1.0);
	let k = (r * r) / 8.0;

	let denom = n_dot_v * (1.0 - k) + k;

	return n_dot_v / denom;
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
	let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
	let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);	

	return ggx1 * ggx2;
}

fn fresnel_schlick(n_dot_h: f32, f0: vec3<f32>) -> vec3<f32> {
	return f0 + (1.0 - f0) * pow(clamp(1.0 - n_dot_h, 0.0, 1.0), 5.0);
}

fn directional_light(
	index: u32, 
	n: vec3<f32>, 
	v: vec3<f32>, 
	f0: vec3<f32>, 
	base_color: vec3<f32>,
	metallic: f32, 
	roughness: f32
) -> vec3<f32> {
	let light = lights.directional_lights[index];

	let l = light.dir_to_light;
	let h = normalize(v + l);
	
	let n_dot_v = max(dot(n, v), 0.0);
	let n_dot_h = max(dot(n, h), 0.0);
	let n_dot_l = max(dot(n, l), 0.0);

	let ndf = distribution_ggx(n_dot_h, roughness);
	let g = geometry_smith(n_dot_v, n_dot_l, roughness);
	let f = fresnel_schlick(n_dot_h, f0);

	let numerator = ndf * g * f;
	let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
	let specular = numerator / denominator;

	let kd = (vec3<f32>(1.0) - f) * (1.0 - metallic);

	let diffuse = light.color.rgb * n_dot_l;

	return (kd * base_color / PI + specular) * diffuse;
}

[[stage(fragment)]]
fn frag(in: FragmentInput) -> [[location(0)]] vec4<f32> {
	let base_color = textureSample(base_color_texture, base_color_sampler, in.uv);
	var color = material.base_color.rgb * base_color.rgb;

	var roughness = material.roughness;
	var metallic = material.metallic;

	var f0 = vec3<f32>(0.04);
	f0 = mix(f0, color, metallic);

	var n = in.w_normal;
	let v = normalize(mesh.camera_position - in.w_position);

	if (!in.front_facing) {
		n = -n;
	}

	var light = vec3<f32>(0.0);

	for (var i = 0u; i < lights.directional_light_count; i = i + 1u) {
		light = light + directional_light(i, n, v, f0, color, roughness, metallic);
	}

	color = color / (color + 1.0);
	color = pow(color, vec3<f32>(1.0 / 2.2));

	return vec4<f32>(color * light, 1.0);
}
