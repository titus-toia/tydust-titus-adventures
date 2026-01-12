use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{Material2d, AlphaMode2d};

#[derive(Clone, Copy, ShaderType, Default)]
pub struct EffectsParams {
	pub glow_color: LinearRgba,
	pub flash_color: LinearRgba,
	pub glow_intensity: f32,
	pub glow_width: f32,
	pub flash_amount: f32,
	pub dissolve_amount: f32,
	pub dissolve_edge_width: f32,
	pub dissolve_edge_brightness: f32,
	pub scanline_intensity: f32,
	pub scanline_count: f32,
	pub time: f32,
	pub _padding: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct EffectsMaterial {
	#[texture(0)]
	#[sampler(1)]
	pub texture: Handle<Image>,

	#[texture(2)]
	#[sampler(3)]
	pub noise_texture: Handle<Image>,

	#[uniform(4)]
	pub params: EffectsParams,
}

impl Material2d for EffectsMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/effects_material.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode2d {
		AlphaMode2d::Blend // Enable transparency!
	}
}

impl EffectsMaterial {
	pub fn with_glow(texture: Handle<Image>, noise: Handle<Image>, color: LinearRgba, intensity: f32) -> Self {
		Self {
			texture,
			noise_texture: noise,
			params: EffectsParams {
				glow_color: color,
				glow_intensity: intensity,
				glow_width: 3.0,
				flash_color: LinearRgba::WHITE,
				..default()
			},
		}
	}

	pub fn with_dissolve(texture: Handle<Image>, noise: Handle<Image>, glow_color: LinearRgba) -> Self {
		Self {
			texture,
			noise_texture: noise,
			params: EffectsParams {
				glow_color,
				dissolve_amount: 0.0,
				dissolve_edge_width: 0.08,
				dissolve_edge_brightness: 3.0,
				..default()
			},
		}
	}

	pub fn with_scanlines(texture: Handle<Image>, noise: Handle<Image>, intensity: f32) -> Self {
		Self {
			texture,
			noise_texture: noise,
			params: EffectsParams {
				scanline_intensity: intensity,
				scanline_count: 100.0,
				..default()
			},
		}
	}
}

impl Default for EffectsMaterial {
	fn default() -> Self {
		Self {
			texture: Handle::default(),
			noise_texture: Handle::default(),
			params: EffectsParams::default(),
		}
	}
}
