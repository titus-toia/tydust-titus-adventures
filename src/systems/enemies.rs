use bevy::prelude::*;
use crate::components::{
	Enemy, EnemyType, EnemyMovement, MovementPattern, Player, EnemyBehavior, BehaviorType, SineAxis,
	EasingType, FormationLeader, FormationMember, EnemyShooter, EnemyProjectile, EnemyPreviousPosition,
	EnemyProjectileType, EnemyFireOverride, EnemyFireConfig, EnemyWeaponSockets, FirePattern, AimMode,
	SocketSelector, PlayerVelocity, WeaponSocket,
};
use crate::materials::ProjectileMaterialHandles;
use crate::resources::EnemyAssetRegistry;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;
use super::world::HALF_WORLD_HEIGHT;
use super::level::CurrentLevel;
use std::f32::consts::{PI, FRAC_PI_2};

pub fn update_enemy_movement(
	mut query: Query<(&mut Transform, &mut EnemyMovement), Without<crate::components::Dying>>,
	time: Res<Time>,
	level: Option<Res<CurrentLevel>>,
) {
	let scroll_speed = level
		.and_then(|l| l.get_current_phase().map(|p| p.scroll_speed))
		.unwrap_or(100.0);

	for (mut transform, mut movement) in query.iter_mut() {
		movement.time_alive += time.delta_secs();
		let t = movement.time_alive;

		// All enemies scroll down with level
		transform.translation.y -= scroll_speed * time.delta_secs();

		// Apply movement pattern
		match movement.pattern {
			MovementPattern::SineWave { amplitude, frequency } => {
				transform.translation.x = movement.spawn_x + (t * frequency).sin() * amplitude;
			}
			MovementPattern::PassBy { speed } => {
				transform.translation.y -= speed * time.delta_secs();
			}
			MovementPattern::Circle { radius, angular_speed } => {
				let angle = t * angular_speed;
				transform.translation.x = movement.spawn_x + angle.cos() * radius;
				// Small Y oscillation for circle pattern
				transform.translation.y += angle.sin() * radius * 0.5 * time.delta_secs();
			}
			MovementPattern::Straight { speed } => {
				transform.translation.y -= speed * time.delta_secs();
			}
		}
	}
}


pub fn cleanup_enemies(
	mut commands: Commands,
	query: Query<(Entity, &Transform, &Enemy), (With<Enemy>, Without<crate::components::Dying>)>,
) {
	let despawn_y = -(HALF_WORLD_HEIGHT + 200.0);
	for (entity, transform, enemy) in query.iter() {
		if transform.translation.y < despawn_y {
			info!("🗑️  Despawning {:?} at y={:.1} (below {:.1})", enemy.enemy_type, transform.translation.y, despawn_y);
			commands.entity(entity).despawn();
		}
	}
}

// === New Behavior System ===

