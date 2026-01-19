use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::components::{CapsuleAxis, CollisionShape, DeathFx, Enemy, EnemyBehavior, EnemyMovement, EnemyType, FxPolicy, Health, HitFx, IdleFx, Collider, ProjectileHitbox, ShaderEffects, SpriteFrameAnimation, EnemyWeaponSockets, WeaponSocket};
use crate::materials::EffectsMaterial;
use crate::resources::EnemyAssetRegistry;

fn stable_z_jitter(transform: &Transform) -> f32 {
	// Tiny deterministic Z offset to stabilize render ordering between overlapping transparent quads.
	// Prevents flicker when multiple asteroids share the same Z and overlap.
	let x = transform.translation.x;
	let y = transform.translation.y;
	((x * 12.9898 + y * 78.233).sin() * 43758.5453).fract() * 0.001
}

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

fn default_fx_policy_for(enemy_type: EnemyType, render_mode: EnemyRenderMode) -> FxPolicy {
	match (enemy_type, render_mode) {
		(EnemyType::SmallAsteroid | EnemyType::MediumAsteroid | EnemyType::LargeAsteroid, EnemyRenderMode::EffectsMaterial) => {
			FxPolicy::new(IdleFx::None, HitFx::ShaderFlash, DeathFx::AsteroidDissolveAndDebris)
		}
		// AsteroidTurret uses animated frame explosion
		(EnemyType::AsteroidTurret, _) => {
			FxPolicy::new(IdleFx::SpriteShimmer, HitFx::None, DeathFx::FrameExplosion)
		}
		(_, EnemyRenderMode::Sprite) => {
			FxPolicy::new(IdleFx::SpriteShimmer, HitFx::None, DeathFx::SpriteExplosion)
		}
		// Fallback: shader-rendered non-asteroid (not expected yet)
		(_, EnemyRenderMode::EffectsMaterial) => {
			FxPolicy::new(IdleFx::None, HitFx::ShaderFlash, DeathFx::SpriteExplosion)
		}
	}
}

fn maybe_sprite_animation(enemy_type: EnemyType, asset_server: &AssetServer) -> Option<SpriteFrameAnimation> {
	match enemy_type {
		EnemyType::Drill => {
			let frames: Vec<Handle<Image>> = (0..=5)
				.map(|i| asset_server.load(format!("enemies/drill/drill_{}.png", i)))
				.collect();
			Some(SpriteFrameAnimation::looping_fps(frames, 14.0))
		}
		_ => None,
	}
}

struct EnemySpriteMetrics {
	sprite_size: Vec2,
	sprite_offset: Vec2,
	projectile_hitbox: ProjectileHitbox,
	content_height_gu: f32,
}

fn bounds_size_px(bounds: [u32; 4]) -> Vec2 {
	Vec2::new(
		(bounds[2].saturating_sub(bounds[0]) + 1) as f32,
		(bounds[3].saturating_sub(bounds[1]) + 1) as f32,
	)
}

