[[block]]
struct AvgLum {
	lum: f32;
};

[[group(0), binding(0)]]
var<storage, read_write> avg_lum: AvgLum;

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
	time_coefficient: f32;
	num_pixels: u32;
};

[[group(0), binding(2)]]
var<uniform> uniforms: Uniforms; 

var<workgroup> shared_histogram: array<atomic<u32>, 256>;

struct ComputeInput {
	[[builtin(local_invocation_index)]] local: u32;
};

[[stage(compute), workgroup_size(16, 16, 1)]]
fn main(in: ComputeInput) {
	let bin_count = histogram.histogram[in.local];
	shared_histogram[in.local] = bin_count * in.local;

	workgroupBarrier();

	histogram.histogram[in.local] = 0u;

	for (var c = 128u; c > 0u; c = c >> 1u) {
		if (in.local < c) {
			shared_histogram[in.local] = shared_histogram[in.local] + shared_histogram[in.local + c];
		}

		workgroupBarrier();
	}

	if (in.local == 0u) {
		let weighted_log_avg = (f32(shared_histogram[0]) / max(f32(uniforms.num_pixels) - f32(bin_count), 1.0)) - 1.0;

		let weighted_avg = exp2(((weighted_log_avg / 254.0) / uniforms.inv_log_lum_range) + uniforms.min_log_lum);

		let lum_last_frame = avg_lum.lum;

		let adapted_lum = lum_last_frame + (weighted_avg - lum_last_frame) * uniforms.time_coefficient;
		avg_lum.lum = adapted_lum;
	}
}