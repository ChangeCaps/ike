[[group(0), binding(0)]]
var eq_texture: texture_2d<f32>;

[[group(1), binding(1)]]
var cube_texture: texture_storage_2d_array<rgba32float, write>;

[[stage(compute), workgroup_size(1024, 1, 1)]]
fn main([[builtin(global_invocation_id)]] param: vec3<u32>) {

}