pub fn execute_enemy_behaviors(
	mut query: Query<(&mut Transform, &mut EnemyBehavior, &mut Sprite), (Without<Player>, Without<FormationMember>, Without<crate::components::Dying>)>,
	mut query_no_sprite: Query<(&mut Transform, &mut EnemyBehavior), (Without<Player>, Without<FormationMember>, Without<Sprite>, Without<crate::components::Dying>)>,
	time: Res<Time>,
	player_query: Query<&Transform, With<Player>>,
	level: Option<Res<CurrentLevel>>,
) {
	let delta = time.delta_secs();
	let _scroll_speed = level
		.and_then(|l| l.get_current_phase().map(|p| p.scroll_speed))
		.unwrap_or(100.0);

	for (mut transform, mut behavior_state, mut sprite) in query.iter_mut() {
		behavior_state.total_time_alive += delta;

		// Enemies with explicit behaviors handle their own positioning
		// Don't auto-scroll them or they fight against MoveToPosition/MoveCircular

		if behavior_state.current_index >= behavior_state.behaviors.len() {
			continue;
		}

		let current = &behavior_state.behaviors[behavior_state.current_index];
		let elapsed = behavior_state.total_time_alive - behavior_state.behavior_start_time;

// 		// Debug logging for boss every 0.5s
// 		if transform.translation.y > 600.0 && (behavior_state.total_time_alive * 2.0) as i32 % 1 == 0 {
// 			info!("🎯 Boss behavior update: pos=({:.1}, {:.1}), behavior={:?}, elapsed={:.2}s",
// 				transform.translation.x, transform.translation.y, current.behavior_type, elapsed);
// 		}

		let should_advance = match current.duration {
			Some(dur) => elapsed >= dur,
			None => false,
		};

		if should_advance {
			behavior_state.current_index += 1;
			behavior_state.behavior_start_time = behavior_state.total_time_alive;
			continue;
		}

		execute_behavior(
			&current.behavior_type,
			&mut transform,
			&mut sprite,
			&behavior_state,
			elapsed,
			delta,
			&player_query,
		);
	}

	// Material-based enemies (e.g. shader asteroids) don't have `Sprite`, but they may still use behaviors.
	for (mut transform, mut behavior_state) in query_no_sprite.iter_mut() {
		behavior_state.total_time_alive += delta;

		if behavior_state.current_index >= behavior_state.behaviors.len() {
			continue;
		}

		let current = &behavior_state.behaviors[behavior_state.current_index];
		let elapsed = behavior_state.total_time_alive - behavior_state.behavior_start_time;

		let should_advance = match current.duration {
			Some(dur) => elapsed >= dur,
			None => false,
		};

		if should_advance {
			behavior_state.current_index += 1;
			behavior_state.behavior_start_time = behavior_state.total_time_alive;
			continue;
		}

		execute_behavior_no_sprite(
			&current.behavior_type,
			&mut transform,
			&behavior_state,
			elapsed,
			delta,
			&player_query,
		);
	}
}

