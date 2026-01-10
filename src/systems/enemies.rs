use bevy::prelude::*;
use crate::components::{Enemy, EnemyMovement, MovementPattern, Player, EnemyBehavior, BehaviorType, SineAxis, EasingType, FormationLeader, FormationMember, EnemyShooter, EnemyProjectile, EnemyPreviousPosition, EnemyProjectileType};
use super::world::HALF_WORLD_HEIGHT;
use super::level::CurrentLevel;
use std::f32::consts::{PI, FRAC_PI_2};

pub fn update_enemy_movement(
	mut query: Query<(&mut Transform, &mut EnemyMovement)>,
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
	query: Query<(Entity, &Transform, &Enemy), With<Enemy>>,
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
	mut query: Query<(&mut Transform, &mut EnemyBehavior, &mut Sprite), (Without<Player>, Without<FormationLeader>)>,
	time: Res<Time>,
	player_query: Query<&Transform, With<Player>>,
	formation_query: Query<&Transform, With<FormationLeader>>,
	level: Option<Res<CurrentLevel>>,
) {
	let delta = time.delta_secs();
	let scroll_speed = level
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
			&formation_query,
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
	_formation_query: &Query<&Transform, With<FormationLeader>>,
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
					_formation_query,
				);
			}
		}
	}
}

pub fn update_formations(
	leader_query: Query<(&Transform, &FormationLeader)>,
	mut member_query: Query<(&mut Transform, &FormationMember), Without<FormationLeader>>,
) {
	for (mut member_transform, member) in member_query.iter_mut() {
		if let Ok((leader_transform, _)) = leader_query.get(member.leader) {
			let target_pos = leader_transform.translation.truncate() + member.offset;
			member_transform.translation = target_pos.extend(member_transform.translation.z);
		}
	}
}

// === Enemy Shooting System ===

pub fn setup_enemy_shooters(
	mut commands: Commands,
	query: Query<(Entity, &Enemy), Without<EnemyShooter>>,
) {
	for (entity, enemy) in query.iter() {
		if let Some((projectile_type, fire_rate)) = enemy.enemy_type.shooting_config() {
			commands.entity(entity).insert(EnemyShooter {
				projectile_type,
				fire_timer: Timer::from_seconds(fire_rate, TimerMode::Repeating),
				burst_remaining: 0,
				burst_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
			});
		}
	}
}

pub fn enemy_shooting(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut shooters: Query<(&Transform, &mut EnemyShooter), With<Enemy>>,
	player_query: Query<&Transform, With<Player>>,
	time: Res<Time>,
) {
	let Ok(player_transform) = player_query.get_single() else { return };
	let player_pos = player_transform.translation.truncate();

	for (transform, mut shooter) in shooters.iter_mut() {
		let enemy_pos = transform.translation.truncate();
		shooter.fire_timer.tick(time.delta());
		shooter.burst_timer.tick(time.delta());

		let config = shooter.projectile_type.config();

		// Handle burst shooting
		if shooter.burst_remaining > 0 && shooter.burst_timer.just_finished() {
			spawn_enemy_projectiles(&mut commands, &asset_server, enemy_pos, player_pos, &shooter, &config);
			shooter.burst_remaining -= 1;
		}

		// Handle regular fire timer
		if shooter.fire_timer.just_finished() {
			if config.burst_count > 1 {
				shooter.burst_remaining = config.burst_count;
				shooter.burst_timer.reset();
			} else {
				spawn_enemy_projectiles(&mut commands, &asset_server, enemy_pos, player_pos, &shooter, &config);
			}
		}
	}
}

fn spawn_enemy_projectiles(
	commands: &mut Commands,
	asset_server: &AssetServer,
	enemy_pos: Vec2,
	player_pos: Vec2,
	shooter: &EnemyShooter,
	config: &crate::components::EnemyProjectileConfig,
) {
	let to_player = (player_pos - enemy_pos).normalize_or_zero();
	let base_angle = to_player.y.atan2(to_player.x);

	let count = config.count as i32;
	let half_spread = config.spread_angle / 2.0;

	// Select sprite based on projectile type
	let (sprite_path, sprite_size) = match shooter.projectile_type {
		EnemyProjectileType::PlasmaBall => ("sprites/enemy_projectiles/plasma_ball.png", Vec2::splat(48.0)),
		EnemyProjectileType::SpreadShot => ("sprites/enemy_projectiles/spread_shot.png", Vec2::splat(24.0)),
		_ => ("sprites/enemy_projectiles/basic_shot.png", Vec2::splat(32.0)),
	};

	for i in 0..count {
		let angle_offset = if count == 1 {
			0.0
		} else if config.spread_angle >= std::f32::consts::TAU - 0.1 {
			// Full circle (Ring pattern)
			(i as f32 / count as f32) * std::f32::consts::TAU
		} else {
			// Spread pattern
			-half_spread + (i as f32 / (count - 1).max(1) as f32) * config.spread_angle
		};

		let angle = base_angle + angle_offset;
		let velocity = Vec2::new(angle.cos(), angle.sin()) * config.speed;

		commands.spawn((
			Sprite {
				image: asset_server.load(sprite_path),
				custom_size: Some(sprite_size),
				..default()
			},
			Transform::from_xyz(enemy_pos.x, enemy_pos.y, 0.6)
				.with_rotation(Quat::from_rotation_z(angle - FRAC_PI_2)),
			EnemyProjectile {
				damage: config.damage,
				velocity,
				lifetime: Timer::from_seconds(5.0, TimerMode::Once),
			},
		));
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
	mut query: Query<(&mut Transform, &mut EnemyPreviousPosition), With<Enemy>>,
	time: Res<Time>,
) {
	let delta = time.delta_secs();
	if delta < 0.001 { return; }

	for (mut transform, mut prev_pos) in query.iter_mut() {
		let current_pos = transform.translation;
		let movement = current_pos - prev_pos.0;

		// Only rotate if moving significantly
		if movement.length() > 0.5 {
			let direction = movement.truncate();
			// Calculate angle (0 degrees = pointing up)
			let target_angle = direction.y.atan2(direction.x) - FRAC_PI_2;
			transform.rotation = Quat::from_rotation_z(target_angle);
		}

		prev_pos.0 = current_pos;
	}
}

pub fn shimmer_enemies(
	mut query: Query<(&mut Sprite, &Enemy)>,
	time: Res<Time>,
) {
	let elapsed = time.elapsed_secs();

	for (mut sprite, _enemy) in query.iter_mut() {
		// Shimmer effect - pulsing between 0.8 and 1.3
		let shimmer = 1.05 + ((elapsed * 3.5).sin() * 0.25);
		sprite.color = Color::srgba(shimmer, shimmer, shimmer, 1.0);
	}
}
