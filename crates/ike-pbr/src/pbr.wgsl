let NORMAL_MAP_FLAG_BIT: u32 = 1u;

struct VertexInput {
	[[builtin(instance_index)]] index: u32;

	// vertex
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] uv: vec2<f32>;
	[[location(3)]] tangent: vec4<f32>;
	[[location(4)]] color: vec4<f32>;

// #ifdef SKINNED
	[[location(5)]] joints: vec4<u32>;
	[[location(6)]] weights: vec4<f32>;
// #endif

	// instance
	[[location(8)]] transform_0: vec4<f32>;
	[[location(9)]] transform_1: vec4<f32>;
	[[location(10)]] transform_2: vec4<f32>;
	[[location(11)]] transform_3: vec4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] w_position: vec4<f32>;
	[[location(1)]] w_normal: vec3<f32>;
	[[location(2)]] w_tangent: vec4<f32>;
	[[location(3)]] uv: vec2<f32>;
	[[location(4)]] color: vec4<f32>;
};

struct PointLight {
	position: vec4<f32>;
	color: vec4<f32>;
	params: vec4<f32>;
};

let MAX_POINT_LIGHTS: u32 = 64u;

struct DirectionalLight {
	position: vec3<f32>;
	direction: vec3<f32>;
	color: vec4<f32>;
	view_proj: mat4x4<f32>;
	near: f32;
	far: f32;
	size: vec2<f32>; 
};

let MAX_DIRECTIONAL_LIGHTS: u32 = 16u;

[[block]]
struct Uniforms {
	view_proj: mat4x4<f32>;
	camera_position: vec3<f32>;
	point_lights: array<PointLight, MAX_POINT_LIGHTS>;
	directional_lights: array<DirectionalLight, MAX_DIRECTIONAL_LIGHTS>;
	point_light_count: u32;
	directional_light_count: u32;
}; 

[[group(0), binding(0)]] 
var<uniform> uniforms: Uniforms;

[[group(0), binding(1)]]
var env_texture: texture_cube<f32>;

[[group(0), binding(2)]]
var irradiance_texture: texture_cube<f32>;

[[group(0), binding(3)]]
var env_sampler: sampler;

// [[block]]
// struct JointMatrices {
// 	matrices: array<mat4x4<f32>>; 
// };
// 
// [[group(2), binding(0)]]
// var<storage, read> joint_matrices: JointMatrices;

struct Material {
	albedo: vec4<f32>;
	emission: vec4<f32>;
	roughness: f32;
	metallic: f32;
	reflectance: f32;
	shadow_softness: f32;
	shadow_softness_falloff: f32;
	shadow_blocker_samples: u32;
	shadow_pcf_samples: u32;
};

[[block]]
struct Mesh {
	material: Material;
	flags: u32;
	joint_count: u32;
};

[[group(1), binding(0)]]
var sampler: sampler;

[[group(1), binding(1)]]
var albedo_texture: texture_2d<f32>;

[[group(1), binding(2)]]
var metallic_roughness_texture: texture_2d<f32>;

[[group(1), binding(3)]]
var normal_map: texture_2d<f32>;

[[group(1), binding(4)]]
var<uniform> mesh: Mesh;

[[group(3), binding(0)]]
var directional_light_shadow_texture: texture_depth_2d_array;

[[group(3), binding(1)]]
var light_sampler: sampler;

// since Scalar * Matrix isn't implemented this is necessary
fn mul_mat(weight: f32, joint: mat4x4<f32>) -> mat4x4<f32> {
	return mat4x4<f32>(
		vec4<f32>(weight) * joint.x,
		vec4<f32>(weight) * joint.y,
		vec4<f32>(weight) * joint.z,
		vec4<f32>(weight) * joint.w,
	); 
}

