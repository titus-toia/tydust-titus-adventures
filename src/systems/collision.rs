use bevy::prelude::*;
use crate::components::{
	Enemy, Player, Projectile, Collider, Health, PlayerDefenses, DamageSink,
	Invincible, ContactDamage, EnemyHitEvent, EnemyDeathEvent, PlayerHitEvent,
	EnemyProjectile, ProjectileHitbox, HitboxShape, CapsuleAxis,
};
use crate::systems::level::GamePaused;
use crate::systems::audio::PlaySfxEvent;

const SHIELD2_REGEN_DELAY_SECS: f64 = 2.0;
const SHIELD2_REGEN_DURATION_SECS: f64 = 1.5;
const SHIELD1_REGEN_PER_SEC: f32 = 5.0;
const DEFAULT_ENEMY_DEATH_CAP: u8 = 3;

pub fn check_projectile_enemy_collisions(
	mut commands: Commands,
	projectiles: Query<(Entity, &Transform, &Projectile)>,
	enemies: Query<(Entity, &Transform, &Collider, Option<&ProjectileHitbox>, &Enemy), Without<Player>>,
	mut hit_events: EventWriter<EnemyHitEvent>,
) {
	for (proj_entity, proj_transform, projectile) in projectiles.iter() {
		let proj_pos = proj_transform.translation.truncate();
		let proj_radius = projectile.damage.sqrt() * 2.0; // Rough projectile size from damage

		for (enemy_entity, enemy_transform, collider, projectile_hitbox, _enemy) in enemies.iter() {
			let enemy_pos = enemy_transform.translation.truncate();
			let hit = if let Some(hitbox) = projectile_hitbox {
				let inv_rotation = enemy_transform.rotation.conjugate();
				let local_proj = inv_rotation * Vec3::new(
					proj_pos.x - enemy_pos.x,
					proj_pos.y - enemy_pos.y,
					0.0,
				);
				let local = Vec2::new(local_proj.x, local_proj.y) - hitbox.offset;

				match hitbox.shape {
					HitboxShape::Circle { radius } => {
						local.length_squared() <= (proj_radius + radius).powi(2)
					}
					HitboxShape::Ellipse { radii } => {
						let rx = radii.x + proj_radius;
						let ry = radii.y + proj_radius;
						if rx <= 0.0 || ry <= 0.0 {
							false
						} else {
							(local.x * local.x) / (rx * rx) + (local.y * local.y) / (ry * ry) <= 1.0
						}
					}
					HitboxShape::Capsule { radius, half_length, axis } => {
						let effective_radius = radius + proj_radius;
						let (dx, dy) = match axis {
							CapsuleAxis::Vertical => {
								let clamped_y = local.y.clamp(-half_length, half_length);
								(local.x, local.y - clamped_y)
							}
							CapsuleAxis::Horizontal => {
								let clamped_x = local.x.clamp(-half_length, half_length);
								(local.x - clamped_x, local.y)
							}
						};
						dx * dx + dy * dy <= effective_radius * effective_radius
					}
				}
			} else {
				let distance = proj_pos.distance(enemy_pos);
				distance < proj_radius + collider.radius
			};

			if hit {
				hit_events.send(EnemyHitEvent {
					enemy: enemy_entity,
					damage: projectile.damage,
					hit_sound: None,
				});
				commands.entity(proj_entity).despawn();
				break;
			}
		}
	}
}

pub fn apply_enemy_damage(
	mut commands: Commands,
	mut hit_events: EventReader<EnemyHitEvent>,
	mut enemies: Query<(Entity, &mut Health, &Transform, &Enemy), Without<crate::components::Dying>>,
	mut death_events: EventWriter<EnemyDeathEvent>,
) {
	for event in hit_events.read() {
		if let Ok((entity, mut health, transform, enemy)) = enemies.get_mut(event.enemy) {
			health.current -= event.damage;

			if enemy.enemy_type == crate::components::EnemyType::Boss {
				info!("💥 Boss hit! Damage: {:.1}, HP: {:.1}/{:.1}", event.damage, health.current, health.max);
			}

			if health.current <= 0.0 {
				if enemy.enemy_type == crate::components::EnemyType::Boss {
					info!("☠️  BOSS DESTROYED!");
				}
				death_events.send(EnemyDeathEvent {
					entity,
					position: transform.translation.truncate(),
					enemy_type: enemy.enemy_type,
				});
				// Presentation (dissolve/particles/despawn) is owned by the centralized DeathFX system.
				// Here we only mark the entity as non-interactive immediately.
				commands.entity(entity)
					.insert(crate::components::Dying)
					.remove::<Collider>();
				commands.entity(entity)
					.remove::<ProjectileHitbox>();
			}
		}
	}
}

