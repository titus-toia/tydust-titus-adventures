use bevy::prelude::*;
use rand::Rng;
use crate::components::{Particle, ParticleEmitter, Player, EnemyDeathEvent, EnemyType, PlayerHitEvent, EnemyHitEvent, Enemy};

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
		sprite.color = Color::srgba(1.0, 1.0, 1.0, remaining);
	}
}

pub fn spawn_explosion_particles(
	mut commands: Commands,
	mut death_events: EventReader<EnemyDeathEvent>,
	asset_server: Res<AssetServer>,
) {
	let mut rng = rand::thread_rng();

	for event in death_events.read() {
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
