use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::Rng;
use std::collections::HashSet;
use crate::components::{
	Weapon, WeaponType, Enemy, Health, Player,
	EnemyHitEvent, ChargeMeter, Collider,
	LightningBolt, LightningImpact, LightningAoeEffect, PendingBabyWhip, LightningArc,
	PendingSound, FadingSound,
};

const VIEWPORT_HEIGHT: f32 = 1000.0;
const VIEWPORT_TOP: f32 = VIEWPORT_HEIGHT / 2.0; // 500.0
const CURVE_DISTANCE: f32 = 175.0; // How far the curve section travels
const MAX_CURVE_ANGLE: f32 = 30.0;
const CURVE_MARGIN: f32 = 225.0; // Start curve this far before viewport edge

struct RaycastResult {
	hit_enemy: Option<Entity>,
	hit_position: Vec2,
	ray_end_visual: Vec2,
	curve_point: Vec2,
}

fn perform_hitscan_ray(
	start_pos: Vec2,
	direction: Vec2,
	enemies: &Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
) -> RaycastResult {
	let mut rng = rand::thread_rng();

	// Calculate dynamic straight distance based on player position
	let player_in_top_quarter = start_pos.y > VIEWPORT_TOP / 2.0; // > 250

	let straight_distance = if player_in_top_quarter {
		// Top quarter: let it travel far, curve can go off-screen
		600.0
	} else {
		// Bottom 3/4: curve must be visible on screen
		// Distance to viewport top, minus margin for curve visibility
		let distance_to_top = VIEWPORT_TOP - start_pos.y;
		(distance_to_top - CURVE_MARGIN).max(100.0) // At least 100gu straight
	};

	// Generate random curve angle ±30°
	let curve_angle_rad = rng.gen_range(-MAX_CURVE_ANGLE.to_radians()..MAX_CURVE_ANGLE.to_radians());

	// Straight section endpoint
	let straight_end = start_pos + direction * straight_distance;

	// Rotate direction by curve angle
	let cos_a = curve_angle_rad.cos();
	let sin_a = curve_angle_rad.sin();
	let curve_dir = Vec2::new(
		direction.x * cos_a - direction.y * sin_a,
		direction.x * sin_a + direction.y * cos_a,
	);

	// Curved section endpoint
	let ray_end_visual = straight_end + curve_dir * CURVE_DISTANCE;

	// Collision detection along ray path
	let mut closest_hit: Option<(Entity, Vec2, f32)> = None;

	// Sample straight section (40 points)
	for i in 0..=40 {
		let t = i as f32 / 40.0;
		let sample_point = start_pos.lerp(straight_end, t);

		for (entity, transform, _, collider) in enemies.iter() {
			let enemy_pos = transform.translation.truncate();
			let dist = sample_point.distance(enemy_pos);

			if dist < collider.radius {
				let ray_dist = start_pos.distance(sample_point);
				if closest_hit.is_none() || ray_dist < closest_hit.as_ref().unwrap().2 {
					closest_hit = Some((entity, enemy_pos, ray_dist));
				}
			}
		}
	}

	// Sample curved section (20 points), only if no hit yet
	if closest_hit.is_none() {
		for i in 0..=20 {
			let t = i as f32 / 20.0;
			let sample_point = straight_end.lerp(ray_end_visual, t);

			for (entity, transform, _, collider) in enemies.iter() {
				let enemy_pos = transform.translation.truncate();
				let dist = sample_point.distance(enemy_pos);

				if dist < collider.radius {
					let ray_dist = start_pos.distance(sample_point);
					if closest_hit.is_none() || ray_dist < closest_hit.as_ref().unwrap().2 {
						closest_hit = Some((entity, enemy_pos, ray_dist));
					}
				}
			}
		}
	}

	RaycastResult {
		hit_enemy: closest_hit.map(|(e, _, _)| e),
		hit_position: closest_hit.map(|(_, pos, _)| pos).unwrap_or(ray_end_visual),
		ray_end_visual,
		curve_point: straight_end,
	}
}

fn calculate_whip_directions(level: u8, is_charged: bool) -> Vec<f32> {
	// Returns angle offsets in radians
	match (level, is_charged) {
		(1..=3, _) => vec![0.0],
		(4..=6, _) => vec![-0.087, 0.087], // ±5°
		(7..=8, _) => vec![-0.174, 0.0, 0.174], // -10°, 0°, +10°
		(9, false) => vec![-0.174, -0.087, 0.087, 0.174],
		(9, true) | (10, false) => vec![-0.174, -0.087, 0.0, 0.087, 0.174],
		(10, true) => vec![-0.262, -0.174, -0.087, 0.0, 0.087, 0.174, 0.262], // ±15°
		_ => vec![0.0],
	}
}