fn execute_behavior(
	behavior: &BehaviorType,
	transform: &mut Transform,
	sprite: &mut Sprite,
	state: &EnemyBehavior,
	elapsed: f32,
	delta: f32,
	player_query: &Query<&Transform, With<Player>>,
) {
	match behavior {
		BehaviorType::MoveStraight { velocity } => {
			transform.translation.x += velocity.x * delta;
			transform.translation.y += velocity.y * delta;
		}

		BehaviorType::MoveSineWave { base_velocity, amplitude, frequency, axis } => {
			let offset = (elapsed * frequency).sin() * amplitude;

			transform.translation.x += base_velocity.x * delta;
			transform.translation.y += base_velocity.y * delta;

			match axis {
				SineAxis::Horizontal => {
					transform.translation.x = state.spawn_position.x + offset;
				}
				SineAxis::Vertical => {
					transform.translation.y = state.spawn_position.y + offset;
				}
			}
		}

		BehaviorType::MoveCircular { center_offset, radius, angular_speed, clockwise } => {
			let angle = elapsed * angular_speed * if *clockwise { 1.0 } else { -1.0 };
			let center = state.spawn_position + *center_offset;
			transform.translation.x = center.x + angle.cos() * radius;
			transform.translation.y = center.y + angle.sin() * radius;
		}

		BehaviorType::MoveToPosition { target, speed, easing } => {
			let current_pos = transform.translation.truncate();
			let direction = (*target - current_pos).normalize_or_zero();
			let distance = current_pos.distance(*target);

			if distance > 0.1 {
				let effective_speed = match easing {
					EasingType::Linear => *speed,
					EasingType::EaseOut => {
						let progress = (distance / 100.0).min(1.0);
						speed * progress
					}
					EasingType::EaseIn => {
						let progress = 1.0 - (distance / 100.0).min(1.0);
						speed * (0.3 + progress * 0.7)
					}
					EasingType::EaseInOut => {
						let progress = (distance / 100.0).min(1.0);
						if progress > 0.5 {
							speed * progress
						} else {
							speed * (0.5 + progress)
						}
					}
				};

				transform.translation += (direction * effective_speed * delta).extend(0.0);
			}
		}

		BehaviorType::FollowPlayer { speed, max_distance, offset } => {
			if let Ok(player_transform) = player_query.get_single() {
				let target = player_transform.translation.truncate() + *offset;
				let current_pos = transform.translation.truncate();
				let distance = current_pos.distance(target);

				if let Some(max_dist) = max_distance {
					if distance < *max_dist {
						return;
					}
				}

				let direction = (target - current_pos).normalize_or_zero();
				transform.translation += (direction * speed * delta).extend(0.0);
			}
		}

		BehaviorType::FollowFormation { .. } => {
			// Handled by update_formations system
		}

		BehaviorType::Drift { velocity, variance } => {
			let noise_x = (elapsed * 2.0).sin() * variance;
			let noise_y = (elapsed * 3.0).cos() * variance;
			transform.translation.x += (velocity.x + noise_x) * delta;
			transform.translation.y += (velocity.y + noise_y) * delta;
		}

		BehaviorType::Accelerate { initial_velocity, acceleration } => {
			let velocity = *initial_velocity + *acceleration * elapsed;
			transform.translation.x += velocity.x * delta;
			transform.translation.y += velocity.y * delta;
		}

		BehaviorType::Wait { maintain_velocity: _ } => {
			// Do nothing, just wait
		}

		BehaviorType::FacePlayer { rotation_speed } => {
			if let Ok(player_transform) = player_query.get_single() {
				let to_player = player_transform.translation.truncate() - transform.translation.truncate();
				let target_angle = to_player.y.atan2(to_player.x) - FRAC_PI_2;

				if *rotation_speed > 0.0 {
					let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
					let mut angle_diff = target_angle - current_angle;
					while angle_diff > PI { angle_diff -= 2.0 * PI; }
					while angle_diff < -PI { angle_diff += 2.0 * PI; }
					let rotation_delta = angle_diff.clamp(-rotation_speed * delta, rotation_speed * delta);
					transform.rotation = Quat::from_rotation_z(current_angle + rotation_delta);
				} else {
					transform.rotation = Quat::from_rotation_z(target_angle);
				}
			}
		}

		BehaviorType::FaceDirection { direction, rotation_speed } => {
			let target_angle = direction.y.atan2(direction.x) - FRAC_PI_2;

			if *rotation_speed > 0.0 {
				let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
				let mut angle_diff = target_angle - current_angle;
				while angle_diff > PI { angle_diff -= 2.0 * PI; }
				while angle_diff < -PI { angle_diff += 2.0 * PI; }
				let rotation_delta = angle_diff.clamp(-rotation_speed * delta, rotation_speed * delta);
				transform.rotation = Quat::from_rotation_z(current_angle + rotation_delta);
			} else {
				transform.rotation = Quat::from_rotation_z(target_angle);
			}
		}

		BehaviorType::FaceVelocity => {
			// Track velocity based on position change (simplified)
			// In a real implementation, you'd store velocity in the component
		}

		BehaviorType::FadeOut { fade_speed } => {
			let current_alpha = sprite.color.alpha();
			sprite.color.set_alpha((current_alpha - fade_speed * delta).max(0.0));
		}

		BehaviorType::FadeIn { fade_speed } => {
			let current_alpha = sprite.color.alpha();
			sprite.color.set_alpha((current_alpha + fade_speed * delta).min(1.0));
		}

		BehaviorType::Flash { color, frequency } => {
			let alpha = ((elapsed * frequency * 2.0 * PI).sin() * 0.5 + 0.5).max(0.0).min(1.0);
			sprite.color = Color::srgba(color[0], color[1], color[2], alpha);
		}

		BehaviorType::Parallel { behaviors } => {
			for sub_behavior in behaviors {
				execute_behavior(
					&sub_behavior.behavior_type,
					transform,
					sprite,
					state,
					elapsed,
					delta,
					player_query,
				);
			}
		}
	}
}

