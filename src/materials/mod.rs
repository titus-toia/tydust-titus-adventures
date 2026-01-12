mod effects_material;
pub mod noise;

pub use effects_material::{EffectsMaterial, EffectsParams};

use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(Material2dPlugin::<EffectsMaterial>::default());
	}
}
