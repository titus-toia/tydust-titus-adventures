use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use crate::components::{
	Enemy, Player, Projectile, Collider, Health, PlayerHealth,
	Invincible, ContactDamage, EnemyHitEvent, EnemyDeathEvent, PlayerHitEvent,
	EnemyProjectile,
};

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

			if health.current <= 0.0 {
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
	mut player_health: Query<&mut PlayerHealth>,
	mut hit_events: EventWriter<PlayerHitEvent>,
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

			if let Ok(mut health) = player_health.get_single_mut() {
				health.current = (health.current - damage).max(0.0);
				info!("Player hit! HP: {:.0}/{:.0}", health.current, health.max);
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
	mut player_health: Query<&mut PlayerHealth>,
	mut hit_events: EventWriter<PlayerHitEvent>,
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
			if let Ok(mut health) = player_health.get_single_mut() {
				health.current = (health.current - projectile.damage).max(0.0);
				info!("Player hit by projectile! HP: {:.0}/{:.0}", health.current, health.max);
			}

			commands.entity(proj_entity).despawn();
			hit_events.send(PlayerHitEvent);
			commands.entity(player_entity).insert(Invincible::new(1.0));
			break; // Only one hit per frame
		}
	}
}

pub fn play_enemy_hit_sound(
	mut hit_events: EventReader<EnemyHitEvent>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
) {
	for _ in hit_events.read() {
		audio.play(asset_server.load("sounds/enemy_hit.ogg")).with_volume(0.6);
	}
}

pub fn play_enemy_death_sound(
	mut death_events: EventReader<EnemyDeathEvent>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
) {
	for _ in death_events.read() {
		audio.play(asset_server.load("sounds/enemy_death.ogg")).with_volume(0.8);
	}
}
