#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var sprite_texture: texture_2d<f32>;
@group(2) @binding(1) var sprite_sampler: sampler;
@group(2) @binding(2) var noise_texture: texture_2d<f32>;
@group(2) @binding(3) var noise_sampler: sampler;

struct EffectsParams {
	glow_color: vec4<f32>,
	flash_color: vec4<f32>,
	glow_intensity: f32,
	glow_width: f32,
	flash_amount: f32,
	dissolve_amount: f32,
	dissolve_edge_width: f32,
	dissolve_edge_brightness: f32,
	scanline_intensity: f32,
	scanline_count: f32,
	time: f32,
	_padding: f32,
}

@group(2) @binding(4) var<uniform> params: EffectsParams;

fn sample_noise(uv: vec2<f32>) -> f32 {
	return textureSample(noise_texture, noise_sampler, uv).r;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	let texel_size = 1.0 / vec2<f32>(textureDimensions(sprite_texture));
	var original = textureSample(sprite_texture, sprite_sampler, in.uv);

	// ═══════════════════════════════════════════════════════════
	// DISSOLVE EFFECT
	// ═══════════════════════════════════════════════════════════
	if (params.dissolve_amount > 0.0) {
		// Higher frequency noise reduces the "paint girder" look for crumbling materials.
		let noise = sample_noise(in.uv * 9.0);
		let dissolve_threshold = params.dissolve_amount;

		if (noise < dissolve_threshold && original.a > 0.1) {
			let edge_dist = dissolve_threshold - noise;
			if (edge_dist < params.dissolve_edge_width && edge_dist > 0.0) {
				let edge_factor = 1.0 - (edge_dist / params.dissolve_edge_width);
				let edge_color = vec4<f32>(
					params.glow_color.rgb * params.dissolve_edge_brightness,
					edge_factor * original.a
				);
				return edge_color;
			}
			discard;
		}
	}

	// ═══════════════════════════════════════════════════════════
	// DAMAGE FLASH
	// ═══════════════════════════════════════════════════════════
	if (params.flash_amount > 0.0 && original.a > 0.1) {
		original = vec4<f32>(
			mix(original.rgb, params.flash_color.rgb, params.flash_amount),
			original.a
		);
	}

	// ═══════════════════════════════════════════════════════════
	// SCANLINES
	// ═══════════════════════════════════════════════════════════
	if (params.scanline_intensity > 0.0 && original.a > 0.1) {
		let scanline = sin(in.uv.y * params.scanline_count * 3.14159) * 0.5 + 0.5;
		let scanline_factor = 1.0 - (scanline * params.scanline_intensity);
		original = vec4<f32>(original.rgb * scanline_factor, original.a);
	}

	// If we're on a solid pixel, return (possibly modified) original
	if (original.a > 0.5) {
		return original;
	}

	// ═══════════════════════════════════════════════════════════
	// OUTER GLOW (only for transparent pixels near edges)
	// ═══════════════════════════════════════════════════════════
	if (params.glow_intensity > 0.0) {
		var max_neighbor_alpha = 0.0;
		var distance_factor = 0.0;

		let steps = i32(params.glow_width);
		for (var x = -steps; x <= steps; x++) {
			for (var y = -steps; y <= steps; y++) {
				if (x == 0 && y == 0) { continue; }

				let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
				let neighbor = textureSample(sprite_texture, sprite_sampler, in.uv + offset);

				if (neighbor.a > 0.5) {
					let dist = length(vec2<f32>(f32(x), f32(y)));
					let falloff = 1.0 - (dist / (f32(steps) + 1.0));
					distance_factor = max(distance_factor, falloff);
					max_neighbor_alpha = max(max_neighbor_alpha, neighbor.a);
				}
			}
		}

		if (max_neighbor_alpha > 0.5 && distance_factor > 0.0) {
			let glow_alpha = distance_factor * params.glow_intensity * 0.8;
			return vec4<f32>(params.glow_color.rgb, glow_alpha);
		}
	}

	return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
