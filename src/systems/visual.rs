use bevy::prelude::*;
use crate::components::ParallaxEntity;

/// Applies atmospheric tint to sprites based on their z-depth
/// Creates depth perception: further back = more blue/purple haze
pub fn apply_atmospheric_tint(
	mut query: Query<(&Transform, &mut Sprite), With<ParallaxEntity>>,
) {
	for (transform, mut sprite) in query.iter_mut() {
		let z = transform.translation.z;

		// Only tint things in the "environmental" range (z=-8 to z=-3)
		// Cosmic backdrop (z < -8) stays untinted
		// Gameplay layer (z > -2) stays untinted
		if z < -8.0 || z > -2.0 {
			continue;
		}

		// Calculate tint intensity: deeper z = more tint
		// z=-8 → max tint, z=-3 → min tint
		let t = ((-z - 3.0) / 5.0).clamp(0.0, 1.0);

		// Atmospheric tint: blue-purple shift
		let r = 1.0 - (t * 0.3);  // Red dims
		let g = 1.0 - (t * 0.2);  // Green dims less
		let b = 1.0 - (t * 0.05); // Blue barely dims

		// Alpha fade: far structures (z=-8) are ghostly, closer ones solid
		let a = if z < -7.0 {
			1.0 - (t * 0.25)  // DeepStructures: more fade
		} else {
			1.0  // MegaStructures & details: solid
		};

		sprite.color = Color::srgba(r, g, b, a);
	}
}

/// Marker component for entities that should have the "interactable" visual style
#[derive(Component)]
pub struct Interactable;