fn execute_behavior_no_sprite(
	behavior: &BehaviorType,
	transform: &mut Transform,
	state: &EnemyBehavior,
	elapsed: f32,
	delta: f32,
	player_query: &Query<&Transform, With<Player>>,
) {
	match behavior {
		BehaviorType::MoveStraight { velocity } => {
			transform.translation.x += velocity.x * delta;
			transform.translation.y += velocity.y * delta;
		}

		BehaviorType::MoveSineWave { base_velocity, amplitude, frequency, axis } => {
			let offset = (elapsed * frequency).sin() * amplitude;

			transform.translation.x += base_velocity.x * delta;
			transform.translation.y += base_velocity.y * delta;

			match axis {
				SineAxis::Horizontal => {
					transform.translation.x = state.spawn_position.x + offset;
				}
				SineAxis::Vertical => {
					transform.translation.y = state.spawn_position.y + offset;
				}
			}
		}

		BehaviorType::MoveCircular { center_offset, radius, angular_speed, clockwise } => {
			let angle = elapsed * angular_speed * if *clockwise { 1.0 } else { -1.0 };
			let center = state.spawn_position + *center_offset;
			transform.translation.x = center.x + angle.cos() * radius;
			transform.translation.y = center.y + angle.sin() * radius;
		}

		BehaviorType::MoveToPosition { target, speed, easing } => {
			let current_pos = transform.translation.truncate();
			let direction = (*target - current_pos).normalize_or_zero();
			let distance = current_pos.distance(*target);

			if distance > 0.1 {
				let effective_speed = match easing {
					EasingType::Linear => *speed,
					EasingType::EaseOut => {
						let progress = (distance / 100.0).min(1.0);
						speed * progress
					}
					EasingType::EaseIn => {
						let progress = 1.0 - (distance / 100.0).min(1.0);
						speed * (0.3 + progress * 0.7)
					}
					EasingType::EaseInOut => {
						let progress = (distance / 100.0).min(1.0);
						if progress > 0.5 {
							speed * progress
						} else {
							speed * (0.5 + progress)
						}
					}
				};

				transform.translation += (direction * effective_speed * delta).extend(0.0);
			}
		}

		BehaviorType::FollowPlayer { speed, max_distance, offset } => {
			if let Ok(player_transform) = player_query.get_single() {
				let target = player_transform.translation.truncate() + *offset;
				let current_pos = transform.translation.truncate();
				let distance = current_pos.distance(target);

				if let Some(max_dist) = max_distance {
					if distance < *max_dist {
						return;
					}
				}

				let direction = (target - current_pos).normalize_or_zero();
				transform.translation += (direction * speed * delta).extend(0.0);
			}
		}

		BehaviorType::FollowFormation { .. } => {
			// Handled by update_formations system
		}

		BehaviorType::Drift { velocity, variance } => {
			let noise_x = (elapsed * 2.0).sin() * variance;
			let noise_y = (elapsed * 3.0).cos() * variance;
			transform.translation.x += (velocity.x + noise_x) * delta;
			transform.translation.y += (velocity.y + noise_y) * delta;
		}

		BehaviorType::Accelerate { initial_velocity, acceleration } => {
			let velocity = *initial_velocity + *acceleration * elapsed;
			transform.translation.x += velocity.x * delta;
			transform.translation.y += velocity.y * delta;
		}

		BehaviorType::Wait { maintain_velocity: _ } => {
			// Do nothing, just wait
		}

		BehaviorType::FacePlayer { rotation_speed } => {
			if let Ok(player_transform) = player_query.get_single() {
				let to_player = player_transform.translation.truncate() - transform.translation.truncate();
				let target_angle = to_player.y.atan2(to_player.x) - FRAC_PI_2;

				if *rotation_speed > 0.0 {
					let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
					let mut angle_diff = target_angle - current_angle;
					while angle_diff > PI { angle_diff -= 2.0 * PI; }
					while angle_diff < -PI { angle_diff += 2.0 * PI; }
					let rotation_delta = angle_diff.clamp(-rotation_speed * delta, rotation_speed * delta);
					transform.rotation = Quat::from_rotation_z(current_angle + rotation_delta);
				} else {
					transform.rotation = Quat::from_rotation_z(target_angle);
				}
			}
		}

		BehaviorType::FaceDirection { direction, rotation_speed } => {
			let target_angle = direction.y.atan2(direction.x) - FRAC_PI_2;

			if *rotation_speed > 0.0 {
				let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
				let mut angle_diff = target_angle - current_angle;
				while angle_diff > PI { angle_diff -= 2.0 * PI; }
				while angle_diff < -PI { angle_diff += 2.0 * PI; }
				let rotation_delta = angle_diff.clamp(-rotation_speed * delta, rotation_speed * delta);
				transform.rotation = Quat::from_rotation_z(current_angle + rotation_delta);
			} else {
				transform.rotation = Quat::from_rotation_z(target_angle);
			}
		}

		BehaviorType::FaceVelocity => {
			// Not implemented
		}

		BehaviorType::FadeOut { .. } | BehaviorType::FadeIn { .. } | BehaviorType::Flash { .. } => {
			// Sprite-only presentation behaviors (ignored for material-based enemies).
		}

		BehaviorType::Parallel { behaviors } => {
			for sub_behavior in behaviors {
				execute_behavior_no_sprite(
					&sub_behavior.behavior_type,
					transform,
					state,
					elapsed,
					delta,
					player_query,
				);
			}
		}
	}
}

