[[group(0), binding(0)]]
var from: texture_2d<f32>;

[[group(0), binding(1)]]
var to: texture_storage_2d<rgba32float, write>;

[[block]]
struct Uniforms {
	pre_filter: i32;
	threshold: f32;
	knee: f32;
	scale: f32;
};

[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;

fn quadratic_threshold(color: vec4<f32>, threshold: f32, curve: vec3<f32>) -> vec4<f32> {
	let br = max(max(color.r, color.g), color.b);

	var rq = clamp(br - curve.x, 0.0, curve.y);
	rq = curve.z * rq * rq;

	return color * max(rq, br - threshold) / max(br, 0.0001); 
}

fn sample(uv: vec2<f32>) -> vec4<f32> {
	let base = vec2<i32>(floor(uv + 0.5));
	let f = fract(uv + 0.5);

	let a = textureLoad(from, base + vec2<i32>(0, 0), 0);
	let b = textureLoad(from, base + vec2<i32>(1, 0), 0);
	let c = textureLoad(from, base + vec2<i32>(0, 1), 0);
	let d = textureLoad(from, base + vec2<i32>(1, 1), 0);

	let a = a * (1.0 - f.x) + b * f.x;
	let b = c * (1.0 - f.x) + d * f.x;

	return a * (1.0 - f.y) + b * f.y;
}

[[stage(compute), workgroup_size(8, 8, 1)]]
fn main([[builtin(global_invocation_id)]] param: vec3<u32>) {
	let uv = vec2<i32>(param.xy);
	let up_uv = vec2<f32>(f32(uv.x) * 2.0, f32(uv.y) * 2.0); 

	let scale = uniforms.scale;
	
	let a = sample(up_uv + vec2<f32>(-1.0, -1.0) * scale);
	let b = sample(up_uv + vec2<f32>( 0.0, -1.0) * scale);
	let c = sample(up_uv + vec2<f32>( 1.0, -1.0) * scale);
	let d = sample(up_uv + vec2<f32>(-0.5, -0.5) * scale);
	let e = sample(up_uv + vec2<f32>( 0.5, -0.5) * scale);
	let f = sample(up_uv + vec2<f32>(-1.0,  0.0) * scale);
	let g = sample(up_uv + vec2<f32>( 0.0,  0.0) * scale);
	let h = sample(up_uv + vec2<f32>( 1.0,  0.0) * scale);
	let i = sample(up_uv + vec2<f32>(-0.5,  0.5) * scale);
	let j = sample(up_uv + vec2<f32>( 0.5,  0.5) * scale);
	let k = sample(up_uv + vec2<f32>(-1.0,  1.0) * scale);
	let l = sample(up_uv + vec2<f32>( 0.0,  1.0) * scale);
	let m = sample(up_uv + vec2<f32>( 1.0,  1.0) * scale);

	let div = (1.0 / 4.0) * vec2<f32>(0.5, 0.125);

	var o = (d + e + i + j) * div.x;
	o = o + (a + b + g + f) * div.y;
	o = o + (b + c + h + g) * div.y;
	o = o + (f + g + l + k) * div.y;
	o = o + (g + h + m + l) * div.y;

	if (uniforms.pre_filter == 1) {
		let curve = vec3<f32>(
			uniforms.threshold - uniforms.knee,
			uniforms.knee * 2.0,
			0.25 / uniforms.knee,
		);

		o = quadratic_threshold(o, uniforms.threshold, curve);
		o = max(o, vec4<f32>(0.0));
	}

	textureStore(to, uv, o);
}