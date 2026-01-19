#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ProjectileParams {
	color: vec4<f32>,
	glow_color: vec4<f32>,
	radius: f32,
	softness: f32,
	glow_width: f32,
	glow_intensity: f32,
	_padding: f32,
}

@group(2) @binding(0) var<uniform> params: ProjectileParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	// Centered UV space: [-1..1]
	let uv = in.uv * 2.0 - vec2<f32>(1.0, 1.0);
	let dist = length(uv);

	if (params.radius <= 0.0) {
		return vec4<f32>(0.0, 0.0, 0.0, 0.0);
	}

	let inner = max(params.radius - params.softness, 0.0);
	let body_alpha = 1.0 - smoothstep(inner, params.radius, dist);
	let glow_alpha = (1.0 - smoothstep(params.radius, params.radius + params.glow_width, dist))
		* params.glow_intensity;

	let alpha = clamp(body_alpha + glow_alpha, 0.0, 1.0);
	let rgb = params.color.rgb * body_alpha + params.glow_color.rgb * glow_alpha;

	return vec4<f32>(rgb, alpha);
}