fn rotate_direction(dir: Vec2, angle_radians: f32) -> Vec2 {
	let cos_a = angle_radians.cos();
	let sin_a = angle_radians.sin();
	Vec2::new(
		dir.x * cos_a - dir.y * sin_a,
		dir.x * sin_a + dir.y * cos_a,
	)
}

fn find_chain_target(
	from_pos: Vec2,
	range: f32,
	prioritize_low_health: bool,
	enemies: &Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
	already_hit: &HashSet<Entity>,
) -> Option<(Entity, Vec2)> {
	let mut best_target: Option<(Entity, Vec2, f32)> = None;

	for (entity, transform, health, _) in enemies.iter() {
		if already_hit.contains(&entity) {
			continue;
		}

		let enemy_pos = transform.translation.truncate();
		let distance = from_pos.distance(enemy_pos);

		if distance <= range {
			let priority_score = if prioritize_low_health {
				distance / health.current.max(1.0)
			} else {
				distance
			};

			if best_target.is_none() || priority_score < best_target.as_ref().unwrap().2 {
				best_target = Some((entity, enemy_pos, priority_score));
			}
		}
	}

	best_target.map(|(e, pos, _)| (e, pos))
}

fn spawn_impact_buzz(commands: &mut Commands, position: Vec2) {
	commands.spawn(LightningImpact {
		position,
		lifetime: Timer::from_seconds(0.08, TimerMode::Once),
		branch_count: 8,
		radius: 40.0,
		intensity: 1.0,
	});
}

fn execute_aoe_explosion(
	center: Vec2,
	radius: f32,
	base_damage: f32,
	enemies: &Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
	hit_events: &mut EventWriter<EnemyHitEvent>,
	commands: &mut Commands,
) {
	for (entity, transform, _, _) in enemies.iter() {
		let enemy_pos = transform.translation.truncate();
		let distance = center.distance(enemy_pos);

		if distance <= radius {
			// Spawn visual arc
			commands.spawn(LightningArc {
				start: center,
				end: enemy_pos,
				lifetime: Timer::from_seconds(0.12, TimerMode::Once),
				thickness: 1.5,
				intensity: 0.7,
			});

			// Apply damage (50% of base)
			hit_events.send(EnemyHitEvent {
				enemy: entity,
				damage: base_damage * 0.5,
				hit_sound: Some("sounds/lightning/lightning_wave_light.ogg"),
			});
		}
	}

	// Spawn AoE shimmer effect
	commands.spawn(LightningAoeEffect {
		position: center,
		radius,
		lifetime: Timer::from_seconds(0.25, TimerMode::Once),
		intensity: 0.8,
	});
	// Note: boom+crackle sounds are pooled and played once from fire_lightning_weapon
}

fn try_spawn_delayed_baby_whip(
	commands: &mut Commands,
	parent_pos: Vec2,
	target_pos: Vec2,
	parent_damage: f32,
	parent_level: u8,
	chain_range: f32,
	aoe_radius: f32,
	baby_spawn_chance: f32,
	recursion_depth: u8,
) {
	if recursion_depth >= 3 {
		return; // Max recursion depth
	}

	let mut rng = rand::thread_rng();
	if rng.gen::<f32>() > baby_spawn_chance {
		return; // Didn't spawn
	}

	// Calculate perpendicular spawn direction
	let chain_dir = (target_pos - parent_pos).normalize();
	let perpendicular = Vec2::new(-chain_dir.y, chain_dir.x);
	let side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };

	commands.spawn(PendingBabyWhip {
		delay_timer: Timer::from_seconds(rng.gen_range(0.1..0.2), TimerMode::Once),
		spawn_from: parent_pos.lerp(target_pos, 0.5),
		direction: perpendicular * side,
		parent_damage: parent_damage * 0.50,
		parent_level,
		parent_chain_range: chain_range * 0.8,
		parent_aoe_radius: aoe_radius * 0.7,
		recursion_depth: recursion_depth + 1,
		baby_spawn_chance: baby_spawn_chance * 0.6,
	});
}

