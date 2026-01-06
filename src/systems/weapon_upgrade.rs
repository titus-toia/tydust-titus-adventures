use bevy::prelude::*;
use crate::components::{Player, Weapon, WeaponSwitchEvent, WeaponUpgradeEvent, PlayerHitEvent, WeaponType};

pub fn handle_weapon_switch(
	mut weapon_switch_events: EventReader<WeaponSwitchEvent>,
	mut query: Query<&mut Weapon, With<Player>>,
) {
	for event in weapon_switch_events.read() {
		if let Ok(mut weapon) = query.get_single_mut() {
			weapon.weapon_type = event.new_weapon;
			weapon.level = 1;

			let config = weapon.weapon_type.config();
			weapon.fire_cooldown = Timer::from_seconds(
				config.base_cooldown,
				TimerMode::Repeating
			);

			info!("Switched to {:?} (Level 1)", weapon.weapon_type);
		}
	}
}

pub fn handle_weapon_upgrade(
	mut weapon_upgrade_events: EventReader<WeaponUpgradeEvent>,
	mut query: Query<&mut Weapon, With<Player>>,
) {
	for event in weapon_upgrade_events.read() {
		if let Ok(mut weapon) = query.get_single_mut() {
			let config = weapon.weapon_type.config();

			if weapon.weapon_type == WeaponType::BasicBlaster {
				continue;
			}

			let new_level = (weapon.level as i8 + event.level_change)
				.clamp(1, config.max_level as i8) as u8;

			if new_level != weapon.level {
				weapon.level = new_level;
				info!("{:?} upgraded to Level {}", weapon.weapon_type, weapon.level);
			}
		}
	}
}

pub fn handle_player_hit(
	mut player_hit_events: EventReader<PlayerHitEvent>,
	mut query: Query<&mut Weapon, With<Player>>,
) {
	for _ in player_hit_events.read() {
		if let Ok(mut weapon) = query.get_single_mut() {
			if weapon.weapon_type == WeaponType::BasicBlaster {
				continue;
			}

			weapon.level = weapon.level.saturating_sub(2);

			if weapon.level == 0 {
				weapon.weapon_type = WeaponType::BasicBlaster;
				weapon.level = 0;
				let config = weapon.weapon_type.config();
				weapon.fire_cooldown = Timer::from_seconds(
					config.base_cooldown,
					TimerMode::Repeating
				);
				info!("Weapon downgraded to BasicBlaster");
			} else {
				info!("{:?} downgraded to Level {}", weapon.weapon_type, weapon.level);
			}
		}
	}
}

pub fn debug_weapon_controls(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut weapon_upgrade_events: EventWriter<WeaponUpgradeEvent>,
	mut weapon_switch_events: EventWriter<WeaponSwitchEvent>,
) {
	if keyboard_input.just_pressed(KeyCode::Equal) || keyboard_input.just_pressed(KeyCode::NumpadAdd) {
		weapon_upgrade_events.send(WeaponUpgradeEvent { level_change: 1 });
	}

	if keyboard_input.just_pressed(KeyCode::Minus) || keyboard_input.just_pressed(KeyCode::NumpadSubtract) {
		weapon_upgrade_events.send(WeaponUpgradeEvent { level_change: -1 });
	}

	if keyboard_input.just_pressed(KeyCode::Digit1) {
		weapon_switch_events.send(WeaponSwitchEvent { new_weapon: WeaponType::BasicBlaster });
	}

	if keyboard_input.just_pressed(KeyCode::Digit2) {
		weapon_switch_events.send(WeaponSwitchEvent { new_weapon: WeaponType::PlasmaCannon });
	}

	if keyboard_input.just_pressed(KeyCode::Digit3) {
		weapon_switch_events.send(WeaponSwitchEvent { new_weapon: WeaponType::WaveGun });
	}

	if keyboard_input.just_pressed(KeyCode::Digit4) {
		weapon_switch_events.send(WeaponSwitchEvent { new_weapon: WeaponType::SpreadShot });
	}

	if keyboard_input.just_pressed(KeyCode::Digit5) {
		weapon_switch_events.send(WeaponSwitchEvent { new_weapon: WeaponType::MissilePods });
	}

	if keyboard_input.just_pressed(KeyCode::Digit6) {
		weapon_switch_events.send(WeaponSwitchEvent { new_weapon: WeaponType::LaserArray });
	}

	if keyboard_input.just_pressed(KeyCode::Digit7) {
		weapon_switch_events.send(WeaponSwitchEvent { new_weapon: WeaponType::OrbitalDefense });
	}
}
