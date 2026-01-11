use bevy::prelude::*;
use crate::components::{Player, ParticleEmitter, ShipType, Weapon, WeaponType, PlayerDefenses, Collider};
use crate::resources::{SelectedShip, SelectedWeapon};
use super::world::player_bounds;

const TILT_ANGLE: f32 = 0.15;  // ~8.5 degrees, subtle bank
const TILT_SPEED: f32 = 10.0;  // How fast to tilt

pub fn spawn_player(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	selected_ship: Res<SelectedShip>,
	selected_weapon: Res<SelectedWeapon>,
) {
	let ship_type = selected_ship.ship_type.unwrap_or(ShipType::Tempest);
	let stats = ship_type.get_stats();

	let weapon_type = selected_weapon.weapon_type;
	let weapon_config = weapon_type.config();
	let weapon_level = if weapon_type == WeaponType::BasicBlaster { 0 } else { 1 };

	commands.spawn((
		Sprite {
			image: asset_server.load(ship_type.sprite_path()),
			custom_size: Some(Vec2::new(stats.size, stats.size)),
			..default()
		},
		Transform::from_xyz(0.0, player_bounds::SPAWN_Y, 1.0),
		Player {
			fire_cooldown: Timer::from_seconds(stats.fire_cooldown, TimerMode::Repeating),
			ship_type,
		},
		Weapon {
			weapon_type,
			level: weapon_level,
			fire_cooldown: Timer::from_seconds(weapon_config.base_cooldown, TimerMode::Repeating),
		},
		PlayerTilt { target: 0.0, current: 0.0 },
		ParticleEmitter {
			spawn_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
			offset: Vec2::new(0.0, -stats.size / 2.0 + 10.0),
		},
		PlayerDefenses::default(),
		Collider::new(stats.size / 2.0 * 0.7), // Slightly smaller than visual for fair gameplay
	));

	info!(
		"Player ship spawned: {:?} (size: {:.0} gu, speed: {}, fire rate: {})",
		ship_type, stats.size, stats.speed, stats.fire_cooldown
	);
}

#[derive(Component)]
pub struct PlayerTilt {
	pub target: f32,   // Target rotation angle
	pub current: f32,  // Current rotation (smoothly interpolates)
}

pub fn player_movement(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut query: Query<(&mut Transform, &mut PlayerTilt, &Player), With<Player>>,
	time: Res<Time>,
) {
	for (mut transform, mut tilt, player) in query.iter_mut() {
		let stats = player.ship_type.get_stats();
		let speed = stats.speed;

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
			transform.translation += velocity * speed * time.delta_secs();
		}

		transform.translation.x = transform.translation.x.clamp(player_bounds::MIN_X, player_bounds::MAX_X);
		transform.translation.y = transform.translation.y.clamp(player_bounds::MIN_Y, player_bounds::MAX_Y);
	}
}
