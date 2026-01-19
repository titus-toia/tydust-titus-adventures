use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{AlphaMode2d, Material2d};

#[derive(Clone, Copy, ShaderType, Default)]
pub struct ProjectileParams {
	pub color: LinearRgba,
	pub glow_color: LinearRgba,
	pub radius: f32,
	pub softness: f32,
	pub glow_width: f32,
	pub glow_intensity: f32,
	pub _padding: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ProjectileMaterial {
	#[uniform(0)]
	pub params: ProjectileParams,
}

impl ProjectileMaterial {
	pub fn orange_pellet() -> Self {
		Self {
			params: ProjectileParams {
				color: LinearRgba::new(1.4, 0.55, 0.12, 1.0),
				glow_color: LinearRgba::new(2.6, 1.0, 0.2, 1.0),
				radius: 0.92,
				softness: 0.12,
				glow_width: 0.35,
				glow_intensity: 0.7,
				_padding: 0.0,
			},
		}
	}
}

impl Default for ProjectileMaterial {
	fn default() -> Self {
		Self {
			params: ProjectileParams::default(),
		}
	}
}

impl Material2d for ProjectileMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/projectile_material.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode2d {
		AlphaMode2d::Blend
	}
}

#[derive(Resource, Clone)]
pub struct ProjectileMaterialHandles {
	pub orange_pellet: Handle<ProjectileMaterial>,
}

impl ProjectileMaterialHandles {
	pub fn new(materials: &mut Assets<ProjectileMaterial>) -> Self {
		Self {
			orange_pellet: materials.add(ProjectileMaterial::orange_pellet()),
		}
	}
}
