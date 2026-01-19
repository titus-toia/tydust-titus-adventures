use bevy::prelude::*;

use crate::components::ThrusterFx;

pub fn animate_thrusters(
	time: Res<Time>,
	mut q: Query<(&ThrusterFx, &mut Transform, &mut Sprite)>,
) {
	let t = time.elapsed_secs();

	for (fx, mut transform, mut sprite) in &mut q {
		// Keep the local offset stable even if other code touches the transform.
		transform.translation = fx.local_offset;

		let wave = (t * fx.pulse_hz * std::f32::consts::TAU + fx.phase).sin() * 0.5 + 0.5; // 0..1
		let scale_mul = 1.0 + (wave - 0.5) * 2.0 * fx.scale_pulse;
		transform.scale = fx.base_scale * scale_mul;

		let alpha = (fx.base_alpha + (wave - 0.5) * 2.0 * fx.alpha_pulse).clamp(0.0, 1.0);
		sprite.color.set_alpha(alpha);
	}
}