pub fn check_player_enemy_collisions(
	mut commands: Commands,
	player_query: Query<(Entity, &Transform, &Collider), (With<Player>, Without<Invincible>)>,
	enemies: Query<(&Transform, &Collider, &Enemy), Without<Player>>,
	mut player_defenses: Query<&mut PlayerDefenses>,
	mut hit_events: EventWriter<PlayerHitEvent>,
	time: Res<Time>,
) {
	let Ok((player_entity, player_transform, player_collider)) = player_query.get_single() else {
		return;
	};
	let player_pos = player_transform.translation.truncate();

	for (enemy_transform, enemy_collider, enemy) in enemies.iter() {
		let enemy_pos = enemy_transform.translation.truncate();
		let distance = player_pos.distance(enemy_pos);

		if distance < player_collider.radius + enemy_collider.radius {
			let damage = ContactDamage::for_enemy_type(enemy.enemy_type);

			let mut sink = DamageSink::Armor;
			let mut depleted = false;

			if let Ok(mut defenses) = player_defenses.get_single_mut() {
				// Any hit resets shield regen cooldown/state.
				defenses.last_damage_time = time.elapsed_secs_f64();
				defenses.shield2_regen_start_time = None;
				defenses.shield2_regen_from = defenses.shield2;

				sink = defenses.take_damage(damage);
				depleted = match sink {
					DamageSink::Shield2 => defenses.shield2 <= 0.0,
					DamageSink::Shield1 => defenses.shield1 <= 0.0,
					DamageSink::Armor => defenses.armor <= 0.0,
					DamageSink::Dead => true,
				};
				info!("Player hit for {:.0} damage! Hit: {:?}, Armor: {:.0}/{:.0}",
					damage, sink, defenses.armor, defenses.armor_max);

				if sink == DamageSink::Dead {
					info!("Player armor destroyed! Game Over!");
				}
			}

			hit_events.send(PlayerHitEvent { sink, depleted });
			commands.entity(player_entity).insert(Invincible::new(0.05));
			break;
		}
	}
}

pub fn update_invincibility(
	mut commands: Commands,
	mut query: Query<(Entity, &mut Invincible, &mut Sprite)>,
	time: Res<Time>,
) {
	for (entity, mut invincible, mut sprite) in query.iter_mut() {
		invincible.timer.tick(time.delta());

		// Flash effect during invincibility
		let alpha = if (time.elapsed_secs() * 10.0).sin() > 0.0 { 1.0 } else { 0.3 };
		sprite.color.set_alpha(alpha);

		if invincible.timer.finished() {
			sprite.color.set_alpha(1.0);
			commands.entity(entity).remove::<Invincible>();
		}
	}
}

pub fn check_enemy_projectile_player_collisions(
	mut commands: Commands,
	projectiles: Query<(Entity, &Transform, &EnemyProjectile)>,
	player_query: Query<(Entity, &Transform, &Collider), (With<Player>, Without<Invincible>)>,
	mut player_defenses: Query<&mut PlayerDefenses>,
	mut hit_events: EventWriter<PlayerHitEvent>,
	time: Res<Time>,
) {
	let Ok((player_entity, player_transform, player_collider)) = player_query.get_single() else {
		return;
	};
	let player_pos = player_transform.translation.truncate();

	for (proj_entity, proj_transform, projectile) in projectiles.iter() {
		let proj_pos = proj_transform.translation.truncate();
		let proj_radius = 5.0; // Small collision radius for projectiles

		let distance = player_pos.distance(proj_pos);

		if distance < proj_radius + player_collider.radius {
			let mut sink = DamageSink::Armor;
			let mut depleted = false;

			if let Ok(mut defenses) = player_defenses.get_single_mut() {
				// Any hit resets shield regen cooldown/state.
				defenses.last_damage_time = time.elapsed_secs_f64();
				defenses.shield2_regen_start_time = None;
				defenses.shield2_regen_from = defenses.shield2;

				sink = defenses.take_damage(projectile.damage);
				depleted = match sink {
					DamageSink::Shield2 => defenses.shield2 <= 0.0,
					DamageSink::Shield1 => defenses.shield1 <= 0.0,
					DamageSink::Armor => defenses.armor <= 0.0,
					DamageSink::Dead => true,
				};
				info!("Player hit by projectile for {:.0} damage! Hit: {:?}, Armor: {:.0}/{:.0}",
					projectile.damage, sink, defenses.armor, defenses.armor_max);

				if sink == DamageSink::Dead {
					info!("Player armor destroyed! Game Over!");
				}
			}

			commands.entity(proj_entity).despawn();
			hit_events.send(PlayerHitEvent { sink, depleted });
			commands.entity(player_entity).insert(Invincible::new(0.05));
			break; // Only one hit per frame
		}
	}
}

