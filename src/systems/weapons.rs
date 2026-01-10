use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::Rng;
use crate::components::{Player, Weapon, Projectile, SineMotion, WeaponType, Particle, HomingProjectile, OrbitalEntity, Enemy, AngledShot, ChargeMeter, Health, Collider, EnemyHitEvent};
use super::world::{HALF_WORLD_HEIGHT};
use super::lightning;
use std::f32::consts::{PI, FRAC_PI_2};

const PROJECTILE_Z: f32 = 0.5;
const PROJECTILE_LIFETIME: f32 = 3.0;

pub fn fire_weapons(
	keyboard_input: Res<ButtonInput<KeyCode>>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
	mut query: Query<(&Transform, &mut Weapon), With<Player>>,
	time: Res<Time>,
	mut charge_meter: ResMut<ChargeMeter>,
	enemies: Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
	mut hit_events: EventWriter<EnemyHitEvent>,
) {
	if !keyboard_input.pressed(KeyCode::Space) {
		return;
	}

	for (transform, mut weapon) in query.iter_mut() {
		weapon.fire_cooldown.tick(time.delta());

		if weapon.fire_cooldown.finished() {
			let config = weapon.weapon_type.config();

			let cooldown = config.base_cooldown -
				(weapon.level as f32 * config.cooldown_reduction_per_level);
			let damage = config.base_damage +
				(weapon.level as f32 * config.damage_per_level);

			weapon.fire_cooldown.set_duration(
				std::time::Duration::from_secs_f32(cooldown.max(0.05))
			);

			let spawn_pos = transform.translation + Vec3::new(0.0, 55.0, 0.0);

			match weapon.weapon_type {
				WeaponType::BasicBlaster => {
					spawn_basic_projectile(&mut commands, &asset_server, spawn_pos, &weapon, &config, damage);
				},
				WeaponType::PlasmaCannon => {
					spawn_plasma_projectile(&mut commands, &asset_server, spawn_pos, &weapon, &config, damage);
				},
				WeaponType::WaveGun => {
					spawn_wave_projectile(&mut commands, &asset_server, spawn_pos, &weapon, &config, damage);
				},
				WeaponType::SpreadShot => {
					spawn_spread_projectiles(&mut commands, &asset_server, spawn_pos, &weapon, &config, damage);
				},
				WeaponType::MissilePods => {
					spawn_missile_projectiles(&mut commands, &asset_server, spawn_pos, &weapon, &config, damage);
				},
				WeaponType::LaserArray => {
					spawn_laser_beams(&mut commands, &asset_server, spawn_pos, &weapon, &config, damage);
				},
				WeaponType::OrbitalDefense => {
					// Orbital defense doesn't fire projectiles traditionally
					// It spawns/maintains orbs that are handled separately
				},
				WeaponType::LightningChain => {
					lightning::fire_lightning_weapon(
						&mut commands,
						&asset_server,
						&audio,
						spawn_pos,
						&weapon,
						damage,
						&mut charge_meter,
						&enemies,
						&mut hit_events,
					);
				},
			}

			spawn_muzzle_flash(&mut commands, &asset_server, spawn_pos, weapon.weapon_type);

			let sound_path = match weapon.weapon_type {
				WeaponType::BasicBlaster => "sounds/basic_blaster_fire.ogg",
				WeaponType::PlasmaCannon => "sounds/plasma_cannon_fire.ogg",
				WeaponType::WaveGun => "sounds/wave_gun_fire.ogg",
				WeaponType::SpreadShot => "sounds/spread_shot_fire.ogg",
				WeaponType::MissilePods => "sounds/missile_launch.ogg",
				WeaponType::LaserArray => "sounds/laser_array_fire.ogg",
				WeaponType::OrbitalDefense => "sounds/orbital_fire.ogg",
				WeaponType::LightningChain => "sounds/lightning_fire.ogg",
			};

			audio.play(asset_server.load(sound_path));
			weapon.fire_cooldown.reset();
		}
	}
}

fn spawn_basic_projectile(
	commands: &mut Commands,
	asset_server: &AssetServer,
	spawn_pos: Vec3,
	weapon: &Weapon,
	config: &crate::components::WeaponConfig,
	damage: f32,
) {
	commands.spawn((
		Sprite {
			image: asset_server.load("sprites/projectiles/basic_blaster.png"),
			custom_size: Some(Vec2::new(20.0, 60.0)),
			..default()
		},
		Transform::from_translation(spawn_pos.with_z(PROJECTILE_Z)),
		Projectile {
			weapon_type: weapon.weapon_type,
			level: weapon.level,
			speed: config.projectile_speed,
			damage,
			lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
		},
	));
}