fn execute_chain_sequence(
	start_pos: Vec2,
	max_chains: u8,
	chain_range: f32,
	damage: f32,
	damage_falloff: f32,
	prioritize_low_health: bool,
	enemies: &Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
	hit_events: &mut EventWriter<EnemyHitEvent>,
	commands: &mut Commands,
	already_hit: &mut HashSet<Entity>,
	level: u8,
	aoe_radius: f32,
	baby_spawn_chance: f32,
	recursion_depth: u8,
	audio: &Audio,
	asset_server: &AssetServer,
) {
	let mut current_pos = start_pos;

	for chain_index in 0..max_chains {
		// Find next target
		let next_target = find_chain_target(
			current_pos,
			chain_range,
			prioritize_low_health,
			enemies,
			already_hit,
		);

		if let Some((target_entity, target_pos)) = next_target {
			// Spawn visual arc
			commands.spawn(LightningArc {
				start: current_pos,
				end: target_pos,
				lifetime: Timer::from_seconds(0.15, TimerMode::Once),
				thickness: 2.0 - (chain_index as f32 * 0.2),
				intensity: 0.9 - (chain_index as f32 * 0.1),
			});

			// Chain arc sound (lighter than main impact)
			audio.play(asset_server.load("sounds/lightning/lightning_wave_light.ogg"))
				.with_volume(0.15);

			// Apply damage
			let chain_damage = damage * (1.0 - damage_falloff * chain_index as f32);
			hit_events.send(EnemyHitEvent {
				enemy: target_entity,
				damage: chain_damage,
				hit_sound: Some("sounds/lightning/lightning_wave_light.ogg"),
			});

			already_hit.insert(target_entity);

			// Spawn baby whip with delay
			try_spawn_delayed_baby_whip(
				commands,
				current_pos,
				target_pos,
				damage,
				level,
				chain_range,
				aoe_radius,
				baby_spawn_chance,
				recursion_depth,
			);

			current_pos = target_pos;
		} else {
			break; // No more targets
		}
	}

	// AoE explosion at final position
	execute_aoe_explosion(current_pos, aoe_radius, damage, &enemies, hit_events, commands);
}

