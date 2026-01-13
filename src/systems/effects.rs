use bevy::prelude::*;
use bevy::sprite::MeshMaterial2d;
use crate::components::{EnemyHitEvent, FxPolicy, HitFx, ShaderEffects};
use crate::materials::EffectsMaterial;

pub fn apply_shader_hit_flash(
	mut hit_events: EventReader<EnemyHitEvent>,
	mut query: Query<(&mut ShaderEffects, Option<&FxPolicy>)>,
) {
	for event in hit_events.read() {
		let Ok((mut effects, policy)) = query.get_mut(event.enemy) else { continue };
		if !matches!(policy.map(|p| p.on_hit), Some(HitFx::ShaderFlash)) {
			continue;
		}

		// A rugged, high-contrast "impact" flash for shader-rendered entities.
		effects.flash_amount = (effects.flash_amount + 0.7).min(1.0);
		effects.flash_decay_speed = 9.0;
		effects.glow_intensity = effects.glow_intensity.max(0.6);
		effects.pulse_amount = effects.pulse_amount.max(0.15);
		effects.pulse_speed = effects.pulse_speed.max(12.0);
	}
}

pub fn update_shader_effects(
	time: Res<Time>,
	mut materials: ResMut<Assets<EffectsMaterial>>,
	mut query: Query<(&mut ShaderEffects, &MeshMaterial2d<EffectsMaterial>)>,
) {
	let dt = time.delta_secs();
	let elapsed = time.elapsed_secs();

	for (mut effects, material_handle) in query.iter_mut() {
		if let Some(material) = materials.get_mut(&material_handle.0) {
			// Glow with pulse
			let pulse = if effects.pulse_speed > 0.0 {
				1.0 + effects.pulse_amount * (elapsed * effects.pulse_speed).sin()
			} else {
				1.0
			};
			material.params.glow_intensity = effects.glow_intensity * pulse;
			material.params.glow_color = LinearRgba::new(
				effects.glow_color[0],
				effects.glow_color[1],
				effects.glow_color[2],
				effects.glow_color[3],
			);

			// Flash decay
			if effects.flash_amount > 0.0 {
				effects.flash_amount = (effects.flash_amount - dt * effects.flash_decay_speed).max(0.0);
				material.params.flash_amount = effects.flash_amount;
			}

			// Dissolve animation
			if effects.is_dissolving {
				effects.dissolve_amount = (effects.dissolve_amount + dt * effects.dissolve_speed).min(1.0);
				material.params.dissolve_amount = effects.dissolve_amount;
			}

			// Time uniform for shader animations
			material.params.time = elapsed;
		}
	}
}

pub fn cleanup_dissolved_entities(
	mut commands: Commands,
	query: Query<(Entity, &ShaderEffects)>,
) {
	for (entity, effects) in query.iter() {
		if effects.is_dissolving && effects.dissolve_amount >= 1.0 {
			commands.entity(entity).despawn_recursive();
		}
	}
}