fn spawn_plasma_projectile(
	commands: &mut Commands,
	asset_server: &AssetServer,
	spawn_pos: Vec3,
	weapon: &Weapon,
	config: &crate::components::WeaponConfig,
	damage: f32,
) {
	commands.spawn((
		Sprite {
			image: asset_server.load("sprites/projectiles/plasma_cannon.png"),
			custom_size: Some(Vec2::new(35.0, 80.0)),
			..default()
		},
		Transform::from_translation(spawn_pos.with_z(PROJECTILE_Z)),
		Projectile {
			weapon_type: weapon.weapon_type,
			level: weapon.level,
			speed: config.projectile_speed,
			damage,
			lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
		},
	));

	spawn_plasma_trail(commands, asset_server, spawn_pos);
}

fn spawn_wave_projectile(
	commands: &mut Commands,
	asset_server: &AssetServer,
	spawn_pos: Vec3,
	weapon: &Weapon,
	config: &crate::components::WeaponConfig,
	damage: f32,
) {
	let amplitude = 100.0 + (weapon.level as f32 * 30.0);
	let frequency = 2.0 + (weapon.level as f32 * 0.5);

	commands.spawn((
		Sprite {
			image: asset_server.load("sprites/projectiles/wave_gun.png"),
			custom_size: Some(Vec2::new(40.0, 90.0)),
			..default()
		},
		Transform::from_translation(spawn_pos.with_z(PROJECTILE_Z)),
		Projectile {
			weapon_type: weapon.weapon_type,
			level: weapon.level,
			speed: config.projectile_speed,
			damage,
			lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
		},
		SineMotion {
			amplitude,
			frequency,
			time_offset: 0.0,
			base_x: spawn_pos.x,
		},
	));
}

fn spawn_plasma_trail(
	commands: &mut Commands,
	asset_server: &AssetServer,
	position: Vec3,
) {
	let mut rng = rand::thread_rng();

	for _ in 0..3 {
		commands.spawn((
			Sprite {
				image: asset_server.load("particles/exhaust_cyan.png"),
				custom_size: Some(Vec2::splat(18.0)),
				..default()
			},
			Transform::from_translation(position),
			Particle {
				lifetime: Timer::from_seconds(0.3, TimerMode::Once),
				velocity: Vec2::new(
					rng.gen_range(-20.0..20.0),
					rng.gen_range(-50.0..-20.0),
				),
			},
		));
	}
}

fn spawn_muzzle_flash(
	commands: &mut Commands,
	asset_server: &AssetServer,
	position: Vec3,
	weapon_type: WeaponType,
) {
	let mut rng = rand::thread_rng();

	let (particle_sprite, count, size) = match weapon_type {
		WeaponType::BasicBlaster => ("particles/spark_white.png", 2, 10.0),
		WeaponType::PlasmaCannon => ("particles/exhaust_cyan.png", 5, 15.0),
		WeaponType::WaveGun => ("particles/spark_white.png", 3, 12.0),
		WeaponType::SpreadShot => ("particles/flame_orange.png", 4, 12.0),
		WeaponType::MissilePods => ("particles/exhaust_cyan.png", 3, 14.0),
		WeaponType::LaserArray => ("particles/spark_white.png", 6, 8.0),
		WeaponType::OrbitalDefense => ("particles/spark_white.png", 2, 10.0),
		WeaponType::LightningChain => ("particles/electric_arc.png", 5, 14.0),
	};

	for _ in 0..count {
		let offset = Vec2::new(
			rng.gen_range(-10.0..10.0),
			rng.gen_range(-5.0..5.0),
		);

		commands.spawn((
			Sprite {
				image: asset_server.load(particle_sprite),
				custom_size: Some(Vec2::splat(size)),
				..default()
			},
			Transform::from_xyz(
				position.x + offset.x,
				position.y + offset.y,
				0.9
			),
			Particle {
				lifetime: Timer::from_seconds(0.15, TimerMode::Once),
				velocity: Vec2::new(
					rng.gen_range(-30.0..30.0),
					rng.gen_range(50.0..150.0),
				),
			},
		));
	}
}

pub fn move_projectiles_straight(
	mut query: Query<(&mut Transform, &Projectile), (Without<SineMotion>, Without<AngledShot>, Without<HomingProjectile>)>,
	time: Res<Time>,
) {
	for (mut transform, projectile) in query.iter_mut() {
		transform.translation.y += projectile.speed * time.delta_secs();
	}
}

pub fn move_angled_projectiles(
	mut query: Query<(&mut Transform, &AngledShot)>,
	time: Res<Time>,
) {
	for (mut transform, angled) in query.iter_mut() {
		transform.translation += angled.velocity.extend(0.0) * time.delta_secs();
	}
}