// since Matrix + Matrix isn't implemented this is necessary
fn add_mat(lhs: mat4x4<f32>, rhs: mat4x4<f32>) -> mat4x4<f32> {
	return mat4x4<f32>(
		lhs.x + rhs.x,
		lhs.y + rhs.y,
		lhs.z + rhs.z,
		lhs.w + rhs.w,
	);
}

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	let transform = mat4x4<f32>(
		in.transform_0,
		in.transform_1,
		in.transform_2,
		in.transform_3
	);

	let transform3x3 = mat3x3<f32>(
		transform.x.xyz,
		transform.y.xyz,
		transform.z.xyz,
	);

	
	out.w_position = transform * vec4<f32>(in.position, 1.0);
	out.w_normal = transform3x3 * in.normal.xyz;
	out.w_tangent = vec4<f32>(transform3x3 * in.tangent.xyz, in.tangent.w);

	
	//	let joint_offset = in.index * mesh.joint_count;

	//	let x = mul_mat(in.weights.x, joint_matrices.matrices[joint_offset + in.joints.x]);
	//	let y = mul_mat(in.weights.y, joint_matrices.matrices[joint_offset + in.joints.y]);
	//	let z = mul_mat(in.weights.z, joint_matrices.matrices[joint_offset + in.joints.z]);
	//	let w = mul_mat(in.weights.w, joint_matrices.matrices[joint_offset + in.joints.w]);

	//	let skin = add_mat(add_mat(add_mat(x, y), z), w);

	//	let skin3x3 = mat3x3<f32>(
	//		skin.x.xyz,
	//		skin.y.xyz,
	//		skin.z.xyz,
	//	);

	//	out.w_position = transform * skin * vec4<f32>(in.position, 1.0);	
	//	out.w_normal = transform3x3 * skin3x3 * in.normal.xyz;
	//	out.w_tangent = vec4<f32>(transform3x3 * skin3x3 * in.tangent.xyz, in.tangent.w);
	
	

	out.position = uniforms.view_proj * out.w_position;	
	out.uv = in.uv;
	out.color = in.color * mesh.material.albedo;

	return out;
}

struct FragmentInput {
	[[builtin(front_facing)]] front: bool;
	[[location(0)]] position: vec4<f32>;
	[[location(1)]] normal: vec3<f32>; 
	[[location(2)]] tangent: vec4<f32>; 
	[[location(3)]] uv: vec2<f32>;
	[[location(4)]] color: vec4<f32>;
};

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

let PI: f32 = 3.141592653589793;

fn pow5(x: f32) -> f32 {
	let x2 = x * x;
	return x2 * x2 * x;
}

fn saturate(x: f32) -> f32 {
	return clamp(x, 0.0, 1.0);
}

fn distance_attenuation(distance_squared: f32, inverse_range_squared: f32) -> f32 {
	let factor = distance_squared * inverse_range_squared;
	let smooth_factor = saturate(1.0 - factor * factor);
	let attenuation = smooth_factor * smooth_factor;
	return attenuation * 1.0 / max(distance_squared, 0.0001); 
}

fn d_ggx(roughness: f32, noh: f32) -> f32 {
	let x = 1.0 - noh * noh; 
	let a = noh * roughness;
	let k = roughness / (x + a * a);
	return k * k * (1.0 / PI);
}

fn v_smith_ggx_correlated(roughness: f32, nov: f32, nol: f32) -> f32 {
	let a2 = roughness * roughness;
	let lv = nol * sqrt((nov - a2 * nov) * nov + a2);
	let ll = nov * sqrt((nol - a2 * nol) * nol + a2);
	return 0.5 / (lv + ll);
}

fn f_schlick3(f0: vec3<f32>, f90: f32, voh: f32) -> vec3<f32> {
	return f0 + (f90 - f0) * pow(1.0 - voh, 5.0);
}

fn f_schlick(f0: f32, f90: f32, voh: f32) -> f32 {
	return f0 + (f90 - f0) * pow(1.0 - voh, 5.0);
}

fn f_schlick_roughness(f0: vec3<f32>, ndotv: f32, roughness: f32) -> vec3<f32> {
	return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(saturate(1.0 - ndotv), 5.0);
}

fn fresnel(f0: vec3<f32>, loh: f32) -> vec3<f32> {
	let f90 = saturate(dot(f0, vec3<f32>(50.0 * 0.33)));
	return f_schlick3(f0, f90, loh);
}

fn specular(
	f0: vec3<f32>, 
	roughness: f32, 
	nov: f32, 
	nol: f32, 
	noh: f32, 
	loh: f32, 
	specular_intensity: f32,
) -> vec3<f32> {
	let d = d_ggx(roughness, noh);
	let v = v_smith_ggx_correlated(roughness, nov, nol);
	let f = fresnel(f0, loh); 

	return (specular_intensity * d * v) * f;
} 

