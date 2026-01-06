use bevy::prelude::*;
use crate::components::{Player, ParticleEmitter};
use super::world::{sizes, speeds, player_bounds};

const TILT_ANGLE: f32 = 0.15;  // ~8.5 degrees, subtle bank
const TILT_SPEED: f32 = 10.0;  // How fast to tilt

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		Sprite {
			image: asset_server.load("sprites/player_ship.png"),
			custom_size: Some(Vec2::new(sizes::PLAYER, sizes::PLAYER)),
			..default()
		},
		Transform::from_xyz(0.0, player_bounds::SPAWN_Y, 1.0),
		Player {
			fire_cooldown: Timer::from_seconds(0.15, TimerMode::Repeating),
		},
		PlayerTilt { target: 0.0, current: 0.0 },
		ParticleEmitter {
			spawn_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
			offset: Vec2::new(0.0, -sizes::PLAYER / 2.0 + 10.0), // Behind ship
		},
	));

	info!("Player ship spawned (size: {} gu)", sizes::PLAYER);
}

#[derive(Component)]
pub struct PlayerTilt {
	pub target: f32,   // Target rotation angle
	pub current: f32,  // Current rotation (smoothly interpolates)
}

pub fn player_movement(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut query: Query<(&mut Transform, &mut PlayerTilt), With<Player>>,
	time: Res<Time>,
) {
	for (mut transform, mut tilt) in query.iter_mut() {
		let mut velocity = Vec3::ZERO;

		if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
			velocity.x -= 1.0;
		}
		if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
			velocity.x += 1.0;
		}
		if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
			velocity.y += 1.0;
		}
		if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
			velocity.y -= 1.0;
		}

		// Set target tilt based on horizontal movement
		tilt.target = if velocity.x < 0.0 {
			TILT_ANGLE  // Tilt right when moving left (like banking)
		} else if velocity.x > 0.0 {
			-TILT_ANGLE  // Tilt left when moving right
		} else {
			0.0  // Return to center
		};

		// Smooth interpolation toward target tilt
		let diff = tilt.target - tilt.current;
		tilt.current += diff * TILT_SPEED * time.delta_secs();

		// Apply rotation to transform
		transform.rotation = Quat::from_rotation_z(tilt.current);

		if velocity.length() > 0.0 {
			velocity = velocity.normalize();
			transform.translation += velocity * speeds::PLAYER * time.delta_secs();
		}

		transform.translation.x = transform.translation.x.clamp(player_bounds::MIN_X, player_bounds::MAX_X);
		transform.translation.y = transform.translation.y.clamp(player_bounds::MIN_Y, player_bounds::MAX_Y);
	}
}
