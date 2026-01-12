use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::components::{
	Enemy, Player, Projectile, Collider, Health, PlayerDefenses, DamageSink,
	Invincible, ContactDamage, EnemyHitEvent, EnemyDeathEvent, PlayerHitEvent,
	EnemyProjectile,
};
use crate::systems::level::GamePaused;

const SHIELD2_REGEN_DELAY_SECS: f64 = 3.0;
const SHIELD2_REGEN_DURATION_SECS: f64 = 1.5;

pub fn check_projectile_enemy_collisions(
	mut commands: Commands,
	projectiles: Query<(Entity, &Transform, &Projectile)>,
	enemies: Query<(Entity, &Transform, &Collider, &Enemy), Without<Player>>,
	mut hit_events: EventWriter<EnemyHitEvent>,
) {
	for (proj_entity, proj_transform, projectile) in projectiles.iter() {
		let proj_pos = proj_transform.translation.truncate();
		let proj_radius = projectile.damage.sqrt() * 2.0; // Rough projectile size from damage

		for (enemy_entity, enemy_transform, collider, _enemy) in enemies.iter() {
			let enemy_pos = enemy_transform.translation.truncate();
			let distance = proj_pos.distance(enemy_pos);

			if distance < proj_radius + collider.radius {
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
	mut enemies: Query<(&mut Health, &Transform, &Enemy)>,
	mut death_events: EventWriter<EnemyDeathEvent>,
) {
	for event in hit_events.read() {
		if let Ok((mut health, transform, enemy)) = enemies.get_mut(event.enemy) {
			health.current -= event.damage;

			if enemy.enemy_type == crate::components::EnemyType::Boss {
				info!("💥 Boss hit! Damage: {:.1}, HP: {:.1}/{:.1}", event.damage, health.current, health.max);
			}

			if health.current <= 0.0 {
				if enemy.enemy_type == crate::components::EnemyType::Boss {
					info!("☠️  BOSS DESTROYED!");
				}
				death_events.send(EnemyDeathEvent {
					position: transform.translation.truncate(),
					enemy_type: enemy.enemy_type,
				});
				commands.entity(event.enemy).despawn();
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

			if let Ok(mut defenses) = player_defenses.get_single_mut() {
				// Any hit resets shield regen cooldown/state.
				defenses.last_damage_time = time.elapsed_secs_f64();
				defenses.shield2_regen_start_time = None;
				defenses.shield2_regen_from = defenses.shield2;

				let hit_result = defenses.take_damage(damage);
				info!("Player hit for {:.0} damage! Hit: {:?}, Armor: {:.0}/{:.0}",
					damage, hit_result, defenses.armor, defenses.armor_max);

				if hit_result == DamageSink::Dead {
					info!("Player armor destroyed! Game Over!");
				}
			}

			hit_events.send(PlayerHitEvent);
			commands.entity(player_entity).insert(Invincible::new(1.0));
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
			if let Ok(mut defenses) = player_defenses.get_single_mut() {
				// Any hit resets shield regen cooldown/state.
				defenses.last_damage_time = time.elapsed_secs_f64();
				defenses.shield2_regen_start_time = None;
				defenses.shield2_regen_from = defenses.shield2;

				let hit_result = defenses.take_damage(projectile.damage);
				info!("Player hit by projectile for {:.0} damage! Hit: {:?}, Armor: {:.0}/{:.0}",
					projectile.damage, hit_result, defenses.armor, defenses.armor_max);

				if hit_result == DamageSink::Dead {
					info!("Player armor destroyed! Game Over!");
				}
			}

			commands.entity(proj_entity).despawn();
			hit_events.send(PlayerHitEvent);
			commands.entity(player_entity).insert(Invincible::new(1.0));
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

pub fn play_enemy_hit_sound(
	mut hit_events: EventReader<EnemyHitEvent>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	sound_volume: Res<crate::systems::level::SoundVolume>,
) {
	for event in hit_events.read() {
		// Skip if no sound specified (used for silent continuous damage)
		if let Some(sound_path) = event.hit_sound {
			let base_volume = match sound_path {
				"sounds/lightning/lightning_wave_light.ogg" => 0.4,
				"sounds/lightning/deep_lightning_boom.ogg" => 0.5,
				"sounds/lightning/fireworks_crackle.ogg" => 0.4,
				_ => 0.6,
			};
			audio.play(asset_server.load(sound_path)).with_volume(sound_volume.apply(base_volume));
		}
	}
}

pub fn play_enemy_death_sound(
	mut death_events: EventReader<EnemyDeathEvent>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	sound_volume: Res<crate::systems::level::SoundVolume>,
) {
	for _ in death_events.read() {
		audio.play(asset_server.load("sounds/enemy_death.ogg")).with_volume(sound_volume.apply(0.8));
	}
}