pub fn fire_lightning_weapon(
	commands: &mut Commands,
	asset_server: &AssetServer,
	audio: &Audio,
	spawn_pos: Vec3,
	weapon: &Weapon,
	damage: f32,
	charge_meter: &mut ChargeMeter,
	enemies: &Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
	hit_events: &mut EventWriter<EnemyHitEvent>,
) {
	let level = weapon.level;

	// Check charge state (levels 8-10 only)
	let is_charged = level >= 8 && charge_meter.current >= 1.0;

	// Consume charge
	if level >= 8 {
		if is_charged {
			charge_meter.current = 0.0;
			charge_meter.charge_consumed_this_frame = true;
		} else if charge_meter.current >= 0.3 {
			charge_meter.current -= 0.3;
			charge_meter.charge_consumed_this_frame = true;
		} else {
			return; // Not enough charge
		}
	}

	// Level-based config (matching old system)
	let (max_chains, chain_range, aoe_radius, damage_mult, damage_falloff) = match (level, is_charged) {
		(1, _) => (0, 0.0, 40.0, 1.0, 0.0),
		(2, _) => (1, 150.0, 50.0, 1.0, 0.2),
		(3, _) => (2, 180.0, 60.0, 1.0, 0.2),
		(4, _) => (2, 200.0, 70.0, 1.0, 0.15),
		(5, _) => (3, 250.0, 80.0, 1.0, 0.10),
		(6, _) => (4, 280.0, 90.0, 1.0, 0.08),
		(7, _) => (4, 300.0, 100.0, 1.0, 0.05),
		(8, false) => (4, 300.0, 100.0, 1.0, 0.05),
		(8, true) => (6, 350.0, 130.0, 1.375, 0.02),
		(9, false) => (5, 350.0, 120.0, 1.0, 0.03),
		(9, true) => (8, 450.0, 160.0, 1.556, 0.0),
		(10, false) => (6, 400.0, 140.0, 1.0, 0.01),
		(10, true) => (12, 550.0, 220.0, 1.8, 0.0),
		_ => (6, 400.0, 140.0, 1.0, 0.01),
	};

	// Baby whip spawn chance
	let baby_spawn_chance = match (level, is_charged) {
		(0..=4, _) => 0.0,
		(5, _) => 0.15,
		(6, _) => 0.25,
		(7, _) => 0.30,
		(8, false) => 0.30,
		(8, true) => 0.45,
		(9, false) => 0.40,
		(9, true) => 0.60,
		(10, false) => 0.50,
		(10, true) => 0.70,
		_ => 0.0,
	};

	let actual_damage = damage * damage_mult;

	// Visual feedback for charged shots
	let bolt_color = if is_charged {
		Color::srgb(0.8, 1.0, 1.0) // Brighter cyan
	} else {
		Color::srgb(0.7, 0.9, 1.0)
	};

	// Calculate whip directions
	let whip_angles = calculate_whip_directions(level, is_charged);
	let num_whips = whip_angles.len();
	let base_direction = Vec2::Y; // Fire upward

	// Fire each whip
	for angle_offset in &whip_angles {
		let whip_direction = rotate_direction(base_direction, *angle_offset);

		// Perform hitscan
		let ray_result = perform_hitscan_ray(spawn_pos.truncate(), whip_direction, enemies);

		// Spawn main bolt visual
		commands.spawn(LightningBolt {
			start: spawn_pos.truncate(),
			end: ray_result.ray_end_visual,
			curve_point: ray_result.curve_point,
			lifetime: Timer::from_seconds(0.15, TimerMode::Once),
			thickness_start: 2.0,
			thickness_end: 4.0,
			intensity: if is_charged { 1.0 } else { 0.9 },
			is_baby: false,
			recursion_depth: 0,
		});

		// If hit enemy, spawn impact and execute chain
		if let Some(hit_entity) = ray_result.hit_enemy {
			spawn_impact_buzz(commands, ray_result.hit_position);

			hit_events.send(EnemyHitEvent {
				enemy: hit_entity,
				damage: actual_damage,
				hit_sound: Some("sounds/lightning/lightning_wave_light.ogg"),
			});

			let mut already_hit = HashSet::new();
			already_hit.insert(hit_entity);

			// Convert enemy query types for chain execution
			let enemies_for_chain = enemies.iter()
				.map(|(e, t, h, _)| (e, t, h))
				.collect::<Vec<_>>();

			// Create temporary query for chain targeting
			let enemies_health_only: Vec<_> = enemies.iter()
				.map(|(e, t, h, _)| (e, t, h))
				.collect();

			// Execute chain sequence
			execute_chain_sequence(
				ray_result.hit_position,
				max_chains,
				chain_range,
				actual_damage,
				damage_falloff,
				level >= 5,
				&enemies,
				hit_events,
				commands,
				&mut already_hit,
				level,
				aoe_radius,
				baby_spawn_chance,
				0, // recursion_depth
				audio,
				asset_server,
			);
		} else {
			// No hit, AoE at end of bolt
			execute_aoe_explosion(
				ray_result.ray_end_visual,
				aoe_radius,
				actual_damage,
				enemies,
				hit_events,
				commands,
			);
		}
	}

	// Audio - main fire (0.45 volume)
	let fire_sound = if is_charged {
		"sounds/lightning/lightning_charged.ogg"
	} else {
		"sounds/lightning/lightning_standard.ogg"
	};
	audio.play(asset_server.load(fire_sound)).with_volume(0.25);

	// Pooled boom+crackle sounds (delayed, with fade after 300ms)
	// Boom: 50ms delay
	commands.spawn(PendingSound {
		delay: Timer::from_seconds(0.05, TimerMode::Once),
		sound_path: "sounds/lightning/deep_lightning_boom.ogg",
		volume: 0.4,
		fade_after: Some(0.3),
		fade_duration: 0.2,
	});
	// Crackle: 100ms delay
	commands.spawn(PendingSound {
		delay: Timer::from_seconds(0.1, TimerMode::Once),
		sound_path: "sounds/lightning/fireworks_crackle.ogg",
		volume: 0.25,
		fade_after: Some(0.3),
		fade_duration: 0.15,
	});

	info!("Lightning hitscan fired: level={}, charged={}, whips={}, max_chains={}",
		level, is_charged, num_whips, max_chains);
}

pub fn update_charge_meter(
	mut charge_meter: ResMut<ChargeMeter>,
	keyboard: Res<ButtonInput<KeyCode>>,
	weapon_query: Query<&Weapon, With<Player>>,
	time: Res<Time>,
) {
	let Ok(weapon) = weapon_query.get_single() else { return };

	charge_meter.charge_consumed_this_frame = false;

	// Only active for LightningChain at level 8+
	if weapon.weapon_type != WeaponType::LightningChain || weapon.level < 8 {
		charge_meter.current = charge_meter.max;
		charge_meter.is_charging = false;
		return;
	}

	let fire_held = keyboard.pressed(KeyCode::Space);

	if fire_held && charge_meter.current > 0.0 {
		charge_meter.is_charging = true;
		charge_meter.current = (charge_meter.current - 0.2 * time.delta_secs()).max(0.0);
	} else {
		charge_meter.is_charging = false;
		charge_meter.current = (charge_meter.current + charge_meter.recharge_rate * time.delta_secs())
			.min(charge_meter.max);
	}
}

