[[group(0), binding(0)]]
var org: texture_2d<f32>;

[[group(0), binding(1)]]
var up: texture_2d<f32>;

[[group(0), binding(2)]]
var target: texture_storage_2d<rgba32float, write>;

fn sample(uv: vec2<i32>) -> vec4<f32> {
	return textureLoad(up, uv, 0);
}

[[stage(compute), workgroup_size(8, 8, 1)]]
fn main([[builtin(global_invocation_id)]] param: vec3<u32>) {
	let d = vec4<i32>(1, 1, -1, 0);

	let org_uv = vec2<i32>(param.xy);
	let uv = org_uv / 2;

	var s: vec4<f32> = sample(uv - d.xy);
	s = s + sample(uv - d.wy) * 2.0;
	s = s + sample(uv - d.zy);

	s = s + sample(uv + d.zw) * 2.0;
	s = s + sample(uv       ) * 4.0;
	s = s + sample(uv + d.xw) * 2.0;

	s = s + sample(uv + d.zy);
	s = s + sample(uv + d.wy) * 2.0;
	s = s + sample(uv + d.xy);

	let color = textureLoad(org, org_uv, 0) + s / 16.0;

	textureStore(target, org_uv, color);
}