pub fn update_formations(
	leader_query: Query<(&Transform, &FormationLeader)>,
	mut member_query: Query<(&mut Transform, &FormationMember), Without<FormationLeader>>,
) {
	use std::collections::HashMap;

	// Build a quick lookup so members can follow leaders even if spawn order was "member first".
	let mut leader_positions: HashMap<&str, Vec2> = HashMap::new();
	for (leader_transform, leader) in leader_query.iter() {
		leader_positions.insert(leader.formation_id.as_str(), leader_transform.translation.truncate());
	}

	for (mut member_transform, member) in member_query.iter_mut() {
		if let Some(leader_pos) = leader_positions.get(member.formation_id.as_str()) {
			let target_pos = *leader_pos + member.offset;
			member_transform.translation = target_pos.extend(member_transform.translation.z);
		}
	}
}

// === Enemy Shooting System ===

pub fn setup_enemy_shooters(
	mut commands: Commands,
	enemy_assets: Res<EnemyAssetRegistry>,
	query: Query<(Entity, &Enemy, Option<&EnemyFireOverride>), Without<EnemyShooter>>,
) {
	for (entity, enemy, override_config) in query.iter() {
		let Some((projectile_type, fire_rate)) = enemy.enemy_type.shooting_config() else {
			continue;
		};

		let mut base_config = enemy.enemy_type.default_fire_config().unwrap_or(EnemyFireConfig {
			aim: AimMode::AtPlayer,
			pattern: FirePattern::Single,
			cooldown: fire_rate,
			sockets: SocketSelector::All,
		});

		if let Some(meta) = enemy_assets.get(enemy.enemy_type) {
			if let Some(cooldown) = meta.fire_cooldown {
				base_config.cooldown = cooldown.max(0.05);
			}
		}

		let fire_config = override_config
			.map(|override_config| base_config.apply_overrides(&override_config.overrides))
			.unwrap_or(base_config);

		let burst_interval = match fire_config.pattern {
			FirePattern::Burst { interval, .. } => interval.max(0.01),
			_ => 0.1,
		};

		commands.entity(entity).insert(EnemyShooter {
			projectile_type,
			fire_timer: Timer::from_seconds(fire_config.cooldown, TimerMode::Repeating),
			burst_remaining: 0,
			burst_timer: Timer::from_seconds(burst_interval, TimerMode::Repeating),
			fire_config,
		});
	}
}

