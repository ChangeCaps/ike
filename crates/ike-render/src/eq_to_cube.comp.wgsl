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

	let uv = vec2<f32>(atan2(dir.z, dir.x), asin(dir.y)) * INV_ATAN + 0.5;

	let color = textureLoad(eq_texture, vec2<i32>(uv * texture_size), 0);
	textureStore(cube_texture, vec2<i32>(param.xy), i32(param.z), color);
}