fn enemy_sprite_metrics(
	enemy_type: EnemyType,
	sprite_path: &str,
	default_size: f32,
	enemy_assets: &EnemyAssetRegistry,
) -> EnemySpriteMetrics {
	let fallback_hitbox = ProjectileHitbox::circle(Collider::for_enemy_type(enemy_type).radius, Vec2::ZERO);

	if let Some(meta) = enemy_assets.get(enemy_type) {
		if meta.sprite_path != sprite_path {
			debug!(
				"Enemy manifest sprite mismatch: {:?} uses '{}' (manifest '{}')",
				enemy_type,
				sprite_path,
				meta.sprite_path
			);
		}

		let content_height_px = meta.content_size_px[1] as f32;
		if content_height_px > 0.0 {
			let scale = meta.gameplay_height_gu / content_height_px;
			let content_offset = Vec2::new(
				meta.content_center_offset_px[0],
				meta.content_center_offset_px[1],
			);
			let collision_offset_px = Vec2::new(
				meta.collision_center_offset_px[0],
				meta.collision_center_offset_px[1],
			);
			let sprite_size = Vec2::new(meta.texture_px[0] as f32, meta.texture_px[1] as f32) * scale;
			let sprite_offset = content_offset * scale;

			let collision_size_px = bounds_size_px(meta.collision_bounds_px);
			let collision_size_gu = collision_size_px * scale;
			let collision_offset = (collision_offset_px - content_offset) * scale;
			let collision_scale = meta.collision_scale.max(0.01);
			let collision_size_scaled = collision_size_gu * collision_scale;

			let projectile_hitbox = match meta.collision_shape {
				CollisionShape::Circle => {
					let radius = 0.5 * collision_size_scaled.x.max(collision_size_scaled.y);
					ProjectileHitbox::circle(radius, collision_offset)
				}
				CollisionShape::Ellipse => {
					let radii = collision_size_scaled * 0.5;
					ProjectileHitbox::ellipse(radii, collision_offset)
				}
				CollisionShape::Capsule => {
					let (axis, radius, half_length) = if collision_size_scaled.y >= collision_size_scaled.x {
						let radius = 0.5 * collision_size_scaled.x;
						let half_length = (0.5 * collision_size_scaled.y - radius).max(0.0);
						(CapsuleAxis::Vertical, radius, half_length)
					} else {
						let radius = 0.5 * collision_size_scaled.y;
						let half_length = (0.5 * collision_size_scaled.x - radius).max(0.0);
						(CapsuleAxis::Horizontal, radius, half_length)
					};
					ProjectileHitbox::capsule(radius, half_length, axis, collision_offset)
				}
			};

			let metrics = EnemySpriteMetrics {
				sprite_size,
				sprite_offset,
				projectile_hitbox,
				content_height_gu: meta.gameplay_height_gu,
			};

			if std::env::var("TYDUST_LOG_ENEMY_SPRITES").is_ok() {
				let hitbox_desc = match metrics.projectile_hitbox.shape {
					crate::components::HitboxShape::Circle { radius } => {
						format!("circle r={:.1}", radius)
					}
					crate::components::HitboxShape::Ellipse { radii } => {
						format!("ellipse rx={:.1} ry={:.1}", radii.x, radii.y)
					}
					crate::components::HitboxShape::Capsule { radius, half_length, axis } => {
						let axis_label = match axis {
							CapsuleAxis::Horizontal => "horizontal",
							CapsuleAxis::Vertical => "vertical",
						};
						format!(
							"capsule {} r={:.1} half_len={:.1}",
							axis_label,
							radius,
							half_length
						)
					}
				};
				info!(
					"Sprite metrics {:?}: '{}' size=({:.1}, {:.1}) offset=({:.1}, {:.1}) hitbox={}",
					enemy_type,
					sprite_path,
					metrics.sprite_size.x,
					metrics.sprite_size.y,
					metrics.sprite_offset.x,
					metrics.sprite_offset.y,
					hitbox_desc
				);
			}

			return metrics;
		}
	}

	EnemySpriteMetrics {
		sprite_size: Vec2::splat(default_size),
		sprite_offset: Vec2::ZERO,
		projectile_hitbox: fallback_hitbox,
		content_height_gu: default_size,
	}
}

fn enemy_weapon_sockets(
	enemy_type: EnemyType,
	sprite_path: &str,
	enemy_assets: &EnemyAssetRegistry,
) -> Option<EnemyWeaponSockets> {
	let meta = enemy_assets.get(enemy_type)?;
	if meta.sockets.is_empty() {
		return None;
	}
	if meta.sprite_path != sprite_path {
		debug!(
			"Enemy socket sprite mismatch: {:?} uses '{}' (manifest '{}')",
			enemy_type,
			sprite_path,
			meta.sprite_path
		);
	}

	let content_height_px = meta.content_size_px[1] as f32;
	if content_height_px <= 0.0 {
		return None;
	}
	let scale = meta.gameplay_height_gu / content_height_px;
	let content_offset = Vec2::new(
		meta.content_center_offset_px[0],
		meta.content_center_offset_px[1],
	);

	let sockets = meta
		.sockets
		.iter()
		.map(|socket| WeaponSocket {
			id: socket.id.clone(),
			local_offset: (Vec2::new(socket.offset_px[0], socket.offset_px[1]) - content_offset) * scale,
			angle_deg: socket.angle_deg,
			tags: socket.tags.clone(),
		})
		.collect::<Vec<_>>();

	Some(EnemyWeaponSockets { sockets })
}

fn default_weapon_sockets(metrics: &EnemySpriteMetrics) -> EnemyWeaponSockets {
	EnemyWeaponSockets {
		sockets: vec![WeaponSocket {
			id: "default".to_string(),
			local_offset: Vec2::new(0.0, -0.5 * metrics.content_height_gu),
			angle_deg: None,
			tags: vec!["default".to_string()],
		}],
	}
}

