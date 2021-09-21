[[group(0), binding(0)]]
var eq_texture: texture_2d<f32>;

[[group(1), binding(0)]]
var cube_texture: texture_storage_2d_array<rgba32float, write>;

[[block]]
struct Uniforms {
	offset: u32;
};

[[group(1), binding(1)]]
var<uniform> uniforms: Uniforms;

let PI: f32 = 3.141592653589793;
let INV_ATAN: vec2<f32> = vec2<f32>(0.1591, 0.3183);

[[stage(compute), workgroup_size(32, 32, 1)]]
fn main([[builtin(global_invocation_id)]] param: vec3<u32>) {
	let cube_size = vec2<f32>(textureDimensions(cube_texture));
	let texture_size = vec2<f32>(textureDimensions(eq_texture));

	let uv = vec2<f32>(param.xy) / cube_size * 2.0 - 1.0;

	var dir: vec3<f32>;
	
	switch (i32(param.z + uniforms.offset)) {
		case 0: {
			dir = vec3<f32>(1.0, uv.y, -uv.x);
		}
		case 1: {
			dir = vec3<f32>(-1.0, uv.y, uv.x);
		}
		case 2: {
			dir = vec3<f32>(uv.x, -1.0, uv.y);
		}
		case 3: {
			dir = vec3<f32>(uv.x, 1.0, -uv.y);
		}
		case 4: {
			dir = vec3<f32>(uv.x, uv.y, 1.0);
		}
		case 5: {
			dir = vec3<f32>(-uv.x, uv.y, -1.0);
		}
	} 

	dir = normalize(dir);
	let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), dir));
	let up = normalize(cross(dir, right));

	var irradiance = vec3<f32>(0.0);

	let sample_delta = 0.025;
	var nr_samples = 0.0;

	for (var phi = 0.0; phi < 2.0 * PI; phi = phi + sample_delta) {
		for (var theta = 0.0; theta < 0.5 * PI; theta = theta + sample_delta) {
			let sin_theta = sin(theta);
			let cos_theta = cos(theta);
			let tangent_sample = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);

			let sample_vec = tangent_sample.x * right + tangent_sample.y * up + tangent_sample.z * dir;

			let uv = vec2<f32>(atan2(sample_vec.z, sample_vec.x), asin(sample_vec.y)) * INV_ATAN + 0.5;
				
			let color = textureLoad(eq_texture, vec2<i32>(uv * texture_size), 0);

			irradiance = irradiance + color.rgb * cos_theta * sin_theta;
			nr_samples = nr_samples + 1.0;
		}
	}

	irradiance = PI * irradiance * (1.0 / nr_samples);

	textureStore(cube_texture, vec2<i32>(param.xy), i32(param.z), vec4<f32>(irradiance, 1.0));
}