fn fd_burley(roughness: f32, nov: f32, nol: f32, loh: f32) -> f32 {
	let f90 = 0.5 + 2.0 * roughness * loh * loh;
	let light_scatter = f_schlick(1.0, f90, nol);
	let view_scatter = f_schlick(1.0, f90, nov);
	return light_scatter * view_scatter * (1.0 / PI);
}

let c0: vec4<f32> = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
let c1: vec4<f32> = vec4<f32>(1.0, 0.0425, 1.04, -0.04);

fn env_brdf_approx(f0: vec3<f32>, perceptual_roughness: f32, nov: f32) -> vec3<f32> {
	let r = perceptual_roughness * c0 + c1;
	let a004 = min(r.x * r.x, exp2(-9.28 * nov)) * r.x + r.y;
	let ab = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
	return f0 * ab.x + ab.y;
}

fn perceptual_roughness_to_roughness(perceptual_roughness: f32) -> f32 {
	let c = clamp(perceptual_roughness, 0.089, 1.0);
	return c * c;
}

fn reinhard(color: vec3<f32>) -> vec3<f32> {
	return color / (1.0 + color);
}

fn reinhard_extended(color: vec3<f32>, max_white: f32) -> vec3<f32> {
	let num = color * (1.0 + (color / vec3<f32>(max_white * max_white)));
	return num / (1.0 + color);
}