pub fn enemy_shooting(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	projectile_materials: Res<ProjectileMaterialHandles>,
	mut shooters: Query<(&Transform, &Enemy, &mut EnemyShooter, Option<&EnemyWeaponSockets>), With<Enemy>>,
	player_query: Query<(&Transform, Option<&PlayerVelocity>), With<Player>>,
	time: Res<Time>,
) {
	let Ok((player_transform, player_velocity)) = player_query.get_single() else { return };
	let player_pos = player_transform.translation.truncate();
	let player_vel = player_velocity.map(|v| v.0).unwrap_or(Vec2::ZERO);

	for (transform, enemy, mut shooter, sockets) in shooters.iter_mut() {
		shooter.fire_timer.tick(time.delta());
		shooter.burst_timer.tick(time.delta());

		let config = shooter.projectile_type.config();
		let fire_config = shooter.fire_config.clone();

		let mut emit = |commands: &mut Commands| {
			let socket_count = emit_projectiles(
				commands,
				&asset_server,
				&mut meshes,
				&projectile_materials,
				transform,
				sockets,
				&fire_config,
				player_pos,
				player_vel,
				&shooter.projectile_type,
				&config,
			);
			if socket_count > 0 && std::env::var("TYDUST_LOG_ENEMY_FIRE").is_ok() {
				info!(
					"Enemy fire {:?}: aim={:?} pattern={:?} sockets={}",
					enemy.enemy_type,
					fire_config.aim,
					fire_config.pattern,
					socket_count
				);
			}
		};

		match fire_config.pattern {
			FirePattern::Burst { count, interval } => {
				if shooter.burst_remaining > 0 && shooter.burst_timer.just_finished() {
					emit(&mut commands);
					shooter.burst_remaining = shooter.burst_remaining.saturating_sub(1);
				}

				if shooter.fire_timer.just_finished() {
					shooter.burst_remaining = count;
					shooter.burst_timer = Timer::from_seconds(interval.max(0.01), TimerMode::Repeating);
					shooter.burst_timer.reset();
				}
			}
			_ => {
				if shooter.fire_timer.just_finished() {
					emit(&mut commands);
				}
			}
		}
	}
}

