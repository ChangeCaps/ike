[[block]]
struct AvgLum {
	avg: vec3<f32>;
};

[[group(0), binding(0)]]
var<storage, read_write> avg_lum: AvgLum;

[[block]]
struct Histogram {
	histogram: array<atomic<u32>>;
};

[[group(0), binding(1)]]
var<storage, read_write> histogram: Histogram; 

[[stage(compute), workgroup_size(256, 1, 1)]]
fn main() {

}