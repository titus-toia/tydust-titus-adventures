use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::components::{Player, Laser};
use super::world::{sizes, speeds, HALF_WORLD_HEIGHT};

const LASER_Z: f32 = 0.5;

pub fn spawn_lasers(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
	mut query: Query<(&Transform, &mut Player)>,
	time: Res<Time>,
) {
	if !keyboard_input.pressed(KeyCode::Space) {
		return;
	}

	for (transform, mut player) in query.iter_mut() {
		player.fire_cooldown.tick(time.delta());

		if player.fire_cooldown.finished() {
			let laser_x = transform.translation.x;
			let laser_y = transform.translation.y + 55.0;

			commands.spawn((
				Sprite {
					color: Color::srgb(0.2, 0.7, 1.0),
					custom_size: Some(Vec2::new(sizes::LASER_WIDTH, sizes::LASER_HEIGHT)),
					..default()
				},
				Transform::from_xyz(laser_x, laser_y, LASER_Z),
				Laser {
					speed: speeds::LASER,
				},
			));

			audio.play(asset_server.load("sounds/laser_fire.ogg"));
			player.fire_cooldown.reset();
		}
	}
}

pub fn move_lasers(
	mut query: Query<(&mut Transform, &Laser)>,
	time: Res<Time>,
) {
	for (mut transform, laser) in query.iter_mut() {
		transform.translation.y += laser.speed * time.delta_secs();
	}
}

pub fn cleanup_lasers(
	mut commands: Commands,
	query: Query<(Entity, &Transform), With<Laser>>,
) {
	for (entity, transform) in query.iter() {
		if transform.translation.y > HALF_WORLD_HEIGHT + 50.0 {
			commands.entity(entity).despawn();
		}
	}
}