fn generate_curved_lightning_path(
	start: Vec2,
	end: Vec2,
	curve_point: Vec2,
	segment_count: usize,
	displacement: f32,
) -> Vec<Vec2> {
	let mut rng = rand::thread_rng();
	let mut segments = vec![start];

	let straight_ratio = 0.8;
	let straight_segments = (segment_count as f32 * straight_ratio) as usize;
	let curve_segments = segment_count - straight_segments;

	// Straight section
	for i in 1..straight_segments {
		let t = i as f32 / straight_segments as f32;
		let base_point = start.lerp(curve_point, t);

		let direction = (curve_point - start).normalize();
		let perpendicular = Vec2::new(-direction.y, direction.x);
		let offset = perpendicular * rng.gen_range(-displacement..displacement);

		segments.push(base_point + offset);
	}

	segments.push(curve_point);

	// Curved section
	for i in 1..curve_segments {
		let t = i as f32 / curve_segments as f32;
		let base_point = curve_point.lerp(end, t);

		let direction = (end - curve_point).normalize();
		let perpendicular = Vec2::new(-direction.y, direction.x);
		let offset = perpendicular * rng.gen_range(-displacement * 1.5..displacement * 1.5);

		segments.push(base_point + offset);
	}

	segments.push(end);
	segments
}

pub fn render_lightning_bolts(
	mut gizmos: Gizmos,
	bolts: Query<&LightningBolt>,
) {
	for bolt in bolts.iter() {
		let alpha = bolt.lifetime.fraction_remaining();

		// Generate jagged path every frame
		let segments = generate_curved_lightning_path(
			bolt.start,
			bolt.end,
			bolt.curve_point,
			25,
			if bolt.is_baby { 10.0 } else { 15.0 },
		);

		// Multi-layer lightning rendering for visual impact
		for (i, window) in segments.windows(2).enumerate() {
			let t = i as f32 / (segments.len() - 1) as f32;
			let thickness = bolt.thickness_start + (bolt.thickness_end - bolt.thickness_start) * t;

			// Outer glow layer (blue, large spread)
			let outer_glow = Color::srgba(0.3, 0.6, 1.0, alpha * 0.2);
			for offset in (-3..=3).map(|o| o as f32 * 1.5) {
				gizmos.line_2d(
					window[0] + Vec2::new(offset, 0.0),
					window[1] + Vec2::new(offset, 0.0),
					outer_glow,
				);
			}

			// Middle glow (brighter cyan)
			let middle_glow = Color::srgba(0.5, 0.8, 1.0, alpha * 0.5);
			for offset in [-thickness * 0.8, 0.0, thickness * 0.8] {
				gizmos.line_2d(
					window[0] + Vec2::new(offset, 0.0),
					window[1] + Vec2::new(offset, 0.0),
					middle_glow,
				);
			}

			// Main core (bright cyan)
			let core_color = Color::srgba(0.8, 0.95, 1.0, alpha * bolt.intensity);
			gizmos.line_2d(window[0], window[1], core_color);

			// Inner bright white core for extra punch
			let inner_core = Color::srgba(0.95, 1.0, 1.0, alpha * bolt.intensity * 0.7);
			gizmos.line_2d(
				window[0] + Vec2::new(0.5, 0.0),
				window[1] + Vec2::new(0.5, 0.0),
				inner_core,
			);
		}
	}
}