pub fn move_projectiles_sine(
	mut query: Query<(&mut Transform, &Projectile, &mut SineMotion)>,
	time: Res<Time>,
) {
	for (mut transform, projectile, mut sine) in query.iter_mut() {
		transform.translation.y += projectile.speed * time.delta_secs();

		sine.time_offset += time.delta_secs();
		let x_offset = sine.amplitude * (sine.frequency * sine.time_offset).sin();
		transform.translation.x = sine.base_x + x_offset;
	}
}

fn spawn_spread_projectiles(
	commands: &mut Commands,
	asset_server: &AssetServer,
	spawn_pos: Vec3,
	weapon: &Weapon,
	config: &crate::components::WeaponConfig,
	damage: f32,
) {
	let projectile_count = 3 + (weapon.level as usize * 2);
	let max_angle = (30.0 + weapon.level as f32 * 15.0).to_radians();

	for i in 0..projectile_count {
		let t = if projectile_count > 1 {
			i as f32 / (projectile_count - 1) as f32
		} else {
			0.5
		};
		let angle = (t - 0.5) * 2.0 * max_angle;

		let velocity = Vec2::new(angle.sin(), angle.cos()) * config.projectile_speed;

		commands.spawn((
			Sprite {
				image: asset_server.load("sprites/projectiles/spread_shot.png"),
				custom_size: Some(Vec2::new(25.0, 50.0)),
				..default()
			},
			Transform::from_translation(spawn_pos.with_z(PROJECTILE_Z))
				.with_rotation(Quat::from_rotation_z(angle)),
			Projectile {
				weapon_type: weapon.weapon_type,
				level: weapon.level,
				speed: velocity.length(),
				damage,
				lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
			},
			AngledShot {
				velocity,
			},
		));
	}
}

fn spawn_missile_projectiles(
	commands: &mut Commands,
	asset_server: &AssetServer,
	spawn_pos: Vec3,
	weapon: &Weapon,
	config: &crate::components::WeaponConfig,
	damage: f32,
) {
	let missile_count = 1 + (weapon.level / 2) as usize;

	for i in 0..missile_count {
		let offset_x = (i as f32 - (missile_count - 1) as f32 / 2.0) * 15.0;

		commands.spawn((
			Sprite {
				image: asset_server.load("sprites/projectiles/missile.png"),
				custom_size: Some(Vec2::new(30.0, 70.0)),
				..default()
			},
			Transform::from_translation((spawn_pos + Vec3::new(offset_x, 0.0, 0.0)).with_z(PROJECTILE_Z))
				.with_rotation(Quat::from_rotation_z(0.0)), // Points upward
			Projectile {
				weapon_type: weapon.weapon_type,
				level: weapon.level,
				speed: config.projectile_speed,
				damage,
				lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
			},
			HomingProjectile {
				turn_speed: 2.0 + weapon.level as f32 * 0.5,
			},
		));
	}
}

fn spawn_laser_beams(
	commands: &mut Commands,
	asset_server: &AssetServer,
	spawn_pos: Vec3,
	weapon: &Weapon,
	config: &crate::components::WeaponConfig,
	damage: f32,
) {
	let beam_count = 1 + (weapon.level / 3) as usize;

	for i in 0..beam_count {
		let offset_x = (i as f32 - (beam_count - 1) as f32 / 2.0) * 20.0;

		commands.spawn((
			Sprite {
				image: asset_server.load("sprites/projectiles/laser_beam.png"),
				custom_size: Some(Vec2::new(15.0, 100.0)),
				..default()
			},
			Transform::from_translation((spawn_pos + Vec3::new(offset_x, 0.0, 0.0)).with_z(PROJECTILE_Z)),
			Projectile {
				weapon_type: weapon.weapon_type,
				level: weapon.level,
				speed: config.projectile_speed,
				damage,
				lifetime: Timer::from_seconds(0.15, TimerMode::Once),
			},
		));
	}
}

pub fn move_homing_projectiles(
	mut query: Query<(&mut Transform, &Projectile, &HomingProjectile), Without<Enemy>>,
	enemy_query: Query<&Transform, With<Enemy>>,
	time: Res<Time>,
) {
	for (mut projectile_transform, projectile, homing) in query.iter_mut() {
		let mut closest_enemy: Option<Vec2> = None;
		let mut closest_dist = f32::MAX;

		let projectile_pos = projectile_transform.translation.truncate();

		for enemy_transform in enemy_query.iter() {
			let enemy_pos = enemy_transform.translation.truncate();
			let dist = projectile_pos.distance(enemy_pos);

			if dist < closest_dist && dist < 800.0 {
				closest_dist = dist;
				closest_enemy = Some(enemy_pos);
			}
		}

		if let Some(target_pos) = closest_enemy {
			let direction = (target_pos - projectile_pos).normalize();
			let current_direction = Vec2::new(0.0, 1.0);
			let new_direction = current_direction.lerp(direction, homing.turn_speed * time.delta_secs());

			let angle = new_direction.y.atan2(new_direction.x) - PI / 2.0;
			projectile_transform.rotation = Quat::from_rotation_z(angle);

			projectile_transform.translation += (new_direction * projectile.speed * time.delta_secs()).extend(0.0);
		} else {
			projectile_transform.translation.y += projectile.speed * time.delta_secs();
		}
	}
}

