var VERTICES: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
	vec2<f32>(-1.0, -1.0),
	vec2<f32>(3.0, -1.0),
	vec2<f32>(-1.0, 3.0),
);

[[group(0), binding(0)]]
var texture: texture_2d<f32>;

[[group(0), binding(1)]]
var depth: texture_depth_2d;

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] index: u32) -> VertexOutput {
	var out: VertexOutput;

	let vertex = VERTICES[index];

	out.position = vec4<f32>(vertex.x, vertex.y, 0.0, 1.0);
	out.uv = vertex * vec2<f32>(0.5, -0.5) + 0.5;

	return out;
}

fn luminance(v: vec3<f32>) -> f32 {
	return dot(v, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn change_luminance(c_in: vec3<f32>, l_out: f32) -> vec3<f32> {
	let l_in = luminance(c_in);
	return c_in * (l_out / l_in);
}

fn reinhard_luminance(color: vec3<f32>) -> vec3<f32> {
	let l_old = luminance(color);
	let l_new = l_old / (1.0 + l_old);
	return change_luminance(color, l_new);
}

fn tonemap_aces(x: f32) -> f32 {
    // Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
    let a = 2.51;
    let b = 0.03; 
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return (x * (a * x + b)) / (x * (c * x + d) + e);
}

struct FragmentOuput {
	[[builtin(frag_depth)]] depth: f32;
	[[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn main(in: VertexOutput) -> FragmentOuput {
	var out: FragmentOuput;

	out.color = textureLoad(texture, vec2<i32>(vec2<f32>(textureDimensions(texture)) * in.uv), 0);
	out.depth = textureLoad(depth, vec2<i32>(vec2<f32>(textureDimensions(depth)) * in.uv), 0);

	out.color = vec4<f32>(pow(out.color.rgb, vec3<f32>(1.0/2.2)), out.color.a);
	out.color = vec4<f32>(reinhard_luminance(out.color.rgb), out.color.a);

	return out;
}