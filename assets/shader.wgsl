struct GlobalUniforms {
	view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> global: GlobalUniforms;

struct VertexInput {
	@location(0) pos: vec3<f32>,
}

struct VertexOutput {
	@builtin(position) clip_pos: vec4<f32>,
	@location(0) world_pos: vec3<f32>,
}

@vertex
fn main_vertex(input: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_pos = global.view_proj * vec4<f32>(input.pos, 1.0);
	out.world_pos = input.pos;

	return out;
}

@fragment
fn main_frag(input: VertexOutput) -> @location(0) vec4<f32> {

	let dx = dpdx(input.world_pos);
	let dy = dpdy(input.world_pos);

	let normal = normalize(cross(dx, dy));

	let light_dir = normalize(vec3<f32>(0.5, 0.7, 1.0));

	let diffuse = max(dot(normal, light_dir), 0.0);

	let ambient = 0.25;

	let base_color = vec3<f32>(0.2, 0.6, 1.0);

	let lighting = ambient + diffuse * (1.0 - ambient);

	let color = base_color * lighting;

	return vec4<f32>(color, 1.0);
}