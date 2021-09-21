[[group(0), binding(0)]]
var texture: texture_2d<f32>;

[[block]]
struct Histogram {
	histogram: array<atomic<u32>>;
};

[[group(0), binding(1)]]
var<storage, read_write> histogram: Histogram; 

[[block]]
struct Uniforms {
	min_log_lum: f32;
	inv_log_lum_range: f32;
};

[[group(0), binding(2)]]
var<uniform> uniforms: Uniforms;

var<workgroup> histogram_shared: array<atomic<u32>, 256>;

let EPSILON = 0.005;

fn luminance(v: vec3<f32>) -> f32 {
	return dot(v, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn color_to_bin(hdr_color: vec3<f32>, min_log_lum: f32, inv_log_lum_range: f32) -> u32 {
	let lum = luminance(hdr_color);

	if (lum < EPSILON) {
		return 0u;
	}

	let log_lum = clamp((log2(lum) - min_log_lum) * inv_log_lum_range, 0.0, 1.0);

	return u32(log_lum * 254.0 + 1.0);
}

struct ComputeInput {
	[[builtin(local_invocation_index)]] local: u32;
	[[builtin(global_invocation_id)]] global: vec3<u32>; 
};

[[stage(compute), workgroup_size(16, 16, 1)]]
fn main(in: ComputeInput) {
	histogram_shared[in.local] = 0u;
	workgroupBarrier();

	let dim = textureDimensions(texture);

	if (in.global.x < u32(dim.x) && in.global.y < u32(dim.y)) {
		let hdr_color = textureLoad(texture, vec2<i32>(in.global.xy), 0).rgb;
		let bin_index = color_to_bin(hdr_color, uniforms.min_log_lum, uniforms.inv_log_lum_range);

		histogram_shared[bin_index] = histogram_shared[bin_index] + 1u;
	}

	workgroupBarrier();
	
	histogram.histogram[in.local] = histogram.histogram[in.local] + histogram_shared[in.local];
}