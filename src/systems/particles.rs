use bevy::prelude::*;
use bevy::text::{Text2d, TextColor, TextFont};
use rand::Rng;
use crate::components::{Particle, ParticleEmitter, Player, EnemyDeathEvent, EnemyType, PlayerHitEvent, EnemyHitEvent, Enemy, FloatingDamageNumber};
use crate::resources::DamageNumbersEnabled;

pub fn spawn_engine_particles(
	mut commands: Commands,
	mut emitters: Query<(&Transform, &mut ParticleEmitter), With<Player>>,
	asset_server: Res<AssetServer>,
	time: Res<Time>,
) {
	let mut rng = rand::thread_rng();

	for (transform, mut emitter) in emitters.iter_mut() {
		emitter.spawn_timer.tick(time.delta());

		if emitter.spawn_timer.just_finished() {
			let base_pos = transform.translation.truncate() + emitter.offset;

			// Random offset for natural look
			let offset = Vec2::new(
				rng.gen_range(-8.0..8.0),
				rng.gen_range(-5.0..5.0),
			);
			let spawn_pos = base_pos + offset;

			// Randomize velocity (mostly downward with slight spread)
			let velocity = Vec2::new(
				rng.gen_range(-20.0..20.0),
				rng.gen_range(-150.0..-80.0),
			);

			// Pick random particle sprite
			let sprite = if rng.gen_bool(0.7) {
				"particles/flame_orange.png"
			} else {
				"particles/exhaust_cyan.png"
			};

			let size = rng.gen_range(15.0..25.0);
			let lifetime = rng.gen_range(0.2..0.4);

			commands.spawn((
				Sprite {
					image: asset_server.load(sprite),
					custom_size: Some(Vec2::splat(size)),
					..default()
				},
				Transform::from_xyz(spawn_pos.x, spawn_pos.y, 0.9),
				Particle {
					lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
					velocity,
				},
			));
		}
	}
}

pub fn update_particles(
	mut commands: Commands,
	mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
	time: Res<Time>,
) {
	for (entity, mut transform, mut sprite, mut particle) in query.iter_mut() {
		particle.lifetime.tick(time.delta());

		if particle.lifetime.finished() {
			commands.entity(entity).despawn();
			continue;
		}

		// Move particle
		transform.translation.x += particle.velocity.x * time.delta_secs();
		transform.translation.y += particle.velocity.y * time.delta_secs();

		// Fade out based on remaining lifetime
		let remaining = particle.lifetime.fraction_remaining();
	// Preserve existing RGB tint and only fade alpha (useful for smoke/dust coloring).
	sprite.color.set_alpha(remaining);
	}
}

pub fn spawn_explosion_particles(
	mut commands: Commands,
	mut death_events: EventReader<EnemyDeathEvent>,
	asset_server: Res<AssetServer>,
) {
	// NOTE: legacy system (kept temporarily). Death presentation is now owned by `process_enemy_death_fx`.
	// This function is no longer scheduled from main.
	let mut rng = rand::thread_rng();

	for event in death_events.read() {
		// Extra breakup debris for asteroids: chunky shards that sell "splitting apart"
		// while the asteroid body dissolves via shader.
		if matches!(event.enemy_type, EnemyType::SmallAsteroid | EnemyType::MediumAsteroid | EnemyType::LargeAsteroid) {
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

			// A little dust/smoke to keep it rugged (not fiery).
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
		}

		// Scale explosion based on enemy type - more dramatic explosions
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

			// Mix of orange/yellow explosion colors
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
	}
}

pub fn spawn_enemy_hit_particles(
	mut commands: Commands,
	mut hit_events: EventReader<EnemyHitEvent>,
	enemy_query: Query<(&Transform, &Enemy)>,
	asset_server: Res<AssetServer>,
) {
	let mut rng = rand::thread_rng();

	for event in hit_events.read() {
		let Ok((enemy_transform, enemy)) = enemy_query.get(event.enemy) else { continue };
		let pos = enemy_transform.translation.truncate();

		// Scale hit particles by enemy size
		let (particle_count, size_range) = match enemy.enemy_type {
			EnemyType::Boss => (12, 12.0..20.0),
			EnemyType::HeavyGunship | EnemyType::Corvette => (8, 10.0..16.0),
			EnemyType::LargeAsteroid => (7, 9.0..14.0),
			EnemyType::MediumAsteroid | EnemyType::Bomber => (6, 8.0..12.0),
			_ => (5, 6.0..10.0),
		};

		// Small directional burst at impact point
		for _ in 0..particle_count {
			let angle = rng.gen_range(0.0..std::f32::consts::TAU);
			let speed = rng.gen_range(60.0..120.0);
			let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

			let size = rng.gen_range(size_range.clone());
			let lifetime = rng.gen_range(0.15..0.3);

			// Orange/white impact sparks
			let sprite = if rng.gen_bool(0.6) {
				"particles/spark_white.png"
			} else {
				"particles/flame_orange.png"
			};

			commands.spawn((
				Sprite {
					image: asset_server.load(sprite),
					custom_size: Some(Vec2::splat(size)),
					..default()
				},
				Transform::from_xyz(pos.x, pos.y, 0.95),
				Particle {
					lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
					velocity,
				},
			));
		}
	}
}