fn emit_projectiles(
	commands: &mut Commands,
	asset_server: &AssetServer,
	meshes: &mut Assets<Mesh>,
	projectile_materials: &ProjectileMaterialHandles,
	transform: &Transform,
	sockets: Option<&EnemyWeaponSockets>,
	fire_config: &EnemyFireConfig,
	player_pos: Vec2,
	player_vel: Vec2,
	projectile_type: &EnemyProjectileType,
	config: &crate::components::EnemyProjectileConfig,
) -> usize {
	let resolved_sockets = resolve_sockets(transform, sockets, &fire_config.sockets);
	let socket_count = resolved_sockets.len();
	let pattern_angles = pattern_angles(&fire_config.pattern);

	for socket in resolved_sockets {
		let aim_dir = match fire_config.aim {
			AimMode::AtPlayer => (player_pos - socket.world_pos).normalize_or_zero(),
			AimMode::LeadPlayer { lead_strength } => {
				let distance = socket.world_pos.distance(player_pos);
				let travel_time = if config.speed > 0.0 { distance / config.speed } else { 0.0 };
				let target = player_pos + player_vel * travel_time * lead_strength;
				(target - socket.world_pos).normalize_or_zero()
			}
			AimMode::FixedAngle { angle_deg } => {
				let angle = angle_deg.to_radians();
				Vec2::new(angle.cos(), angle.sin())
			}
		};

		let base_angle = aim_dir.y.atan2(aim_dir.x) + socket.angle_offset;

		let sprite_size = config.size;

		for angle_offset in &pattern_angles {
			let angle = base_angle + angle_offset;
			let velocity = Vec2::new(angle.cos(), angle.sin()) * config.speed;

			match projectile_type {
				EnemyProjectileType::BasicShot => {
					let mesh = meshes.add(Mesh::from(bevy::math::primitives::Rectangle::new(
						sprite_size.x,
						sprite_size.y,
					)));
					let material = projectile_materials.orange_pellet.clone();
					commands.spawn((
						Mesh2d(mesh),
						MeshMaterial2d(material),
						Transform::from_xyz(socket.world_pos.x, socket.world_pos.y, 0.6)
							.with_rotation(Quat::from_rotation_z(angle - FRAC_PI_2)),
						EnemyProjectile {
							damage: config.damage,
							velocity,
							lifetime: Timer::from_seconds(5.0, TimerMode::Once),
						},
					));
				}
				_ => {
					// Select sprite based on projectile type
					let (sprite_path, sprite_size) = match projectile_type {
						EnemyProjectileType::PlasmaBall => ("sprites/enemy_projectiles/plasma_ball.png", Vec2::splat(48.0)),
						EnemyProjectileType::SpreadShot => ("sprites/enemy_projectiles/spread_shot.png", Vec2::splat(24.0)),
						_ => ("sprites/enemy_projectiles/basic_shot.png", Vec2::splat(32.0)),
					};
					commands.spawn((
						Sprite {
							image: asset_server.load(sprite_path),
							custom_size: Some(sprite_size),
							..default()
						},
						Transform::from_xyz(socket.world_pos.x, socket.world_pos.y, 0.6)
							.with_rotation(Quat::from_rotation_z(angle - FRAC_PI_2)),
						EnemyProjectile {
							damage: config.damage,
							velocity,
							lifetime: Timer::from_seconds(5.0, TimerMode::Once),
						},
					));
				}
			}
		}
	}

	socket_count
}

struct ResolvedSocket {
	world_pos: Vec2,
	angle_offset: f32,
}

fn resolve_sockets(
	transform: &Transform,
	sockets: Option<&EnemyWeaponSockets>,
	selector: &SocketSelector,
) -> Vec<ResolvedSocket> {
	let mut candidates: Vec<&WeaponSocket> = Vec::new();
	if let Some(sockets) = sockets {
		match selector {
			SocketSelector::All => {
				candidates = sockets.sockets.iter().collect();
			}
			SocketSelector::ById { ids } => {
				candidates = sockets
					.sockets
					.iter()
					.filter(|socket| ids.iter().any(|id| id == &socket.id))
					.collect();
			}
			SocketSelector::ByTag { tags } => {
				candidates = sockets
					.sockets
					.iter()
					.filter(|socket| socket.tags.iter().any(|tag| tags.contains(tag)))
					.collect();
			}
		}
	}

	if candidates.is_empty() {
		return vec![ResolvedSocket {
			world_pos: transform.translation.truncate(),
			angle_offset: 0.0,
		}];
	}

	candidates
		.into_iter()
		.map(|socket| {
			let rotated = transform.rotation * socket.local_offset.extend(0.0);
			ResolvedSocket {
				world_pos: transform.translation.truncate() + rotated.truncate(),
				angle_offset: socket.angle_deg.unwrap_or(0.0).to_radians(),
			}
		})
		.collect()
}

fn pattern_angles(pattern: &FirePattern) -> Vec<f32> {
	match *pattern {
		FirePattern::Single => vec![0.0],
		FirePattern::Spread { count, angle_deg } => {
			let count = count.max(1) as i32;
			let spread = angle_deg.to_radians();
			let half_spread = spread / 2.0;
			if count == 1 {
				vec![0.0]
			} else {
				(0..count)
					.map(|i| -half_spread + (i as f32 / (count - 1) as f32) * spread)
					.collect()
			}
		}
		FirePattern::Ring { count } => {
			let count = count.max(1) as i32;
			(0..count)
				.map(|i| (i as f32 / count as f32) * std::f32::consts::TAU)
				.collect()
		}
		FirePattern::Burst { .. } => vec![0.0],
	}
}