/// Regenerate the outer shield (shield2) if the player hasn't been hit recently.
///
/// Behavior:
/// - Wait `SHIELD2_REGEN_DELAY_SECS` after the last hit (any hit) before starting regen.
/// - Once started, regen eases from the current value to max over `SHIELD2_REGEN_DURATION_SECS`.
/// - Easing is quadratic ease-in: slow at first, then faster near the end.
pub fn update_shield2_regen(
	time: Res<Time>,
	paused: Res<GamePaused>,
	mut player_defenses: Query<&mut PlayerDefenses, With<Player>>,
) {
	if paused.0 {
		return;
	}

	let Ok(mut defenses) = player_defenses.get_single_mut() else { return };

	// No need to regen if already full (or max is invalid).
	if defenses.shield2_max <= 0.0 || defenses.shield2 >= defenses.shield2_max {
		defenses.shield2 = defenses.shield2.clamp(0.0, defenses.shield2_max.max(0.0));
		defenses.shield2_regen_start_time = None;
		return;
	}

	let now = time.elapsed_secs_f64();
	let since_hit = now - defenses.last_damage_time;

	// Cooldown window: do nothing.
	if since_hit < SHIELD2_REGEN_DELAY_SECS {
		defenses.shield2_regen_start_time = None;
		defenses.shield2_regen_from = defenses.shield2;
		return;
	}

	// Start regen if needed.
	if defenses.shield2_regen_start_time.is_none() {
		defenses.shield2_regen_from = defenses.shield2;
		defenses.shield2_regen_start_time = Some(now);
	}
	let start = defenses.shield2_regen_start_time.unwrap_or(now);

	let t = ((now - start) / SHIELD2_REGEN_DURATION_SECS).clamp(0.0, 1.0) as f32;
	let eased = t * t; // quadratic ease-in (slow → fast)
	defenses.shield2 = defenses.shield2_regen_from
		+ (defenses.shield2_max - defenses.shield2_regen_from) * eased;

	if t >= 1.0 {
		defenses.shield2 = defenses.shield2_max;
		defenses.shield2_regen_start_time = None;
	}
}

/// Regenerate the inner shield (shield1) constantly at a fixed rate.
///
/// Behavior:
/// - Always regenerating at SHIELD1_REGEN_PER_SEC (no delay, even through damage)
/// - Provides consistent recovery against light/scattered damage
pub fn update_shield1_regen(
	time: Res<Time>,
	paused: Res<GamePaused>,
	mut player_defenses: Query<&mut PlayerDefenses, With<Player>>,
) {
	if paused.0 {
		return;
	}

	let Ok(mut defenses) = player_defenses.get_single_mut() else { return };

	// No need to regen if already full (or max is invalid).
	if defenses.shield1_max <= 0.0 || defenses.shield1 >= defenses.shield1_max {
		defenses.shield1 = defenses.shield1.clamp(0.0, defenses.shield1_max.max(0.0));
		return;
	}

	// Constant regen - no delay, always active
	defenses.shield1 = (defenses.shield1 + SHIELD1_REGEN_PER_SEC * time.delta_secs())
		.min(defenses.shield1_max);
}

pub fn play_enemy_hit_sound(
	mut hit_events: EventReader<EnemyHitEvent>,
	mut sfx_events: EventWriter<PlaySfxEvent>,
) {
	for event in hit_events.read() {
		// Skip if no sound specified (used for silent continuous damage)
		if let Some(sound_path) = event.hit_sound {
			let (volume, priority, cooldown) = match sound_path {
				// Lightning hits are important but shouldn't drown the charged fire sound.
				"sounds/lightning/lightning_wave_light.ogg" => (0.35, 120, 0.04),
				"sounds/lightning/deep_lightning_boom.ogg" => (0.55, 160, 0.10),
				"sounds/lightning/fireworks_crackle.ogg" => (0.35, 110, 0.06),
				_ => (0.55, 70, 0.03),
			};
			sfx_events.send(PlaySfxEvent::simple(sound_path, volume, priority, cooldown));
		}
	}
}

pub fn play_enemy_death_sound(
	mut death_events: EventReader<EnemyDeathEvent>,
	mut sfx_events: EventWriter<PlaySfxEvent>,
) {
	for _ in death_events.read() {
		sfx_events.send(PlaySfxEvent {
			sound_path: "sounds/enemy_death.ogg",
			volume: 0.75,
			priority: 80,
			cooldown_secs: 0.02,
			max_concurrent: DEFAULT_ENEMY_DEATH_CAP,
			when_full: crate::systems::audio::SfxWhenFull::Reject,
			fade_after: None,
			fade_duration: 0.0,
		});
	}
}

pub fn play_player_hit_sound(
	mut hit_events: EventReader<PlayerHitEvent>,
	mut sfx_events: EventWriter<PlaySfxEvent>,
) {
	for event in hit_events.read() {
		let sound_path = match (event.sink, event.depleted) {
			(DamageSink::Shield2, false) => "sounds/player/shield2_hit.ogg",
			(DamageSink::Shield2, true) => "sounds/player/shield2_down.ogg",
			(DamageSink::Shield1, false) => "sounds/player/shield1_hit.ogg",
			(DamageSink::Shield1, true) => "sounds/player/shield1_down.ogg",
			(DamageSink::Armor, _) => "sounds/player/armor_hit.ogg",
			(DamageSink::Dead, _) => "sounds/player/armor_hit.ogg", // Could add death sound later
		};

		sfx_events.send(PlaySfxEvent::simple(sound_path, 0.7, 150, 0.05));
	}
}
