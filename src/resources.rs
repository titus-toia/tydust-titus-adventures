use bevy::prelude::*;
use crate::components::{ShipType, WeaponType};

#[derive(Resource, Default)]
pub struct SelectedShip {
	pub ship_type: Option<ShipType>,
}

#[derive(Resource)]
pub struct SelectedWeapon {
	pub weapon_type: WeaponType,
}

impl Default for SelectedWeapon {
	fn default() -> Self {
		Self {
			weapon_type: WeaponType::BasicBlaster,
		}
	}
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
	#[default]
	ShipSelection,
	Playing,
}

#[derive(Resource)]
pub struct BloomLevel {
	pub level: f32, // 0.0 = disabled, 0.01-1.0 = intensity
}

impl BloomLevel {
	pub fn new(percent: u32) -> Self {
		Self {
			level: (percent as f32) / 100.0,
		}
	}

	pub fn is_enabled(&self) -> bool {
		self.level > 0.0
	}
}
