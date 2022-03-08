var<private> poisson_offsets: array<vec2<f32>, 64> = array<vec2<f32>, 64>( 
	vec2<f32>(0.0617981, 0.07294159),
	vec2<f32>(0.6470215, 0.7474022), 
	vec2<f32>(-0.5987766, -0.7512833),
	vec2<f32>(-0.693034, 0.6913887),
	vec2<f32>(0.6987045, -0.6843052),
	vec2<f32>(-0.9402866, 0.04474335),
	vec2<f32>(0.8934509, 0.07369385),
	vec2<f32>(0.1592735, -0.9686295),
	vec2<f32>(-0.05664673, 0.995282),
	vec2<f32>(-0.1203411, -0.1301079),
	vec2<f32>(0.1741608, -0.1682285),
	vec2<f32>(-0.09369049, 0.3196758),
	vec2<f32>(0.185363, 0.3213367),
	vec2<f32>(-0.1493771, -0.3147511),
	vec2<f32>(0.4452095, 0.2580113),
	vec2<f32>(-0.1080467, -0.5329178),
	vec2<f32>(0.1604507, 0.5460774),
	vec2<f32>(-0.4037193, -0.2611179),
	vec2<f32>(0.5947998, -0.2146744),
	vec2<f32>(0.3276062, 0.9244621),
	vec2<f32>(-0.6518704, -0.2503952),
	vec2<f32>(-0.3580975, 0.2806469),
	vec2<f32>(0.8587891, 0.4838005),
	vec2<f32>(-0.1596546, -0.8791054),
	vec2<f32>(-0.3096867, 0.5588146),
	vec2<f32>(-0.5128918, 0.1448544),
	vec2<f32>(0.8581337, -0.424046),
	vec2<f32>(0.1562584, -0.5610626),
	vec2<f32>(-0.7647934, 0.2709858),
	vec2<f32>(-0.3090832, 0.9020988),
	vec2<f32>(0.3935608, 0.4609676),
	vec2<f32>(0.3929337, -0.5010948),
	vec2<f32>(-0.8682281, -0.1990303),
	vec2<f32>(-0.01973724, 0.6478714),
	vec2<f32>(-0.3897587, -0.4665619),
	vec2<f32>(-0.7416366, -0.4377831),
	vec2<f32>(-0.5523247, 0.4272514),
	vec2<f32>(-0.5325066, 0.8410385),
	vec2<f32>(0.3085465, -0.7842533),
	vec2<f32>(0.8400612, -0.200119),
	vec2<f32>(0.6632416, 0.3067062),
	vec2<f32>(-0.4462856, -0.04265022),
	vec2<f32>(0.06892014, 0.812484),
	vec2<f32>(0.5149567, -0.7502338),
	vec2<f32>(0.6464897, -0.4666451),
	vec2<f32>(-0.159861, 0.1038342),
	vec2<f32>(0.6455986, 0.04419327),
	vec2<f32>(-0.7445076, 0.5035095),
	vec2<f32>(0.9430245, 0.3139912),
	vec2<f32>(0.0349884, -0.7968109),
	vec2<f32>(-0.9517487, 0.2963554),
	vec2<f32>(-0.7304786, -0.01006928),
	vec2<f32>(-0.5862702, -0.5531025),
	vec2<f32>(0.3029106, 0.09497032),
	vec2<f32>(0.09025345, -0.3503742),
	vec2<f32>(0.4356628, -0.0710125),
	vec2<f32>(0.4112572, 0.7500054),
	vec2<f32>(0.3401214, -0.3047142),
	vec2<f32>(-0.2192158, -0.6911137),
	vec2<f32>(-0.4676369, 0.6570358),
	vec2<f32>(0.6295372, 0.5629555),
	vec2<f32>(0.1253822, 0.9892166),
	vec2<f32>(-0.1154335, 0.8248222),
	vec2<f32>(-0.4230408, -0.7129914),
);

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
	size: vec2<f32>;
	near: f32;
	far: f32;
	shadow_softness: f32;
	shadow_falloff: f32;
	blocker_samples: u32;
	pcf_samples: u32;
};

struct Lights {
	directional_light_count: u32;
	directional_lights: array<DirectionalLight, MAX_DIRECTIONAL_LIGHTS>;
};

[[group(2), binding(0)]]
var<uniform> lights: Lights;

[[group(2), binding(1)]]
var d_shadow_maps: texture_depth_2d_array;

[[group(2), binding(2)]]
var shadow_map_sampler: sampler;

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
	metallic: f32;
	roughness: f32;
	reflectance: f32;
	emission: vec4<f32>;
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

fn rotate(pos: vec2<f32>, trig: vec2<f32>) -> vec2<f32> {
	return vec2<f32>(
		pos.x * trig.x - pos.y * trig.y,
		pos.y * trig.x + pos.x * trig.y,
	);
}

fn is_uv_invalid(uv: vec2<f32>) -> bool {
	return any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0));
}

fn noise(pos: vec3<f32>) -> f32 {
	let s = pos + 0.2127 + pos.x * pos.y * pos.z * 0.3713;
	let r = 4.789 * sin(489.123 * (s));
	return fract(r.x * r.y * r.z * (1.0 + s.x));
}