pub fn manage_orbital_entities(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	player_query: Query<(&Transform, &Weapon), With<Player>>,
	mut orbital_query: Query<(Entity, &mut Transform, &mut OrbitalEntity), Without<Player>>,
	time: Res<Time>,
) {
	let Ok((player_transform, weapon)) = player_query.get_single() else { return };

	if weapon.weapon_type != WeaponType::OrbitalDefense {
		for (entity, _, _) in orbital_query.iter() {
			commands.entity(entity).despawn();
		}
		return;
	}

	let orb_count = 2 + weapon.level as usize;
	let current_orb_count = orbital_query.iter().count();

	if current_orb_count < orb_count {
		for i in current_orb_count..orb_count {
			let angle = (i as f32 / orb_count as f32) * 2.0 * PI;
			let config = weapon.weapon_type.config();

			commands.spawn((
				Sprite {
					image: asset_server.load("sprites/projectiles/orbital_orb.png"),
					custom_size: Some(Vec2::splat(45.0)),
					..default()
				},
				Transform::from_xyz(0.0, 0.0, 0.6),
				OrbitalEntity {
					angle,
					radius: 60.0,
					rotation_speed: 2.0,
					fire_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
				},
			));
		}
	} else if current_orb_count > orb_count {
		let mut iter = orbital_query.iter();
		for _ in orb_count..current_orb_count {
			if let Some((entity, _, _)) = iter.next() {
				commands.entity(entity).despawn();
			}
		}
	}

	for (_, mut orb_transform, mut orbital) in orbital_query.iter_mut() {
		orbital.angle += orbital.rotation_speed * time.delta_secs();

		let offset_x = orbital.angle.cos() * orbital.radius;
		let offset_y = orbital.angle.sin() * orbital.radius;

		orb_transform.translation = player_transform.translation + Vec3::new(offset_x, offset_y, 0.0);
	}
}

pub fn orbital_auto_fire(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut orbital_query: Query<(&Transform, &mut OrbitalEntity)>,
	enemy_query: Query<&Transform, With<Enemy>>,
	time: Res<Time>,
) {
	for (orb_transform, mut orbital) in orbital_query.iter_mut() {
		orbital.fire_timer.tick(time.delta());

		if orbital.fire_timer.just_finished() {
			// Find closest enemy
			let orb_pos = orb_transform.translation.truncate();
			let mut closest_enemy: Option<(Vec2, f32)> = None;

			for enemy_transform in enemy_query.iter() {
				let enemy_pos = enemy_transform.translation.truncate();
				let distance = orb_pos.distance(enemy_pos);

				if distance < 400.0 {
					if let Some((_, closest_dist)) = closest_enemy {
						if distance < closest_dist {
							closest_enemy = Some((enemy_pos, distance));
						}
					} else {
						closest_enemy = Some((enemy_pos, distance));
					}
				}
			}

			// Fire at closest enemy
			if let Some((enemy_pos, _)) = closest_enemy {
				let direction = (enemy_pos - orb_pos).normalize_or_zero();
				let angle = direction.y.atan2(direction.x) - FRAC_PI_2;

				commands.spawn((
					Sprite {
						image: asset_server.load("sprites/projectiles/orbital_orb.png"),
						custom_size: Some(Vec2::new(15.0, 15.0)),
						..default()
					},
					Transform::from_xyz(orb_pos.x, orb_pos.y, 0.6)
						.with_rotation(Quat::from_rotation_z(angle)),
					Projectile {
						weapon_type: WeaponType::OrbitalDefense,
						level: 1,
						speed: 800.0,
						damage: 15.0,
						lifetime: Timer::from_seconds(2.0, TimerMode::Once),
					},
				));
			}
		}
	}
}

pub fn cleanup_projectiles(
	mut commands: Commands,
	mut query: Query<(Entity, &Transform, &mut Projectile)>,
	time: Res<Time>,
) {
	for (entity, transform, mut projectile) in query.iter_mut() {
		projectile.lifetime.tick(time.delta());

		if projectile.lifetime.finished() || transform.translation.y > HALF_WORLD_HEIGHT + 50.0 {
			commands.entity(entity).despawn();
		}
	}
}