pub fn render_lightning_impacts(
	mut gizmos: Gizmos,
	impacts: Query<&LightningImpact>,
) {
	let mut rng = rand::thread_rng();

	for impact in impacts.iter() {
		let alpha = impact.lifetime.fraction_remaining();

		// Draw jagged arc branches radiating outward from impact point
		for i in 0..impact.branch_count {
			let base_angle = (i as f32 / impact.branch_count as f32) * std::f32::consts::TAU;
			// Add slight random offset to angle each frame for crackling effect
			let angle = base_angle + rng.gen_range(-0.15..0.15);
			let direction = Vec2::new(angle.cos(), angle.sin());

			// Branch endpoint
			let branch_end = impact.position + direction * impact.radius;

			// Generate jagged path with 5-6 segments
			let segment_count = 5;
			let mut points = vec![impact.position];

			for j in 1..segment_count {
				let t = j as f32 / segment_count as f32;
				let base_point = impact.position.lerp(branch_end, t);

				// Perpendicular displacement for jaggedness
				let perpendicular = Vec2::new(-direction.y, direction.x);
				let displacement = rng.gen_range(-12.0..12.0) * (1.0 - t * 0.5); // Less jag near end
				points.push(base_point + perpendicular * displacement);
			}
			points.push(branch_end);

			// Draw with multi-layer glow
			for w in points.windows(2) {
				let p1 = w[0];
				let p2 = w[1];

				// Outer glow (wider, dimmer)
				let outer_color = Color::srgba(0.4, 0.7, 1.0, alpha * impact.intensity * 0.3);
				gizmos.line_2d(p1 + Vec2::new(-2.0, 0.0), p2 + Vec2::new(-2.0, 0.0), outer_color);
				gizmos.line_2d(p1 + Vec2::new(2.0, 0.0), p2 + Vec2::new(2.0, 0.0), outer_color);
				gizmos.line_2d(p1 + Vec2::new(0.0, -2.0), p2 + Vec2::new(0.0, -2.0), outer_color);
				gizmos.line_2d(p1 + Vec2::new(0.0, 2.0), p2 + Vec2::new(0.0, 2.0), outer_color);

				// Middle glow
				let mid_color = Color::srgba(0.6, 0.85, 1.0, alpha * impact.intensity * 0.6);
				gizmos.line_2d(p1, p2, mid_color);

				// Bright core
				let core_color = Color::srgba(0.9, 0.98, 1.0, alpha * impact.intensity * 0.9);
				gizmos.line_2d(
					p1 + Vec2::new(0.3, 0.3),
					p2 + Vec2::new(0.3, 0.3),
					core_color
				);
			}
		}

		// Small bright center flash
		let center_color = Color::srgba(1.0, 1.0, 1.0, alpha * impact.intensity);
		let center_radius = 6.0;
		for i in 0..8 {
			let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
			let p1 = impact.position + Vec2::new(angle.cos(), angle.sin()) * 2.0;
			let p2 = impact.position + Vec2::new(angle.cos(), angle.sin()) * center_radius;
			gizmos.line_2d(p1, p2, center_color);
		}
	}
}

pub fn render_lightning_aoe(
	mut gizmos: Gizmos,
	aoe_effects: Query<&LightningAoeEffect>,
) {
	let mut rng = rand::thread_rng();

	for aoe in aoe_effects.iter() {
		let alpha = aoe.lifetime.fraction_remaining();
		let progress = 1.0 - alpha; // 0 at start, 1 at end

		// Expanding shockwave radius
		let shockwave_radius = aoe.radius * (0.3 + progress * 0.7);

		// === Electric Tendrils radiating outward ===
		let tendril_count = 12;
		for i in 0..tendril_count {
			let base_angle = (i as f32 / tendril_count as f32) * std::f32::consts::TAU;
			// Rotate slightly each frame for swirling effect
			let angle = base_angle + rng.gen_range(-0.2..0.2);
			let direction = Vec2::new(angle.cos(), angle.sin());

			// Tendril length varies - some reach full radius, some shorter
			let tendril_length = shockwave_radius * rng.gen_range(0.6..1.0);
			let tendril_end = aoe.position + direction * tendril_length;

			// Generate jagged tendril path
			let segment_count = 4;
			let mut points = vec![aoe.position];

			for j in 1..segment_count {
				let t = j as f32 / segment_count as f32;
				let base_point = aoe.position.lerp(tendril_end, t);
				let perpendicular = Vec2::new(-direction.y, direction.x);
				let displacement = rng.gen_range(-8.0..8.0);
				points.push(base_point + perpendicular * displacement);
			}
			points.push(tendril_end);

			// Draw tendril with glow
			for w in points.windows(2) {
				let p1 = w[0];
				let p2 = w[1];

				// Outer glow
				let outer = Color::srgba(0.3, 0.5, 0.9, alpha * 0.25);
				gizmos.line_2d(p1 + Vec2::new(-1.5, 0.0), p2 + Vec2::new(-1.5, 0.0), outer);
				gizmos.line_2d(p1 + Vec2::new(1.5, 0.0), p2 + Vec2::new(1.5, 0.0), outer);

				// Core
				let core = Color::srgba(0.7, 0.85, 1.0, alpha * 0.7);
				gizmos.line_2d(p1, p2, core);
			}

			// Spark at tendril tip
			if rng.gen_bool(0.4) {
				let spark_color = Color::srgba(1.0, 1.0, 1.0, alpha * 0.9);
				let spark_dir = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero();
				gizmos.line_2d(tendril_end, tendril_end + spark_dir * 8.0, spark_color);
			}
		}

		// === Expanding shockwave ring ===
		let ring_color = Color::srgba(0.5, 0.7, 1.0, alpha * 0.4);
		let segments = 24;
		for i in 0..segments {
			let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
			let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

			// Add wobble to the ring
			let wobble1 = 1.0 + rng.gen_range(-0.05..0.05);
			let wobble2 = 1.0 + rng.gen_range(-0.05..0.05);

			let p1 = aoe.position + Vec2::new(angle1.cos(), angle1.sin()) * shockwave_radius * wobble1;
			let p2 = aoe.position + Vec2::new(angle2.cos(), angle2.sin()) * shockwave_radius * wobble2;

			gizmos.line_2d(p1, p2, ring_color);
		}

		// === Central bright flash (fades quickly) ===
		let flash_alpha = (alpha * 3.0).min(1.0); // Bright at start, fades fast
		let flash_color = Color::srgba(0.9, 0.95, 1.0, flash_alpha * 0.8);
		let flash_radius = 15.0 * (1.0 - progress * 0.5);

		for i in 0..6 {
			let angle = (i as f32 / 6.0) * std::f32::consts::TAU + rng.gen_range(-0.1..0.1);
			let p1 = aoe.position;
			let p2 = aoe.position + Vec2::new(angle.cos(), angle.sin()) * flash_radius;
			gizmos.line_2d(p1, p2, flash_color);
		}

		// === Random sparks popping off ===
		let spark_count = (8.0 * alpha) as usize; // More sparks at start
		for _ in 0..spark_count {
			let angle = rng.gen_range(0.0..std::f32::consts::TAU);
			let dist = rng.gen_range(0.2..0.9) * shockwave_radius;
			let spark_pos = aoe.position + Vec2::new(angle.cos(), angle.sin()) * dist;

			let spark_color = Color::srgba(0.8, 0.9, 1.0, alpha * 0.6);
			let spark_dir = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero();
			gizmos.line_2d(spark_pos, spark_pos + spark_dir * 5.0, spark_color);
		}
	}
}

