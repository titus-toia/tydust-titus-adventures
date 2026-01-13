use bevy::prelude::*;
use rand::Rng;

use crate::components::{
	DeathFx, Dying, EnemyDeathEvent, EnemyType, FxPolicy, Particle, ShaderEffects,
};

/// Centralized death presentation system.
/// Owns: dissolve start, debris/explosion particles, despawn (for non-dissolve deaths).
pub fn process_enemy_death_fx(
	mut commands: Commands,
	mut death_events: EventReader<EnemyDeathEvent>,
	mut shader_query: Query<Option<&mut ShaderEffects>>,
	fx_query: Query<Option<&FxPolicy>>,
	asset_server: Res<AssetServer>,
) {
	let mut rng = rand::thread_rng();

	for event in death_events.read() {
		let entity = event.entity;

		let policy = fx_query.get(entity).ok().flatten().copied().unwrap_or_default();
		let on_death = if matches!(event.enemy_type, EnemyType::SmallAsteroid | EnemyType::MediumAsteroid | EnemyType::LargeAsteroid) {
			// Hard guarantee: asteroids never use the generic "confetti" explosion path.
			DeathFx::AsteroidDissolveAndDebris
		} else {
			policy.on_death
		};

		match on_death {
			DeathFx::AsteroidDissolveAndDebris => {
				// Start shader dissolve (body breaks apart) if present.
				if let Ok(Some(mut effects)) = shader_query.get_mut(entity) {
					effects.is_dissolving = true;
					effects.dissolve_speed = 4.5;
					effects.flash_amount = 1.0;
					effects.flash_decay_speed = 10.0;
					effects.glow_intensity = effects.glow_intensity.max(1.0);
					effects.glow_color = [0.9, 0.7, 0.4, 1.0];
					effects.pulse_amount = effects.pulse_amount.max(0.25);
					effects.pulse_speed = effects.pulse_speed.max(16.0);
				}

				// Chunky debris + dust (rugged, not confetti).
				let (chunk_count, chunk_size, chunk_speed, chunk_lifetime) = match event.enemy_type {
					EnemyType::LargeAsteroid => (18, 14.0..34.0, 120.0..260.0, 0.7..1.3),
					EnemyType::MediumAsteroid => (14, 12.0..28.0, 110.0..230.0, 0.6..1.1),
					_ => (10, 10.0..22.0, 90.0..200.0, 0.5..0.9),
				};

				for _ in 0..chunk_count {
					let angle = rng.gen_range(0.0..std::f32::consts::TAU);
					let speed = rng.gen_range(chunk_speed.clone());
					let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
					let size = rng.gen_range(chunk_size.clone());
					let lifetime = rng.gen_range(chunk_lifetime.clone());

					commands.spawn((
						Sprite {
							image: asset_server.load("particles/debris_metal.png"),
							custom_size: Some(Vec2::splat(size)),
							..default()
						},
						Transform::from_xyz(event.position.x, event.position.y, 1.1)
							.with_rotation(Quat::from_rotation_z(rng.gen_range(0.0..std::f32::consts::TAU))),
						Particle {
							lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
							velocity,
						},
					));
				}

				for _ in 0..6 {
					let angle = rng.gen_range(0.0..std::f32::consts::TAU);
					let speed = rng.gen_range(40.0..110.0);
					let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
					let size = rng.gen_range(26.0..50.0);
					let lifetime = rng.gen_range(0.7..1.4);

					commands.spawn((
						Sprite {
							image: asset_server.load("particles/smoke_gray.png"),
							custom_size: Some(Vec2::splat(size)),
							..default()
						},
						Transform::from_xyz(event.position.x, event.position.y, 1.05),
						Particle {
							lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
							velocity,
						},
					));
				}

				// Do NOT despawn here; shader cleanup will remove when dissolve finishes.
			}
			DeathFx::SpriteExplosion => {
				// Default "legacy" explosion particles (can be made less confetti later).
				let (particle_count, size_range, speed_range, lifetime_range) = match event.enemy_type {
					EnemyType::Boss => (60, 30.0..60.0, 200.0..400.0, 0.5..0.9),
					EnemyType::HeavyGunship | EnemyType::Corvette => (40, 25.0..45.0, 150.0..320.0, 0.4..0.7),
					EnemyType::LargeAsteroid => (35, 22.0..38.0, 130.0..280.0, 0.4..0.7),
					EnemyType::MediumAsteroid | EnemyType::Bomber => (28, 18.0..32.0, 110.0..240.0, 0.35..0.6),
					EnemyType::Fighter | EnemyType::StationDebris => (22, 15.0..28.0, 90.0..200.0, 0.3..0.55),
					_ => (18, 12.0..24.0, 70.0..170.0, 0.3..0.5),
				};

				for _ in 0..particle_count {
					let angle = rng.gen_range(0.0..std::f32::consts::TAU);
					let speed = rng.gen_range(speed_range.clone());
					let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

					let size = rng.gen_range(size_range.clone());
					let lifetime = rng.gen_range(lifetime_range.clone());

					let sprite = match rng.gen_range(0..3) {
						0 => "particles/flame_orange.png",
						1 => "particles/spark_white.png",
						_ => "particles/exhaust_cyan.png",
					};

					commands.spawn((
						Sprite {
							image: asset_server.load(sprite),
							custom_size: Some(Vec2::splat(size)),
							..default()
						},
						Transform::from_xyz(event.position.x, event.position.y, 1.0),
						Particle {
							lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
							velocity,
						},
					));
				}

				commands.entity(entity).despawn_recursive();
			}
		}
	}
}

/// Ensure dying entities are non-interactive immediately (even if their death FX lasts a while).
pub fn mark_dying_noninteractive(
	mut commands: Commands,
	mut death_events: EventReader<EnemyDeathEvent>,
) {
	for event in death_events.read() {
		commands.entity(event.entity)
			.insert(Dying)
			.remove::<crate::components::Collider>();
	}
}

