use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::components::{Player, WeaponPickup, PowerUp, WeaponSwitchEvent, WeaponUpgradeEvent, WeaponType};

const PICKUP_RADIUS: f32 = 50.0;
const PICKUP_DRIFT_SPEED: f32 = 50.0;
const PICKUP_Z: f32 = 0.8;

pub fn spawn_weapon_pickup(
	commands: &mut Commands,
	asset_server: &AssetServer,
	position: Vec2,
	weapon_type: WeaponType,
) {
	let (sprite_color, size) = match weapon_type {
		WeaponType::BasicBlaster => return,
		WeaponType::PlasmaCannon => (Color::srgb(0.8, 0.2, 1.0), 40.0),
		WeaponType::WaveGun => (Color::srgb(0.2, 1.0, 0.5), 40.0),
		WeaponType::SpreadShot => (Color::srgb(1.0, 0.5, 0.2), 40.0),
		WeaponType::MissilePods => (Color::srgb(1.0, 0.2, 0.2), 40.0),
		WeaponType::LaserArray => (Color::srgb(0.0, 0.8, 1.0), 40.0),
		WeaponType::OrbitalDefense => (Color::srgb(1.0, 0.8, 0.0), 40.0),
	};

	commands.spawn((
		Sprite {
			color: sprite_color,
			custom_size: Some(Vec2::splat(size)),
			..default()
		},
		Transform::from_xyz(position.x, position.y, PICKUP_Z),
		WeaponPickup { weapon_type },
	));
}

pub fn spawn_power_up(
	commands: &mut Commands,
	_asset_server: &AssetServer,
	position: Vec2,
	upgrade_amount: i8,
) {
	commands.spawn((
		Sprite {
			color: Color::srgb(1.0, 0.9, 0.2),
			custom_size: Some(Vec2::splat(35.0)),
			..default()
		},
		Transform::from_xyz(position.x, position.y, PICKUP_Z),
		PowerUp { upgrade_amount },
	));
}

pub fn collect_pickups(
	mut commands: Commands,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	player_query: Query<&Transform, With<Player>>,
	weapon_pickup_query: Query<(Entity, &Transform, &WeaponPickup)>,
	power_up_query: Query<(Entity, &Transform, &PowerUp)>,
	mut weapon_switch_events: EventWriter<WeaponSwitchEvent>,
	mut weapon_upgrade_events: EventWriter<WeaponUpgradeEvent>,
) {
	let Ok(player_transform) = player_query.get_single() else { return };
	let player_pos = player_transform.translation.truncate();

	for (entity, pickup_transform, weapon_pickup) in weapon_pickup_query.iter() {
		let pickup_pos = pickup_transform.translation.truncate();
		if player_pos.distance(pickup_pos) < PICKUP_RADIUS {
			weapon_switch_events.send(WeaponSwitchEvent {
				new_weapon: weapon_pickup.weapon_type,
			});
			commands.entity(entity).despawn();
			audio.play(asset_server.load("sounds/laser_fire.ogg"));
		}
	}

	for (entity, pickup_transform, power_up) in power_up_query.iter() {
		let pickup_pos = pickup_transform.translation.truncate();
		if player_pos.distance(pickup_pos) < PICKUP_RADIUS {
			weapon_upgrade_events.send(WeaponUpgradeEvent {
				level_change: power_up.upgrade_amount,
			});
			commands.entity(entity).despawn();
			audio.play(asset_server.load("sounds/laser_fire.ogg"));
		}
	}
}

pub fn move_pickups(
	mut query: Query<&mut Transform, Or<(With<WeaponPickup>, With<PowerUp>)>>,
	time: Res<Time>,
) {
	for mut transform in query.iter_mut() {
		transform.translation.y -= PICKUP_DRIFT_SPEED * time.delta_secs();
	}
}

pub fn cleanup_pickups(
	mut commands: Commands,
	query: Query<(Entity, &Transform), Or<(With<WeaponPickup>, With<PowerUp>)>>,
) {
	for (entity, transform) in query.iter() {
		if transform.translation.y < -600.0 {
			commands.entity(entity).despawn();
		}
	}
}