pub fn spawn_enemy_with_behavior(
	commands: &mut Commands,
	asset_server: &AssetServer,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<EffectsMaterial>,
	noise_texture: &Handle<Image>,
	enemy_assets: &EnemyAssetRegistry,
	enemy_type: EnemyType,
	sprite_path: &str,
	size: f32,
	transform: Transform,
	behavior: EnemyBehavior,
) -> Entity {
	let render_mode = default_render_mode_for(enemy_type);
	let fx_policy = default_fx_policy_for(enemy_type, render_mode);
	let metrics = enemy_sprite_metrics(enemy_type, sprite_path, size, enemy_assets);
	let weapon_sockets = enemy_weapon_sockets(enemy_type, sprite_path, enemy_assets)
		.unwrap_or_else(|| default_weapon_sockets(&metrics));

	match render_mode {
		EnemyRenderMode::Sprite => {
			let anim = maybe_sprite_animation(enemy_type, asset_server);
			let image = anim
				.as_ref()
				.and_then(|a| a.frames.first())
				.cloned()
				.unwrap_or_else(|| asset_server.load(sprite_path));

			let mut transform = transform;
			transform.translation.x += metrics.sprite_offset.x;
			transform.translation.y += metrics.sprite_offset.y;

			let mut ec = commands.spawn((
				Sprite {
					image,
					custom_size: Some(metrics.sprite_size),
					..default()
				},
				transform,
				Enemy { enemy_type },
				behavior,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				metrics.projectile_hitbox,
				fx_policy,
			));

			ec.insert(weapon_sockets.clone());

			if let Some(anim) = anim {
				ec.insert(anim);
			}

			ec.id()
		}
		EnemyRenderMode::EffectsMaterial => {
			let texture = asset_server.load(sprite_path);
			let mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle::new(
				metrics.sprite_size.x,
				metrics.sprite_size.y,
			)));
			let mut mat = EffectsMaterial::with_dissolve(
				texture,
				noise_texture.clone(),
				LinearRgba::new(0.55, 0.75, 0.95, 1.0),
			);
			// Reduce thick internal edge bands on dissolve for asteroids.
			mat.params.dissolve_edge_width = 0.03;
			mat.params.dissolve_edge_brightness = 1.6;
			// Avoid hard white flash blending.
			mat.params.flash_color = LinearRgba::new(0.75, 0.72, 0.68, 1.0);

			let material = materials.add(mat);

			let mut transform = transform;
			transform.translation.x += metrics.sprite_offset.x;
			transform.translation.y += metrics.sprite_offset.y;
			transform.translation.z += stable_z_jitter(&transform);

			let mut ec = commands.spawn((
				Mesh2d(mesh),
				MeshMaterial2d(material),
				transform,
				Enemy { enemy_type },
				behavior,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				metrics.projectile_hitbox,
				ShaderEffects::default(),
				fx_policy,
			));

			ec.insert(weapon_sockets.clone());

			ec.id()
		}
	}
}

pub fn spawn_enemy_with_movement(
	commands: &mut Commands,
	asset_server: &AssetServer,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<EffectsMaterial>,
	noise_texture: &Handle<Image>,
	enemy_assets: &EnemyAssetRegistry,
	enemy_type: EnemyType,
	sprite_path: &str,
	size: f32,
	transform: Transform,
	movement: EnemyMovement,
) -> Entity {
	let render_mode = default_render_mode_for(enemy_type);
	let fx_policy = default_fx_policy_for(enemy_type, render_mode);
	let metrics = enemy_sprite_metrics(enemy_type, sprite_path, size, enemy_assets);
	let weapon_sockets = enemy_weapon_sockets(enemy_type, sprite_path, enemy_assets)
		.unwrap_or_else(|| default_weapon_sockets(&metrics));

	match render_mode {
		EnemyRenderMode::Sprite => {
			let anim = maybe_sprite_animation(enemy_type, asset_server);
			let image = anim
				.as_ref()
				.and_then(|a| a.frames.first())
				.cloned()
				.unwrap_or_else(|| asset_server.load(sprite_path));

			let mut transform = transform;
			transform.translation.x += metrics.sprite_offset.x;
			transform.translation.y += metrics.sprite_offset.y;

			let mut ec = commands.spawn((
				Sprite {
					image,
					custom_size: Some(metrics.sprite_size),
					..default()
				},
				transform,
				Enemy { enemy_type },
				movement,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				metrics.projectile_hitbox,
				fx_policy,
			));

			ec.insert(weapon_sockets.clone());

			if let Some(anim) = anim {
				ec.insert(anim);
			}

			ec.id()
		}
		EnemyRenderMode::EffectsMaterial => {
			let texture = asset_server.load(sprite_path);
			let mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle::new(
				metrics.sprite_size.x,
				metrics.sprite_size.y,
			)));
			let mut mat = EffectsMaterial::with_dissolve(
				texture,
				noise_texture.clone(),
				LinearRgba::new(0.55, 0.75, 0.95, 1.0),
			);
			mat.params.dissolve_edge_width = 0.03;
			mat.params.dissolve_edge_brightness = 1.6;
			mat.params.flash_color = LinearRgba::new(0.75, 0.72, 0.68, 1.0);

			let material = materials.add(mat);

			let mut transform = transform;
			transform.translation.x += metrics.sprite_offset.x;
			transform.translation.y += metrics.sprite_offset.y;
			transform.translation.z += stable_z_jitter(&transform);

			let mut ec = commands.spawn((
				Mesh2d(mesh),
				MeshMaterial2d(material),
				transform,
				Enemy { enemy_type },
				movement,
				Health::for_enemy_type(enemy_type),
				Collider::for_enemy_type(enemy_type),
				metrics.projectile_hitbox,
				ShaderEffects::default(),
				fx_policy,
			));

			ec.insert(weapon_sockets.clone());

			ec.id()
		}
	}
}

