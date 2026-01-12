use bevy::prelude::*;
use crate::components::{Player, WeaponPickup, PowerUp, WeaponSwitchEvent, WeaponUpgradeEvent};
use crate::systems::audio::PlaySfxEvent;

const PICKUP_RADIUS: f32 = 50.0;
const PICKUP_DRIFT_SPEED: f32 = 50.0;

pub fn collect_pickups(
	mut commands: Commands,
	player_query: Query<&Transform, With<Player>>,
	weapon_pickup_query: Query<(Entity, &Transform, &WeaponPickup)>,
	power_up_query: Query<(Entity, &Transform, &PowerUp)>,
	mut weapon_switch_events: EventWriter<WeaponSwitchEvent>,
	mut weapon_upgrade_events: EventWriter<WeaponUpgradeEvent>,
	mut sfx_events: EventWriter<PlaySfxEvent>,
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
			sfx_events.send(PlaySfxEvent::simple("sounds/laser_fire.ogg", 0.55, 60, 0.08));
		}
	}

	for (entity, pickup_transform, power_up) in power_up_query.iter() {
		let pickup_pos = pickup_transform.translation.truncate();
		if player_pos.distance(pickup_pos) < PICKUP_RADIUS {
			weapon_upgrade_events.send(WeaponUpgradeEvent {
				level_change: power_up.upgrade_amount,
			});
			commands.entity(entity).despawn();
			sfx_events.send(PlaySfxEvent::simple("sounds/laser_fire.ogg", 0.55, 60, 0.08));
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