fn luminance(v: vec3<f32>) -> f32 {
	return dot(v, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn change_luminance(c_in: vec3<f32>, l_out: f32) -> vec3<f32> {
	let l_in = luminance(c_in);
	return c_in * (l_out / l_in);
}

fn reinhard_luminance(color: vec3<f32>) -> vec3<f32> {
	let l_old = luminance(color);
	let l_new = l_old / (1.0 + l_old);
	return change_luminance(color, l_new);
}

fn reinhard_extended_luminance(color: vec3<f32>, max_white: f32) -> vec3<f32> {
	let l_old = luminance(color);
	let num = l_old * (1.0 + (l_old / (max_white * max_white)));
	let l_new = num / (1.0 + l_old);
	return change_luminance(color, l_new);
}


fn noise(pos: vec3<f32>) -> f32 {
	let s = pos + 0.2127 + pos.x * pos.y * pos.z * 0.3713;
	let r = 4.789 * sin(489.123 * (s));
	return fract(r.x * r.y * r.z * (1.0 + s.x));
}

fn rotate(pos: vec2<f32>, trig: vec2<f32>) -> vec2<f32> {
	return vec2<f32>(
		pos.x * trig.x - pos.y * trig.y,
		pos.y * trig.x + pos.x * trig.y,
	);
}

fn point_light(
	light: PointLight, 
	position: vec3<f32>,
	roughness: f32, 
	ndotv: f32, 
	n: vec3<f32>, 
	v: vec3<f32>, 
	r: vec3<f32>, 
	f0: vec3<f32>, 
	diffuse_color: vec3<f32>
) -> vec3<f32> {
	let light_to_frag = light.position.xyz - position;
	let distance_square = dot(light_to_frag, light_to_frag);
	let range_attenuation = distance_attenuation(distance_square, light.params.r);

	let a = roughness;
	let radius = light.params.g;
	let center_to_ray = dot(light_to_frag, r) * r - light_to_frag;
	let closest_point = light_to_frag + center_to_ray * saturate(radius * inverseSqrt(dot(center_to_ray, center_to_ray)));
	let l_spec_length_inverse = inverseSqrt(dot(closest_point, closest_point));
	let normalization_factor = a / saturate(a + (radius * 0.5 * l_spec_length_inverse));
	let specular_intensity = normalization_factor * normalization_factor;

	let l = closest_point * l_spec_length_inverse;
	let h = normalize(l + v);
	let nol = saturate(dot(n, l));
	let noh = saturate(dot(n, h));
	let loh = saturate(dot(l, h));

	let specular = specular(f0, roughness, ndotv, nol, noh, loh, specular_intensity);

	let l = normalize(light_to_frag);
	let h = normalize(l + v);
	let nol = saturate(dot(n, l));
	let noh = saturate(dot(n, h));
	let loh = saturate(dot(l, h));

	let diffuse = diffuse_color * fd_burley(roughness, ndotv, nol, loh);

	return ((diffuse + specular) * light.color.rgb) * (range_attenuation * nol);
}

fn directional_light(
	light: DirectionalLight,
	roughness: f32,
	ndotv: f32,
	normal: vec3<f32>,
	view: vec3<f32>,
	r: vec3<f32>,
	f0: vec3<f32>,
	diffuse_color: vec3<f32>,
) -> vec3<f32> {
	let incident_light = -light.direction;

	let half = normalize(incident_light + view);
	let nol = saturate(dot(normal, incident_light));
	let noh = saturate(dot(normal, half));
	let loh = saturate(dot(incident_light, half));

	let diffuse = diffuse_color * fd_burley(roughness, ndotv, nol, loh);
	let specular_intensity = 1.0;
	let specular_light = specular(f0, roughness, ndotv, nol, noh, loh, specular_intensity);

	return (specular_light + diffuse) * light.color.rgb * nol;
}

fn search_region_radius_uv(light: DirectionalLight, z: f32) -> vec2<f32> {
	return mesh.material.shadow_softness / light.size * 0.15; 
}

fn penumbra_radius_uv(z_receiver: f32, z_blocker: f32) -> f32 {
	return z_receiver - z_blocker;
}

fn z_clip_to_eye(light: DirectionalLight, z: f32) -> f32 {
	return light.near - (light.far - light.near) * z;
}

fn find_blocker(
	idx: u32, 
	uv: vec2<f32>, 
	z0: f32,
	bias: f32,
	radius: vec2<f32>,
	trig: vec2<f32>,
) -> vec2<f32> {
	var blocker_sum: f32 = 0.0;
	var num_blockers: f32 = 0.0;

	let biased_depth = z0 - bias;

	for (var i: u32 = 0u; i < mesh.material.shadow_blocker_samples; i = i + 1u) {
		let offset = poisson_offsets[i] * radius;

		let offset_uv = uv + rotate(offset, trig);

		if (any(offset_uv < vec2<f32>(0.0)) || any(offset_uv > vec2<f32>(1.0))) {
			continue;
		}

		let depth = textureSample(
			directional_light_shadow_texture, 
			light_sampler, 
			offset_uv,
			i32(idx)
		);

		if (depth < biased_depth) {
			blocker_sum = blocker_sum + depth;
			num_blockers = num_blockers + 1.0;
		}	
	}

	let avg_blocker_depth = blocker_sum / num_blockers;

	return vec2<f32>(avg_blocker_depth, num_blockers);
}

fn pcf_filter(idx: u32, uv: vec2<f32>, z0: f32, bias: f32, filter_radius: vec2<f32>, trig: vec2<f32>) -> f32 {
	var sum: f32 = 0.0; 

	let biased_depth = z0 - bias;

	for (var i: u32 = 0u; i < mesh.material.shadow_pcf_samples; i = i + 1u) {
		let offset = poisson_offsets[i] * filter_radius;

		let offset_uv = uv + rotate(offset, trig);

		if (any(offset_uv < vec2<f32>(0.0)) || any(offset_uv > vec2<f32>(1.0))) {
			sum = sum + 1.0;
			continue;
		}

		let depth = textureSample(
			directional_light_shadow_texture,
			light_sampler,
			offset_uv,
			i32(idx)
		);

		if (biased_depth <= depth) {
			sum = sum + 1.0;
		}	
	}

	sum = sum / f32(mesh.material.shadow_pcf_samples);

	return sum;
}

fn pcss_filter(idx: u32, light: DirectionalLight, uv: vec2<f32>, z: f32, bias: f32, z_vs: f32, trig: vec2<f32>) -> f32 {
	let search_radius = search_region_radius_uv(light, z_vs) * mesh.material.shadow_softness;
	let blocker = find_blocker(idx, uv, z, bias, search_radius, trig);

	if (blocker.y < 1.0) {
		return 1.0;
	}

	let avg_blocker_depth_vs = z_clip_to_eye(light, blocker.x);
	let penumbra = penumbra_radius_uv(z_vs, avg_blocker_depth_vs) * 0.005 * mesh.material.shadow_softness;
	let penumbra = 1.0 - pow(1.0 - penumbra, mesh.material.shadow_softness_falloff);
	let filter_radius = (penumbra - 0.015 * mesh.material.shadow_softness) / light.size;

	return pcf_filter(idx, uv, z, bias, filter_radius, trig);
}

fn directional_shadow(idx: u32, position: vec4<f32>, normal: vec3<f32>) -> f32 {
	let light = uniforms.directional_lights[idx]; 

	let direction_to_light = -light.direction;

	let light_space_pos = light.view_proj * position;

	if (light_space_pos.w <= 0.0) {
		return 1.0;
	}

	let proj_coords = light_space_pos.xyz / light_space_pos.w;

	if (proj_coords.z < 0.0) {
		return 1.0;
	}

	let flip_correction = vec2<f32>(0.5, -0.5);

	let uv = proj_coords.xy * flip_correction + 0.5;

	let z = proj_coords.z;
	let z_vs = z_clip_to_eye(light, proj_coords.z);

	let n = noise(position.xyz);
	let angle = n * 2.0 * PI;

	let trig = vec2<f32>(cos(angle), sin(angle));

	return pcss_filter(idx, light, uv, z, 0.00005, z_vs, trig);
}
 
[[stage(fragment)]]
fn main(in: FragmentInput) -> [[location(0)]] vec4<f32> {
	let base_color = in.color * textureSample(albedo_texture, sampler, in.uv);

	let metallic_roughness = textureSample(metallic_roughness_texture, sampler, in.uv);

	let perceptual_roughness = mesh.material.roughness;
	let metallic = mesh.material.metallic * metallic_roughness.b;
	let reflectance = mesh.material.reflectance;
	let emissive = mesh.material.emission;
	let occlusion = 1.0;

	let roughness = perceptual_roughness_to_roughness(perceptual_roughness * metallic_roughness.g);

	var n: vec3<f32> = normalize(in.normal);
	var t: vec3<f32> = normalize(in.tangent.xyz);
	var b: vec3<f32> = cross(n, t) * in.tangent.w;

	if (!in.front) {	
		n = -n; 
		t = -t;
		b = -b;
	}

	let tbn = mat3x3<f32>(t, b, n);

	let nm = normalize(textureSample(normal_map, sampler, in.uv).rgb * 2.0 - 1.0);

	if ((mesh.flags & NORMAL_MAP_FLAG_BIT) != 0u) { 
		n = tbn * nm;
	}

	var v: vec3<f32>;

	if (uniforms.view_proj[3][3] != 1.0) {
		v = normalize(uniforms.camera_position - in.position.xyz);
	} else {
		v = normalize(vec3<f32>(-uniforms.view_proj[0][2], -uniforms.view_proj[1][2], -uniforms.view_proj[2][2]));
	} 

	let ndotv = max(dot(n, v), 0.0001);

	let f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + base_color.rgb * metallic;

	let r = reflect(-v, n);

	let env = textureSample(env_texture, env_sampler, r).rgb;
	let irradiance = textureSample(irradiance_texture, env_sampler, n).rbg;

	let diffuse_color = base_color.rgb * (1.0 - metallic) + env * metallic;

	var color: vec3<f32> = vec3<f32>(0.0);	

	for (var i: u32 = 0u; i < uniforms.point_light_count; i = i + 1u) {
		color = color + point_light(uniforms.point_lights[i], in.position.xyz, roughness, ndotv, n, v, r, f0, diffuse_color);
	}

	for (var i: u32 = 0u; i < uniforms.directional_light_count; i = i + 1u) {
		let light = uniforms.directional_lights[i];

		let shadow = directional_shadow(i, in.position, n); 

		if (shadow > 0.0) {
			let l = directional_light(uniforms.directional_lights[i], roughness, ndotv, n, v, r, f0, diffuse_color);
			color = color + l * shadow;
		}
	}

	let ks = f_schlick_roughness(f0, ndotv, roughness);  
	let kd = 1.0 - ks;
	let diffuse = irradiance * diffuse_color;

	let diffuse_ambient = kd * diffuse;

	color = color + diffuse_ambient * occlusion * 0.1;
	color = color + emissive.rgb;

	if (base_color.a < 0.1) {
		discard;
	} else {
		return vec4<f32>(color, 1.0);
	}
}