pub fn spawn_pending_baby_whips(
	mut commands: Commands,
	time: Res<Time>,
	mut pending: Query<(Entity, &mut PendingBabyWhip)>,
	enemies: Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
	mut hit_events: EventWriter<EnemyHitEvent>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
) {
	for (entity, mut pending_whip) in pending.iter_mut() {
		pending_whip.delay_timer.tick(time.delta());

		if pending_whip.delay_timer.finished() {
			// Execute baby whip hitscan
			let ray_result = perform_hitscan_ray(
				pending_whip.spawn_from,
				pending_whip.direction,
				&enemies,
			);

			// Spawn bolt visual
			commands.spawn(LightningBolt {
				start: pending_whip.spawn_from,
				end: ray_result.ray_end_visual,
				curve_point: ray_result.curve_point,
				lifetime: Timer::from_seconds(0.12, TimerMode::Once),
				thickness_start: 1.5,
				thickness_end: 2.5,
				intensity: 0.85,
				is_baby: true,
				recursion_depth: pending_whip.recursion_depth,
			});

			// Baby whip fire sound (sparkly glitter effect)
			audio.play(asset_server.load("sounds/lightning/fireworks_glitter.ogg"))
				.with_volume(0.2);

			// If hit enemy, execute chain
			if let Some(hit_entity) = ray_result.hit_enemy {
				spawn_impact_buzz(&mut commands, ray_result.hit_position);

				// Baby whip impact (quieter than main impact)
				audio.play(asset_server.load("sounds/lightning/lightning_wave_light.ogg"))
					.with_volume(0.15);

				hit_events.send(EnemyHitEvent {
					enemy: hit_entity,
					damage: pending_whip.parent_damage,
					hit_sound: Some("sounds/lightning/lightning_wave_light.ogg"),
				});

				let mut already_hit = HashSet::new();
				already_hit.insert(hit_entity);

				let max_chains = (2 + pending_whip.recursion_depth).min(4) as u8;

				execute_chain_sequence(
					ray_result.hit_position,
					max_chains,
					pending_whip.parent_chain_range,
					pending_whip.parent_damage,
					0.15,
					pending_whip.parent_level >= 5,
					&enemies,
					&mut hit_events,
					&mut commands,
					&mut already_hit,
					pending_whip.parent_level,
					pending_whip.parent_aoe_radius,
					pending_whip.baby_spawn_chance,
					pending_whip.recursion_depth,
					&audio,
					&asset_server,
				);
			}

			commands.entity(entity).despawn();
		}
	}
}

