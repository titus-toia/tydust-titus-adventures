mod effects_material;
mod projectile_material;
pub mod noise;

pub use effects_material::{EffectsMaterial, EffectsParams};
pub use projectile_material::{ProjectileMaterial, ProjectileMaterialHandles, ProjectileParams};

use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(Material2dPlugin::<EffectsMaterial>::default());
		app.add_plugins(Material2dPlugin::<ProjectileMaterial>::default());
	}
}
