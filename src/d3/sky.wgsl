var verts: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
	vec2<f32>(-1.0, -1.0),
	vec2<f32>(3.0, -1.0),
	vec2<f32>(-1.0, 3.0),
);

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] view_dir: vec3<f32>;
};

[[block]]
struct Uniforms {
	view_to_world: mat4x4<f32>;
	clip_to_view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[group(0), binding(1)]]
var texture: texture_2d<f32>;

[[group(0), binding(2)]]
var sampler: sampler;

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] idx: u32) -> VertexOutput {
	var out: VertexOutput;

	let position_clip = vec4<f32>(verts[idx], 0.0, 1.0);

	var position_view = uniforms.clip_to_view * position_clip;
	position_view.w = 0.0;	
	let position_world = uniforms.view_to_world * position_view;

	out.position = position_clip;
	out.view_dir = position_world.xyz;

	return out;
}

let PI: f32 = 3.141592653589793;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	let view_dir = normalize(in.view_dir);

	let x = atan2(view_dir.z, view_dir.x);	
	let y = acos(dot(vec3<f32>(0.0, -1.0, 0.0), view_dir));

	let u = x / PI / 2.0 + 0.5;
	let v = y / PI;

	let uv = vec2<f32>(u, v);

	return textureSample(texture, sampler, uv);
}