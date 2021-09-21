let NORMAL_MAP_FLAG_BIT: u32 = 1u;
let SKINNED_FLAG_BIT: u32 = 2u;

[[block]]
struct View {
	view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: View;

struct Material {
	albedo: vec4<f32>;
	emission: vec4<f32>;
	roughness: f32;
	metallic: f32;
	reflectance: f32;
	shadow_softness: f32;
	shadow_softness_falloff: f32;
	shadow_blocker_samples: u32;
	shadow_pcf_samples: u32;
};

[[block]]
struct Mesh {
	material: Material;
	flags: u32;
	joint_count: u32;
};

[[group(1), binding(4)]]
var<uniform> mesh: Mesh;

[[block]]
struct JointMatrices {
	matrices: array<mat4x4<f32>>; 
};

[[group(2), binding(0)]]
var<storage, read> joint_matrices: JointMatrices;

struct VertexInput {
	[[builtin(instance_index)]] index: u32;

	// mesh
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] joints: vec4<u32>;
	[[location(2)]] weights: vec4<f32>;

	// instance
	[[location(8)]] transform_0: vec4<f32>;
	[[location(9)]] transform_1: vec4<f32>;
	[[location(10)]] transform_2: vec4<f32>;
	[[location(11)]] transform_3: vec4<f32>;
};

fn mul_mat(weight: f32, joint: mat4x4<f32>) -> mat4x4<f32> {
	return mat4x4<f32>(
		vec4<f32>(weight) * joint.x,	
		vec4<f32>(weight) * joint.y,	
		vec4<f32>(weight) * joint.z,	
		vec4<f32>(weight) * joint.w,	
	); 
}

// since Matrix + Matrix isn't implemented this is necessary
fn add_mat(lhs: mat4x4<f32>, rhs: mat4x4<f32>) -> mat4x4<f32> {
	return mat4x4<f32>(
		lhs.x + rhs.x,
		lhs.y + rhs.y,
		lhs.z + rhs.z,
		lhs.w + rhs.w,
	);
}

[[stage(vertex)]]
fn main(in: VertexInput) -> [[builtin(position)]] vec4<f32> {
	let transform = mat4x4<f32>(
		in.transform_0,
		in.transform_1,
		in.transform_2,
		in.transform_3,
	);

	var clip_position: vec4<f32>;

	if ((mesh.flags & SKINNED_FLAG_BIT) == 0u) {	
		clip_position = view.view_proj * transform * vec4<f32>(in.position, 1.0);
	} else {
		let joint_offset = in.index * mesh.joint_count;

		let x = mul_mat(in.weights.x, joint_matrices.matrices[joint_offset + in.joints.x]);
		let y = mul_mat(in.weights.y, joint_matrices.matrices[joint_offset + in.joints.y]);
		let z = mul_mat(in.weights.z, joint_matrices.matrices[joint_offset + in.joints.z]);
		let w = mul_mat(in.weights.w, joint_matrices.matrices[joint_offset + in.joints.w]);

		let skin = add_mat(add_mat(add_mat(x, y), z), w);

		clip_position = view.view_proj * transform * skin * vec4<f32>(in.position, 1.0);
	}

	return clip_position;
}