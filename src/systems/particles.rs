use bevy::prelude::*;
use rand::Rng;
use crate::components::{Particle, ParticleEmitter, Player};

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
