use bevy::prelude::*;
use crate::components::{Enemy, EnemyMovement, MovementPattern, Player};
use super::world::HALF_WORLD_HEIGHT;
use super::level::CurrentLevel;

pub fn update_enemy_movement(
	mut query: Query<(&mut Transform, &mut EnemyMovement)>,
	time: Res<Time>,
	level: Option<Res<CurrentLevel>>,
) {
	let scroll_speed = level
		.and_then(|l| l.get_current_phase().map(|p| p.scroll_speed))
		.unwrap_or(100.0);

	for (mut transform, mut movement) in query.iter_mut() {
		movement.time_alive += time.delta_secs();
		let t = movement.time_alive;

		// All enemies scroll down with level
		transform.translation.y -= scroll_speed * time.delta_secs();

		// Apply movement pattern
		match movement.pattern {
			MovementPattern::SineWave { amplitude, frequency } => {
				transform.translation.x = movement.spawn_x + (t * frequency).sin() * amplitude;
			}
			MovementPattern::PassBy { speed } => {
				transform.translation.y -= speed * time.delta_secs();
			}
			MovementPattern::Circle { radius, angular_speed } => {
				let angle = t * angular_speed;
				transform.translation.x = movement.spawn_x + angle.cos() * radius;
				// Small Y oscillation for circle pattern
				transform.translation.y += angle.sin() * radius * 0.5 * time.delta_secs();
			}
			MovementPattern::Straight { speed } => {
				transform.translation.y -= speed * time.delta_secs();
			}
		}
	}
}

pub fn rotate_enemies_toward_player(
	mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
	player: Query<&Transform, With<Player>>,
) {
	let Ok(player_transform) = player.get_single() else { return };
	let player_pos = player_transform.translation.truncate();

	for mut transform in enemies.iter_mut() {
		let enemy_pos = transform.translation.truncate();
		let direction = player_pos - enemy_pos;

		// atan2 gives angle from +X axis, ships face +Y, so subtract PI/2
		let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
		transform.rotation = Quat::from_rotation_z(angle);
	}
}

pub fn cleanup_enemies(
	mut commands: Commands,
	query: Query<(Entity, &Transform), With<Enemy>>,
) {
	let despawn_y = -(HALF_WORLD_HEIGHT + 200.0);
	for (entity, transform) in query.iter() {
		if transform.translation.y < despawn_y {
			commands.entity(entity).despawn();
		}
	}
}
