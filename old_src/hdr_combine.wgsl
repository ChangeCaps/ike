var VERTICES: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
	vec2<f32>(-1.0, -1.0),
	vec2<f32>(3.0, -1.0),
	vec2<f32>(-1.0, 3.0),
);

[[group(0), binding(0)]]
var texture: texture_2d<f32>;

[[group(0), binding(1)]]
var depth: texture_depth_multisampled_2d;

[[block]]
struct AvgLum {
	lum: f32;
};

[[group(0), binding(2)]]
var<uniform> avg_lum: AvgLum;

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

let RGBTOXYZ = mat3x3<f32>(
	vec3<f32>(0.4124564, 0.2126729, 0.0193339),
	vec3<f32>(0.3575761, 0.7151522, 0.1191920),
	vec3<f32>(0.1804375, 0.0721750, 0.9503041),
);

let XYZTORGB = mat3x3<f32>(
	vec3<f32>(3.2404542, -0.9692660, 0.0556434),
	vec3<f32>(-1.5371385, 1.8760108, -0.2040259),
	vec3<f32>(-0.4985314, 0.0415560, 1.0572252),
);

fn rgb_to_yxy(rgb: vec3<f32>) -> vec3<f32> {
	let xyz = RGBTOXYZ * rgb;

	let x = xyz.r / (xyz.r + xyz.g + xyz.b);
	let y = xyz.g / (xyz.r + xyz.g + xyz.b);

	return vec3<f32>(xyz.g, x, y);
}

fn yxy_to_rgb(yxy: vec3<f32>) -> vec3<f32> {
	let xyz = vec3<f32>(
		yxy.r * yxy.g / yxy.b,
		yxy.r,
		(1.0 - yxy.g -  yxy.b) * (yxy.r / yxy.b),
	);

	return XYZTORGB * xyz;
}

fn gamma_correct(rgb: vec3<f32>) -> vec3<f32> {
	return pow(rgb, vec3<f32>(1.0/2.2));
}

struct FragmentOuput {
	[[builtin(frag_depth)]] depth: f32;
	[[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn main(in: VertexOutput) -> FragmentOuput {
	var out: FragmentOuput;

	var rgb = textureLoad(texture, vec2<i32>(vec2<f32>(textureDimensions(texture)) * in.uv), 0).rgb;
	out.depth = textureLoad(depth, vec2<i32>(vec2<f32>(textureDimensions(depth)) * in.uv), 0);

	var yxy = rgb_to_yxy(rgb);

	let lp = yxy.r / (9.6 * avg_lum.lum + 0.00001);

	yxy = vec3<f32>(tonemap_aces(lp), yxy.gb);

	rgb = yxy_to_rgb(yxy);

	out.color = vec4<f32>(gamma_correct(rgb), 1.0);

	return out;
}