pub fn cleanup_lightning_visuals(
	mut commands: Commands,
	time: Res<Time>,
	mut bolts: Query<(Entity, &mut LightningBolt)>,
	mut impacts: Query<(Entity, &mut LightningImpact)>,
	mut aoe_effects: Query<(Entity, &mut LightningAoeEffect)>,
	mut arcs: Query<(Entity, &mut LightningArc)>,
) {
	for (entity, mut bolt) in bolts.iter_mut() {
		bolt.lifetime.tick(time.delta());
		if bolt.lifetime.finished() {
			commands.entity(entity).despawn();
		}
	}

	for (entity, mut impact) in impacts.iter_mut() {
		impact.lifetime.tick(time.delta());
		if impact.lifetime.finished() {
			commands.entity(entity).despawn();
		}
	}

	for (entity, mut aoe) in aoe_effects.iter_mut() {
		aoe.lifetime.tick(time.delta());
		if aoe.lifetime.finished() {
			commands.entity(entity).despawn();
		}
	}

	for (entity, mut arc) in arcs.iter_mut() {
		arc.lifetime.tick(time.delta());
		if arc.lifetime.finished() {
			commands.entity(entity).despawn();
		}
	}
}

pub fn render_lightning_arcs(
	mut gizmos: Gizmos,
	arcs: Query<&LightningArc>,
) {
	use rand::Rng;
	let mut rng = rand::thread_rng();

	for arc in arcs.iter() {
		let elapsed = arc.lifetime.elapsed_secs();
		let duration = arc.lifetime.duration().as_secs_f32();
		let progress = (elapsed / duration).clamp(0.0, 1.0);
		let fade = 1.0 - progress;

		// Create curved path between start and end (using bezier-like curve)
		let mid_point = arc.start.lerp(arc.end, 0.5);
		let direction = (arc.end - arc.start).normalize();
		let perpendicular = Vec2::new(-direction.y, direction.x);

		// Curve control point (varies with randomness for electric feel)
		let curve_offset = perpendicular * rng.gen_range(-20.0..20.0);
		let control_point = mid_point + curve_offset;

		// Generate curved segments (quadratic bezier)
		let segments = 12;
		let mut path_points = Vec::new();
		for i in 0..=segments {
			let t = i as f32 / segments as f32;
			// Quadratic bezier: B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
			let p = (1.0 - t).powi(2) * arc.start
				+ 2.0 * (1.0 - t) * t * control_point
				+ t.powi(2) * arc.end;

			// Add slight jaggedness to the arc
			let jitter = rng.gen_range(-2.0..2.0);
			let jitter_dir = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize();
			path_points.push(p + jitter_dir * jitter);
		}

		// Draw with multiple glow layers
		for (i, window) in path_points.windows(2).enumerate() {
			let t = i as f32 / path_points.len() as f32;

			// Outer glow (faint blue)
			let outer_glow = Color::srgba(0.3, 0.5, 1.0, arc.intensity * fade * 0.15);
			for offset in (-2..=2).map(|o| o as f32) {
				gizmos.line_2d(
					window[0] + Vec2::new(offset, 0.0),
					window[1] + Vec2::new(offset, 0.0),
					outer_glow,
				);
			}

			// Middle glow (cyan)
			let mid_glow = Color::srgba(0.4, 0.7, 1.0, arc.intensity * fade * 0.3);
			gizmos.line_2d(window[0], window[1], mid_glow);

			// Bright core (white-cyan)
			let core = Color::srgba(
				0.7 * arc.intensity * fade,
				0.9 * arc.intensity * fade,
				1.0 * arc.intensity * fade,
				arc.intensity * fade * 0.8
			);
			gizmos.line_2d(window[0], window[1], core);
		}
	}
}

pub fn process_pending_sounds(
	mut commands: Commands,
	time: Res<Time>,
	mut pending: Query<(Entity, &mut PendingSound)>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
	for (entity, mut sound) in pending.iter_mut() {
		sound.delay.tick(time.delta());

		if sound.delay.finished() {
			// Play the sound and get the instance handle
			let handle = audio.play(asset_server.load(sound.sound_path))
				.with_volume(sound.volume as f64)
				.handle();

			// If fade is configured, spawn a FadingSound to handle it
			if let Some(fade_after) = sound.fade_after {
				commands.spawn(FadingSound {
					fade_timer: Timer::from_seconds(fade_after, TimerMode::Once),
					instance: handle,
				});
			}

			commands.entity(entity).despawn();
		}
	}
}

pub fn process_fading_sounds(
	mut commands: Commands,
	time: Res<Time>,
	mut fading: Query<(Entity, &mut FadingSound)>,
	mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
	for (entity, mut sound) in fading.iter_mut() {
		sound.fade_timer.tick(time.delta());

		if sound.fade_timer.finished() {
			// Start fade out
			if let Some(instance) = audio_instances.get_mut(&sound.instance) {
				instance.stop(AudioTween::linear(std::time::Duration::from_millis(150)));
			}
			commands.entity(entity).despawn();
		}
	}
}
