use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::components::{DamageFxPolicy, Enemy, EnemyBehavior, EnemyMovement, EnemyType, Health, Collider, ShaderEffects};
use crate::materials::EffectsMaterial;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyRenderMode {
	Sprite,
	EffectsMaterial,
}

fn default_render_mode_for(enemy_type: EnemyType) -> EnemyRenderMode {
	// Default mapping: keep ships on sprites (simple + shimmer), but put asteroids on shader material
	// so we can dissolve them on death (and later add debris chunk sprites for breakup motion).
	match enemy_type {
		EnemyType::SmallAsteroid | EnemyType::MediumAsteroid | EnemyType::LargeAsteroid => EnemyRenderMode::EffectsMaterial,
		_ => EnemyRenderMode::Sprite,
	}
}

fn default_fx_policy_for(render_mode: EnemyRenderMode) -> DamageFxPolicy {
	match render_mode {
		EnemyRenderMode::Sprite => DamageFxPolicy::SpriteShimmer,
		EnemyRenderMode::EffectsMaterial => DamageFxPolicy::ShaderFlashDissolve,
	}
}

pub fn spawn_enemy_with_behavior(
	commands: &mut Commands,
	asset_server: &AssetServer,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<EffectsMaterial>,
	noise_texture: &Handle<Image>,
	enemy_type: EnemyType,
	sprite_path: &str,
	size: f32,
	transform: Transform,
	behavior: EnemyBehavior,
) -> Entity {
	let render_mode = default_render_mode_for(enemy_type);
	let fx_policy = default_fx_policy_for(render_mode);

	match render_mode {
		EnemyRenderMode::Sprite => {
			commands.spawn((
				Sprite {
					image: asset_server.load(sprite_path),
					custom_size: Some(Vec2::splat(size)),
					..default()
				},
				transform,
				Enemy { enemy_type },
				behavior,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				fx_policy,
			)).id()
		}
		EnemyRenderMode::EffectsMaterial => {
			let texture = asset_server.load(sprite_path);
			let mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle::new(size, size)));
			let material = materials.add(EffectsMaterial::with_dissolve(
				texture,
				noise_texture.clone(),
				LinearRgba::new(0.55, 0.75, 0.95, 1.0),
			));

			commands.spawn((
				Mesh2d(mesh),
				MeshMaterial2d(material),
				transform,
				Enemy { enemy_type },
				behavior,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				ShaderEffects::default(),
				fx_policy,
			)).id()
		}
	}
}

pub fn spawn_enemy_with_movement(
	commands: &mut Commands,
	asset_server: &AssetServer,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<EffectsMaterial>,
	noise_texture: &Handle<Image>,
	enemy_type: EnemyType,
	sprite_path: &str,
	size: f32,
	transform: Transform,
	movement: EnemyMovement,
) -> Entity {
	let render_mode = default_render_mode_for(enemy_type);
	let fx_policy = default_fx_policy_for(render_mode);

	match render_mode {
		EnemyRenderMode::Sprite => {
			commands.spawn((
				Sprite {
					image: asset_server.load(sprite_path),
					custom_size: Some(Vec2::splat(size)),
					..default()
				},
				transform,
				Enemy { enemy_type },
				movement,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				fx_policy,
			)).id()
		}
		EnemyRenderMode::EffectsMaterial => {
			let texture = asset_server.load(sprite_path);
			let mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle::new(size, size)));
			let material = materials.add(EffectsMaterial::with_dissolve(
				texture,
				noise_texture.clone(),
				LinearRgba::new(0.55, 0.75, 0.95, 1.0),
			));

			commands.spawn((
				Mesh2d(mesh),
				MeshMaterial2d(material),
				transform,
				Enemy { enemy_type },
				movement,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				ShaderEffects::default(),
				fx_policy,
			)).id()
		}
	}
}

