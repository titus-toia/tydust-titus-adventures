use bevy::prelude::*;
use rand::Rng;

use crate::components::{
	DeathFx, Dying, EnemyDeathEvent, EnemyType, FxPolicy, Particle, ShaderEffects,
};

// "Crumble into dust" tuning. This intentionally avoids big square-card spam:
// - low counts
// - small sizes
// - non-uniform aspect ratios
// - tinted smoke (RGB preserved by particle fade system)
const ASTEROID_DUST_ENABLED: bool = true;

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
					// Crumble: slightly slower than "pop", less white-out, more ember edge.
					effects.dissolve_speed = 2.6;
					effects.flash_amount = 0.18;
					effects.flash_decay_speed = 8.0;
					effects.glow_intensity = effects.glow_intensity.max(0.9);
					effects.glow_color = [0.65, 0.55, 0.45, 1.0]; // dusty warm
					effects.pulse_amount = effects.pulse_amount.max(0.12);
					effects.pulse_speed = effects.pulse_speed.max(10.0);
				}

				if ASTEROID_DUST_ENABLED {
					// Dust plume: small + anisotropic so it doesn't read as square cards.
					let (dust_count, w_range, h_range, speed_range, lifetime_range) = match event.enemy_type {
						EnemyType::LargeAsteroid => (18, 10.0..26.0, 6.0..18.0, 60.0..180.0, 0.35..0.75),
						EnemyType::MediumAsteroid => (14, 9.0..22.0, 6.0..16.0, 55.0..160.0, 0.30..0.65),
						_ => (10, 8.0..18.0, 5.0..14.0, 45.0..140.0, 0.25..0.55),
					};

					for _ in 0..dust_count {
						let angle = rng.gen_range(0.0..std::f32::consts::TAU);
						let speed = rng.gen_range(speed_range.clone());
						let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

						let w = rng.gen_range(w_range.clone());
						let h = rng.gen_range(h_range.clone());
						let lifetime = rng.gen_range(lifetime_range.clone());

						commands.spawn((
							Sprite {
								image: asset_server.load("particles/smoke_gray.png"),
								custom_size: Some(Vec2::new(w, h)),
								color: Color::srgba(0.65, 0.65, 0.68, 0.8),
								..default()
							},
							Transform::from_xyz(event.position.x, event.position.y, 1.05)
								.with_rotation(Quat::from_rotation_z(rng.gen_range(0.0..std::f32::consts::TAU))),
							Particle {
								lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
								velocity,
							},
						));
					}

					// A few tiny grit specks (non-confetti) for texture.
					for _ in 0..4 {
						let angle = rng.gen_range(0.0..std::f32::consts::TAU);
						let speed = rng.gen_range(90.0..220.0);
						let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
						let size = rng.gen_range(3.0..7.0);
						let lifetime = rng.gen_range(0.18..0.35);

						commands.spawn((
							Sprite {
								image: asset_server.load("particles/spark_white.png"),
								custom_size: Some(Vec2::splat(size)),
								color: Color::srgba(0.75, 0.72, 0.68, 0.9),
								..default()
							},
							Transform::from_xyz(event.position.x, event.position.y, 1.08),
							Particle {
								lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
								velocity,
							},
						));
					}
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

