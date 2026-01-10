use bevy::prelude::*;
use crate::components::ParallaxEntity;

/// Applies atmospheric tint to sprites based on their z-depth
/// Creates depth perception: further back = more desaturated/darker
pub fn apply_atmospheric_tint(
	mut query: Query<(&Transform, &mut Sprite), With<ParallaxEntity>>,
) {
	for (transform, mut sprite) in query.iter_mut() {
		let z = transform.translation.z;

		// Per-layer tint values (desaturation, brightness)
		let (desaturation, brightness) = if z <= -8.5 {
			// DeepSpace (-9.5), FarField (-8.0): no tint (cosmic backdrop)
			continue;
		} else if z <= -7.2 {
			// DeepStructures (-7.5): heavy
			(0.6, 0.45)
		} else if z <= -6.5 {
			// MegaStructures (-7.0): medium-heavy
			(0.5, 0.55)
		} else if z <= -5.5 {
			// MidDistance (-6.0): soft
			(0.25, 0.75)
		} else if z <= -3.5 {
			// StructureDetails (-4.0): very soft
			(0.1, 0.9)
		} else if z <= -2.5 {
			// NearBackground (-3.0): minimal
			(0.05, 0.95)
		} else {
			// Foreground and gameplay: no tint
			continue;
		};

		let gray = brightness;
		let r = (1.0 - desaturation) + (gray * desaturation);
		let g = (1.0 - desaturation) + (gray * desaturation);
		let b = (1.0 - desaturation) + (gray * desaturation);

		sprite.color = Color::srgba(r, g, b, 1.0);
	}
}

/// Applies ambient occlusion to tall structures in DISTANT layers
/// Darkens sprites based on height to simulate shadow accumulation
pub fn apply_ambient_occlusion(
	mut query: Query<(&Transform, &mut Sprite), With<ParallaxEntity>>,
) {
	for (transform, mut sprite) in query.iter_mut() {
		let z = transform.translation.z;

		// Only apply AO to distant structure layers (deep/mega) - NOT to close layers
		if z <= -6.5 && z > -8.0 {
			if let Some(size) = sprite.custom_size {
				let height = size.y;
				let ao_intensity = (height / 1000.0).clamp(0.0, 0.3);

				let current = sprite.color.to_srgba();
				sprite.color = Color::srgba(
					current.red * (1.0 - ao_intensity),
					current.green * (1.0 - ao_intensity),
					current.blue * (1.0 - ao_intensity),
					current.alpha
				);
			}
		}
	}
}
