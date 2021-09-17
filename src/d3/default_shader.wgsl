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
	position: vec4<f32>;
	color: vec4<f32>;
	params: vec4<f32>;
};


let max_point_lights: u32 = 64u32;

[[block]]
struct Uniforms {
	view_proj: mat4x4<f32>;
	camera_position: vec3<f32>;
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
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>; 
	[[location(2)]] color: vec3<f32>;
};

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
	return f0 + (f90 - f0) * pow5(1.0 - voh);
}

fn f_schlick(f0: f32, f90: f32, voh: f32) -> f32 {
	return f0 + (f90 - f0) * pow5(1.0 - voh);
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
 
[[stage(fragment)]]
fn main(in: FragmentInput) -> [[location(0)]] vec4<f32> {
	let base_color = vec3<f32>(0.5);

	let perceptual_roughness = 0.089;
	let metallic = 0.01;
	let reflectance = 0.5;
	let emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
	let occlusion = 1.0;

	let roughness = perceptual_roughness_to_roughness(perceptual_roughness);

	var n: vec3<f32>;

	if (in.front) {
		n = normalize(in.normal);
	} else {
		n = normalize(-in.normal); 
	}

	var v: vec3<f32>;

	if (uniforms.view_proj[3][3] != 1.0) {
		v = normalize(uniforms.camera_position - in.position);
	} else {
		v = normalize(vec3<f32>(-uniforms.view_proj[0][2], -uniforms.view_proj[1][2], -uniforms.view_proj[2][2]));
	} 

	let ndotv = max(dot(n, v), 0.0001);

	let f0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + base_color * metallic;

	let diffuse_color = base_color * (1.0 - metallic);

	let r = reflect(-v, n);

	var color: vec3<f32> = vec3<f32>(0.0);	

	for (var i: u32 = 0u32; i < uniforms.point_light_count; i = i + 1u32) {
		color = color + point_light(uniforms.point_lights[i], in.position, roughness, ndotv, n, v, r, f0, diffuse_color);
	}

	let diffuse_ambient = env_brdf_approx(diffuse_color, 1.0, ndotv);
	let specular_ambient = env_brdf_approx(f0, perceptual_roughness, ndotv);

	color = color + (diffuse_ambient + specular_ambient) * vec3<f32>(0.05) * occlusion;
	color = color + emissive.rgb;

	color = reinhard_luminance(color);

	return vec4<f32>(color, 1.0);
}