pub fn spawn_player_hit_particles(
	mut commands: Commands,
	mut hit_events: EventReader<PlayerHitEvent>,
	player_query: Query<&Transform, With<Player>>,
	asset_server: Res<AssetServer>,
) {
	let mut rng = rand::thread_rng();

	for _event in hit_events.read() {
		let Ok(player_transform) = player_query.get_single() else { continue };
		let pos = player_transform.translation.truncate();

		// Shield/spark burst effect
		for _ in 0..15 {
			let angle = rng.gen_range(0.0..std::f32::consts::TAU);
			let speed = rng.gen_range(80.0..180.0);
			let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

			let size = rng.gen_range(8.0..16.0);
			let lifetime = rng.gen_range(0.2..0.4);

			// Blue/white sparks for shield hit
			let sprite = if rng.gen_bool(0.6) {
				"particles/spark_white.png"
			} else {
				"particles/exhaust_cyan.png"
			};

			commands.spawn((
				Sprite {
					image: asset_server.load(sprite),
					custom_size: Some(Vec2::splat(size)),
					..default()
				},
				Transform::from_xyz(pos.x, pos.y, 1.5),
				Particle {
					lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
					velocity,
				},
			));
		}
	}
}

// === Floating Damage Numbers ===

// Damage number color palette for visual distinction
const DAMAGE_COLORS: &[(f32, f32, f32)] = &[
	(1.0, 0.9, 0.2),   // Yellow/gold
	(1.0, 0.5, 0.1),   // Orange
	(0.2, 1.0, 0.4),   // Green
	(0.4, 0.9, 1.0),   // Cyan
	(1.0, 0.4, 0.6),   // Pink
	(0.8, 0.6, 1.0),   // Lavender
];

pub fn spawn_floating_damage_numbers(
	mut commands: Commands,
	mut hit_events: EventReader<EnemyHitEvent>,
	enemy_query: Query<&Transform, With<Enemy>>,
	asset_server: Res<AssetServer>,
	damage_numbers_enabled: Res<DamageNumbersEnabled>,
) {
	if !damage_numbers_enabled.0 {
		// Clear events even if disabled so they don't pile up
		hit_events.clear();
		return;
	}

	let mut rng = rand::thread_rng();

	for event in hit_events.read() {
		let Ok(enemy_transform) = enemy_query.get(event.enemy) else { continue };
		let pos = enemy_transform.translation.truncate();

		// Random angle between -45 and 45 degrees from vertical (in radians)
		let angle_offset: f32 = rng.gen_range(-0.785..0.785); // ~45 degrees
		let base_speed: f32 = 80.0;
		let velocity = Vec2::new(
			angle_offset.sin() * base_speed,
			angle_offset.cos() * base_speed,
		);

		// Pick random color from palette
		let color_idx = rng.gen_range(0..DAMAGE_COLORS.len());
		let (r, g, b) = DAMAGE_COLORS[color_idx];

		commands.spawn((
			Text2d::new(format!("{:.0}", event.damage)),
			TextFont {
				font: asset_server.load("fonts/Orbitron-Variable.ttf"),
				font_size: 18.0,
				..default()
			},
			TextColor(Color::srgb(r, g, b)),
			Transform::from_xyz(pos.x, pos.y, 2.0),
			FloatingDamageNumber {
				lifetime: Timer::from_seconds(0.8, TimerMode::Once),
				velocity,
			},
		));
	}
}

pub fn update_floating_damage_numbers(
	mut commands: Commands,
	mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut FloatingDamageNumber)>,
	time: Res<Time>,
) {
	for (entity, mut transform, mut color, mut damage_num) in query.iter_mut() {
		damage_num.lifetime.tick(time.delta());

		if damage_num.lifetime.finished() {
			commands.entity(entity).despawn();
			continue;
		}

		// Move along velocity direction
		transform.translation.x += damage_num.velocity.x * time.delta_secs();
		transform.translation.y += damage_num.velocity.y * time.delta_secs();

		// Fade out
		let alpha = damage_num.lifetime.fraction_remaining();
		color.0 = color.0.with_alpha(alpha);
	}
}