pub fn move_enemy_projectiles(
	mut commands: Commands,
	mut query: Query<(Entity, &mut Transform, &mut EnemyProjectile)>,
	time: Res<Time>,
) {
	let delta = time.delta_secs();

	for (entity, mut transform, mut projectile) in query.iter_mut() {
		projectile.lifetime.tick(time.delta());

		if projectile.lifetime.finished() {
			commands.entity(entity).despawn();
			continue;
		}

		transform.translation.x += projectile.velocity.x * delta;
		transform.translation.y += projectile.velocity.y * delta;

		// Despawn if off screen
		if transform.translation.y < -(HALF_WORLD_HEIGHT + 100.0)
			|| transform.translation.y > HALF_WORLD_HEIGHT + 100.0
			|| transform.translation.x.abs() > 800.0
		{
			commands.entity(entity).despawn();
		}
	}
}

// === Enemy Rotation System ===

pub fn init_enemy_rotation(
	mut commands: Commands,
	query: Query<(Entity, &Transform), (With<Enemy>, Without<EnemyPreviousPosition>)>,
) {
	for (entity, transform) in query.iter() {
		commands.entity(entity).insert(EnemyPreviousPosition(transform.translation));
	}
}

pub fn rotate_enemies_to_movement(
	mut query: Query<(&mut Transform, &mut EnemyPreviousPosition, &Enemy), With<Enemy>>,
	time: Res<Time>,
) {
	let delta = time.delta_secs();
	if delta < 0.001 { return; }

	for (mut transform, mut prev_pos, enemy) in query.iter_mut() {
		let current_pos = transform.translation;
		let movement = current_pos - prev_pos.0;

		// Asteroids tumble slowly instead of facing movement direction
		if matches!(enemy.enemy_type, EnemyType::SmallAsteroid | EnemyType::MediumAsteroid | EnemyType::LargeAsteroid | EnemyType::StationDebris) {
			// Slow tumble rotation for asteroids
			let tumble_speed = 0.3;  // radians per second
			let current_rotation = transform.rotation.to_euler(bevy::math::EulerRot::XYZ).2;
			let new_rotation = current_rotation + tumble_speed * delta;
			transform.rotation = Quat::from_rotation_z(new_rotation);
		} else {
			// Ships rotate to face movement direction
			// Only rotate if moving significantly
			if movement.length() > 0.5 {
				let direction = movement.truncate();
				// Calculate angle (0 degrees = pointing up)
				let mut target_angle = direction.y.atan2(direction.x) - FRAC_PI_2;
				// Some sprites are authored "pointing down" at 0° (instead of up).
				// Apply a per-type facing correction so they still face movement correctly.
				if matches!(enemy.enemy_type, EnemyType::Drill | EnemyType::ScoutSting) {
					target_angle += std::f32::consts::PI;
				}
				transform.rotation = Quat::from_rotation_z(target_angle);
			}
		}

		prev_pos.0 = current_pos;
	}
}

pub fn shimmer_enemies(
	mut query: Query<(&mut Sprite, Option<&crate::components::FxPolicy>)>,
	time: Res<Time>,
) {
	let elapsed = time.elapsed_secs();

	for (mut sprite, policy) in query.iter_mut() {
		if !matches!(policy.map(|p| p.idle), Some(crate::components::IdleFx::SpriteShimmer)) {
			continue;
		}
		// Shimmer effect - pulsing between 0.8 and 1.3
		let shimmer = 1.05 + ((elapsed * 3.5).sin() * 0.25);
		sprite.color = Color::srgba(shimmer, shimmer, shimmer, 1.0);
	}
}
