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