fn penumbra_radius_uv(z_receiver: f32, z_blocker: f32) -> f32 {
	return z_receiver - z_blocker;
}

fn z_clip_to_eye(z: f32, near: f32, far: f32) -> f32 {
	return near - (far - near) * z;
}

fn find_blocker(
	shadow_maps: texture_depth_2d_array,
	map_index: i32,
	shadow_uv: vec2<f32>,
	z0: f32,
	bias: f32,
	radius: vec2<f32>,
	trig: vec2<f32>,
	samples: u32,
) -> vec2<f32> {
	var blocker_sum = 0.0;
	var num_blockers = 0.0;

	let biased_depth = z0; 

	for (var i = 0u; i < samples; i = i + 1u) {
		let offset = poisson_offsets[i] * radius;

		let offset_uv = shadow_uv + rotate(offset, trig);

		if (is_uv_invalid(offset_uv)) {
			continue;
		}

		let depth = textureSample(shadow_maps, shadow_map_sampler, offset_uv, map_index);

		if (depth < biased_depth) {
			blocker_sum = blocker_sum + depth;
			num_blockers = num_blockers + 1.0;
		}
	}

	let avg_blocker_depth = blocker_sum / num_blockers;

	return vec2<f32>(avg_blocker_depth, num_blockers);
}

fn pcf_filter(
	shadow_maps: texture_depth_2d_array,
	map_index: i32,
	shadow_uv: vec2<f32>,
	z0: f32,
	bias: f32,
	filter_radius: vec2<f32>,
	trig: vec2<f32>,
	samples: u32,
) -> f32 {
	var sum = 0.0;
	
	let biased_depth = z0 - bias;

	for (var i = 0u; i < samples; i = i + 1u) {
		let offset = poisson_offsets[i] * filter_radius;

		let offset_uv = shadow_uv + rotate(offset, trig);

		if (is_uv_invalid(offset_uv)) {
			sum = sum + 1.0;
			continue;
		}

		let depth = textureSample(shadow_maps, shadow_map_sampler, offset_uv, map_index);

		if (biased_depth <= depth) {
			sum = sum + 1.0;
		}
	}

	sum = sum / f32(samples);

	return sum;
}

fn d_pcss_filter(
	index: i32,
	shadow_uv: vec2<f32>,
	z: f32,
	bias: f32,
	z_vs: f32,
	trig: vec2<f32>,
) -> f32 {
	let light = lights.directional_lights[index];

	let search_radius = light.shadow_softness / light.size * 0.15;
	let blocker = find_blocker(
		d_shadow_maps, 
		index, 
		shadow_uv, 
		z, 
		bias, 
		search_radius, 
		trig, 
		light.blocker_samples
	);

	if (blocker.y < 1.0) {
		return 1.0;
	}

	let avg_blocker_depth_vs = z_clip_to_eye(blocker.x, light.near, light.far);
	let penumbra = penumbra_radius_uv(z_vs, avg_blocker_depth_vs) * 0.005 * light.shadow_softness;
	let penumbra = 1.0 - pow(1.0 - penumbra, light.shadow_falloff);
	let filter_radius = vec2<f32>((penumbra - 0.015 * light.shadow_softness)) / light.size;

	return pcf_filter(
		d_shadow_maps, 
		index, 
		shadow_uv, 
		z, 
		bias, 
		filter_radius, 
		trig, 
		light.pcf_samples
	);
}

fn d_shadow(
	index: i32,
	position: vec3<f32>,
	normal: vec3<f32>,
) -> f32 {
	let light = lights.directional_lights[index];

	let light_camera_space = light.view_proj * vec4<f32>(position, 1.0);

	if (light_camera_space.w <= 0.0) {
		return 1.0;
	}

	let light_space = light_camera_space.xyz / light_camera_space.w;

	if (light_space.z < 0.0) {
		return 1.0;
	}

	let flip_correction = vec2<f32>(0.5, -0.5);

	let shadow_uv = light_space.xy * flip_correction + 0.5;

	let z = light_space.z;
	let z_vs = z_clip_to_eye(z, light.near, light.far);

	let n = noise(position);
	let angle = n * 2.0 * PI;

	let trig = vec2<f32>(cos(angle), sin(angle));

	let bias_scale = 0.1;
	var bias = max(bias_scale * (1.0 - dot(normal, light.dir_to_light)), bias_scale * 0.01);
	bias = bias / (light.far - light.near);

	return d_pcss_filter(index, shadow_uv, z, bias, z_vs, trig);
}

fn directional_light(
	index: i32, 
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

	for (var i = 0; i < i32(lights.directional_light_count); i = i + 1) {	
		let shadow = d_shadow(i, in.w_position, n);

		light = light + directional_light(i, n, v, f0, color, roughness, metallic) * shadow;
	}

	var lit_color = color * light;
	lit_color = lit_color / (lit_color + 1.0);
	lit_color = pow(lit_color, vec3<f32>(1.0 / 2.2));

	return vec4<f32>(lit_color, 1.0);
}