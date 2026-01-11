use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::Rng;
use std::collections::HashSet;
use crate::components::{
	Weapon, WeaponType, Enemy, Health, Player,
	EnemyHitEvent, ChargeMeter, Collider,
	LightningBolt, LightningImpact, LightningAoeEffect, PendingBabyWhip, LightningArc,
	LightningGlitter, PendingSound, FadingSound,
};

const VIEWPORT_HEIGHT: f32 = 1000.0;
const VIEWPORT_TOP: f32 = VIEWPORT_HEIGHT / 2.0; // 500.0

// Whip curve tuning
const BOW_OFFSET_MIN: f32 = 25.0;  // Min perpendicular bow on "straight" section
const BOW_OFFSET_MAX: f32 = 45.0;  // Max perpendicular bow
const DRIFT_ANGLE_MIN: f32 = 8.0;  // Early drift phase (degrees)
const DRIFT_ANGLE_MAX: f32 = 12.0;
const DRIFT_DISTANCE: f32 = 60.0;  // Drift before commit
const COMMIT_ANGLE_MIN: f32 = 28.0; // Firm commit phase (degrees)
const COMMIT_ANGLE_MAX: f32 = 45.0;
const BOOM_ZONE_BELOW_TOP: f32 = 75.0; // Target Y: this many units below viewport top
const COMMIT_DISTANCE_MIN: f32 = 60.0;  // Minimum commit travel (visual consistency)
const COMMIT_DISTANCE_MAX: f32 = 180.0; // Maximum commit travel

// Glitter effect tuning (burning micro-sparks)
const GLITTER_COUNT_MIN: u32 = 270;
const GLITTER_COUNT_MAX: u32 = 430;
const GLITTER_SPEED_MIN: f32 = 8.0;    // Drift outward
const GLITTER_SPEED_MAX: f32 = 35.0;
const GLITTER_LIFETIME_MIN: f32 = 0.25;
const GLITTER_LIFETIME_MAX: f32 = 0.55;
const GLITTER_SIZE_MIN: f32 = 0.4;     // Tiny spark arm length
const GLITTER_SIZE_MAX: f32 = 1.2;

struct RaycastResult {
	hit_enemy: Option<Entity>,
	hit_position: Vec2,
	ray_end_visual: Vec2,
	straight_end: Vec2,     // End of bowed section
	drift_end: Vec2,        // Where drift transitions to commit
	bow_control: Vec2,      // Bezier control point for bowed "straight" section
	curve_side: f32,        // -1.0 or 1.0, locked at fire time
}

/// Sample a quadratic bezier curve: B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
fn sample_bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
	let inv_t = 1.0 - t;
	inv_t * inv_t * p0 + 2.0 * inv_t * t * p1 + t * t * p2
}

fn perform_hitscan_ray(
	start_pos: Vec2,
	direction: Vec2,
	enemies: &Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
) -> RaycastResult {
	let mut rng = rand::thread_rng();

	// Lock curve direction at fire time (left or right)
	let curve_side: f32 = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
	let perpendicular = Vec2::new(-direction.y, direction.x) * curve_side;

	// Target Y for boom zone (consistent position near top)
	let target_boom_y = VIEWPORT_TOP - BOOM_ZONE_BELOW_TOP;

	// Calculate straight distance to get most of the way there
	// Leave room for drift + commit to finish the journey
	let distance_to_target = target_boom_y - start_pos.y;
	let straight_distance = (distance_to_target - DRIFT_DISTANCE - 50.0).max(100.0);

	// === PHASE 1: Bowed "straight" section ===
	// Endpoints are the same, but path bows out via bezier control point
	let bow_offset = rng.gen_range(BOW_OFFSET_MIN..BOW_OFFSET_MAX);
	let straight_end = start_pos + direction * straight_distance;
	let bow_control = start_pos.lerp(straight_end, 0.5) + perpendicular * bow_offset;

	// === PHASE 2: Drift section (gentle angle, same direction as bow) ===
	let drift_angle = rng.gen_range(DRIFT_ANGLE_MIN..DRIFT_ANGLE_MAX).to_radians() * curve_side;
	let drift_dir = rotate_direction(direction, drift_angle);
	let drift_end = straight_end + drift_dir * DRIFT_DISTANCE;

	// === PHASE 3: Commit section - calculate distance to land at target Y ===
	let commit_angle = rng.gen_range(COMMIT_ANGLE_MIN..COMMIT_ANGLE_MAX).to_radians() * curve_side;
	let commit_dir = rotate_direction(direction, commit_angle);

	// How much Y do we need to travel to reach target?
	let y_remaining = target_boom_y - drift_end.y;

	// Calculate commit distance based on Y component of commit_dir
	// commit_dir.y is how much Y we gain per unit of travel
	let commit_distance = if commit_dir.y.abs() > 0.01 && y_remaining > 0.0 {
		// Solve: drift_end.y + commit_dir.y * distance = target_boom_y
		(y_remaining / commit_dir.y).clamp(COMMIT_DISTANCE_MIN, COMMIT_DISTANCE_MAX)
	} else {
		// Fallback if commit is mostly horizontal or we're past target
		COMMIT_DISTANCE_MIN
	};

	let ray_end_visual = drift_end + commit_dir * commit_distance;

	// Collision detection along the full path
	let mut closest_hit: Option<(Entity, Vec2, f32)> = None;
	let mut accumulated_dist = 0.0;

	// Sample bowed section (bezier curve, 40 points)
	let mut prev_point = start_pos;
	for i in 1..=40 {
		let t = i as f32 / 40.0;
		let sample_point = sample_bezier(start_pos, bow_control, straight_end, t);
		accumulated_dist += prev_point.distance(sample_point);
		prev_point = sample_point;

		for (entity, transform, _, collider) in enemies.iter() {
			let enemy_pos = transform.translation.truncate();
			if sample_point.distance(enemy_pos) < collider.radius {
				if closest_hit.is_none() || accumulated_dist < closest_hit.as_ref().unwrap().2 {
					closest_hit = Some((entity, enemy_pos, accumulated_dist));
				}
			}
		}
	}

	// Sample drift section (15 points)
	if closest_hit.is_none() {
		for i in 1..=15 {
			let t = i as f32 / 15.0;
			let sample_point = straight_end.lerp(drift_end, t);
			accumulated_dist += prev_point.distance(sample_point);
			prev_point = sample_point;

			for (entity, transform, _, collider) in enemies.iter() {
				let enemy_pos = transform.translation.truncate();
				if sample_point.distance(enemy_pos) < collider.radius {
					if closest_hit.is_none() || accumulated_dist < closest_hit.as_ref().unwrap().2 {
						closest_hit = Some((entity, enemy_pos, accumulated_dist));
					}
				}
			}
		}
	}

	// Sample commit section (20 points)
	if closest_hit.is_none() {
		for i in 1..=20 {
			let t = i as f32 / 20.0;
			let sample_point = drift_end.lerp(ray_end_visual, t);
			accumulated_dist += prev_point.distance(sample_point);
			prev_point = sample_point;

			for (entity, transform, _, collider) in enemies.iter() {
				let enemy_pos = transform.translation.truncate();
				if sample_point.distance(enemy_pos) < collider.radius {
					if closest_hit.is_none() || accumulated_dist < closest_hit.as_ref().unwrap().2 {
						closest_hit = Some((entity, enemy_pos, accumulated_dist));
					}
				}
			}
		}
	}

	RaycastResult {
		hit_enemy: closest_hit.map(|(e, _, _)| e),
		hit_position: closest_hit.map(|(_, pos, _)| pos).unwrap_or(ray_end_visual),
		ray_end_visual,
		straight_end,
		drift_end,
		bow_control,
		curve_side,
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

fn spawn_glitter_burst(commands: &mut Commands, center: Vec2, radius: f32, incoming_direction: Option<Vec2>) {
	let mut rng = rand::thread_rng();
	let count = rng.gen_range(GLITTER_COUNT_MIN..=GLITTER_COUNT_MAX);

	// Oval stretch: longer along incoming direction, narrower perpendicular
	let forward = incoming_direction.unwrap_or(Vec2::Y).normalize();
	let perpendicular = Vec2::new(-forward.y, forward.x);
	let forward_stretch = 1.4;  // Stretch along bolt direction
	let perp_stretch = 0.7;     // Compress perpendicular

	for _ in 0..count {
		// Very slow random drift (firefly wander)
		let angle = rng.gen_range(0.0..std::f32::consts::TAU);
		let speed = rng.gen_range(GLITTER_SPEED_MIN..GLITTER_SPEED_MAX);
		let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

		// Spread in OVAL matching AoE shape
		let spawn_angle = rng.gen_range(0.0..std::f32::consts::TAU);
		let spawn_dist = rng.gen_range(0.0..radius);

		// Unit circle point
		let circle_point = Vec2::new(spawn_angle.cos(), spawn_angle.sin());

		// Project onto forward/perpendicular axes and apply stretch
		let forward_component = circle_point.dot(forward) * forward_stretch;
		let perp_component = circle_point.dot(perpendicular) * perp_stretch;

		// Reconstruct position in world space
		let oval_offset = (forward * forward_component + perpendicular * perp_component) * spawn_dist;
		let position = center + oval_offset;

		let lifetime = rng.gen_range(GLITTER_LIFETIME_MIN..GLITTER_LIFETIME_MAX);
		let size = rng.gen_range(GLITTER_SIZE_MIN..GLITTER_SIZE_MAX);
		let color_temp = rng.gen_range(0.0..1.0);

		// Random phase and speed for sine wave twinkle
		let phase = rng.gen_range(0.0..std::f32::consts::TAU);
		let twinkle_speed = rng.gen_range(8.0..16.0);

		commands.spawn(LightningGlitter {
			position,
			velocity,
			lifetime: Timer::from_seconds(lifetime, TimerMode::Once),
			initial_intensity: rng.gen_range(0.8..1.0),
			color_temp,
			size,
			phase,
			twinkle_speed,
		});
	}
}

fn execute_aoe_explosion(
	center: Vec2,
	radius: f32,
	base_damage: f32,
	enemies: &Query<(Entity, &Transform, &Health, &Collider), With<Enemy>>,
	hit_events: &mut EventWriter<EnemyHitEvent>,
	commands: &mut Commands,
	incoming_direction: Option<Vec2>,
	is_final_zone: bool,
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
		lifetime: Timer::from_seconds(if is_final_zone { 0.35 } else { 0.20 }, TimerMode::Once),
		intensity: if is_final_zone { 1.0 } else { 0.6 },
		incoming_direction,
		is_final_zone,
	});

	// Spawn glitter burst for final zones (incandescent firework dots)
	if is_final_zone {
		spawn_glitter_burst(commands, center, radius, incoming_direction);
	}
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
) -> bool {
	if recursion_depth >= 3 {
		return false;
	}

	let mut rng = rand::thread_rng();
	if rng.gen::<f32>() > baby_spawn_chance {
		return false;
	}

	// Highway off-ramp effect: spawn earlier, blend direction
	let chain_dir = (target_pos - parent_pos).normalize();
	let perpendicular = Vec2::new(-chain_dir.y, chain_dir.x);
	let side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };

	// Blend: 40% along parent direction + 60% perpendicular = smooth curve out
	let blended_dir = (chain_dir * 0.4 + perpendicular * side * 0.6).normalize();

	// Spawn earlier along chain (30%) so the curve starts sooner
	let spawn_point = parent_pos.lerp(target_pos, 0.3);

	commands.spawn(PendingBabyWhip {
		delay_timer: Timer::from_seconds(rng.gen_range(0.1..0.2), TimerMode::Once),
		spawn_from: spawn_point,
		direction: blended_dir,
		parent_chain_dir: chain_dir,
		parent_damage: parent_damage * 0.50,
		parent_level,
		parent_chain_range: chain_range * 0.8,
		parent_aoe_radius: aoe_radius * 0.7,
		recursion_depth: recursion_depth + 1,
		baby_spawn_chance: baby_spawn_chance * 0.6,
	});
	true
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

			// Spawn baby whip with delay - if spawned, add extended arc so baby doesn't appear from nowhere
			let baby_spawned = try_spawn_delayed_baby_whip(
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

			if baby_spawned {
				// Extended "ghost" arc that lingers until baby fires
				commands.spawn(LightningArc {
					start: current_pos,
					end: target_pos,
					lifetime: Timer::from_seconds(0.45, TimerMode::Once), // Longer for smooth baby transition
					thickness: 1.0,
					intensity: 0.4, // Dimmer, like a fading afterimage
				});
			}

			current_pos = target_pos;
		} else {
			break; // No more targets
		}
	}

	// Small AoE at final chain position (intermediate, not the main finale)
	execute_aoe_explosion(current_pos, aoe_radius * 0.3, damage * 0.3, &enemies, hit_events, commands, None, false);
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

	// Level-based config - AoE radius increased by 40%
	let (max_chains, chain_range, aoe_radius, damage_mult, damage_falloff) = match (level, is_charged) {
		(1, _) => (0, 0.0, 56.0, 1.0, 0.0),      // was 40
		(2, _) => (1, 150.0, 70.0, 1.0, 0.2),    // was 50
		(3, _) => (2, 180.0, 84.0, 1.0, 0.2),    // was 60
		(4, _) => (2, 200.0, 98.0, 1.0, 0.15),   // was 70
		(5, _) => (3, 250.0, 112.0, 1.0, 0.10),  // was 80
		(6, _) => (4, 280.0, 126.0, 1.0, 0.08),  // was 90
		(7, _) => (4, 300.0, 140.0, 1.0, 0.05),  // was 100
		(8, false) => (4, 300.0, 140.0, 1.0, 0.05),   // was 100
		(8, true) => (6, 350.0, 182.0, 1.375, 0.02),  // was 130
		(9, false) => (5, 350.0, 168.0, 1.0, 0.03),   // was 120
		(9, true) => (8, 450.0, 224.0, 1.556, 0.0),   // was 160
		(10, false) => (6, 400.0, 196.0, 1.0, 0.01),  // was 140
		(10, true) => (12, 550.0, 308.0, 1.8, 0.0),  // was 220
		_ => (6, 400.0, 196.0, 1.0, 0.01),           // was 140
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
			bow_control: ray_result.bow_control,
			straight_end: ray_result.straight_end,
			drift_end: ray_result.drift_end,
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

			// Small discharge at initial hit point (intermediate)
			execute_aoe_explosion(
				ray_result.hit_position,
				aoe_radius * 0.3,
				actual_damage * 0.25,
				enemies,
				hit_events,
				commands,
				None,
				false,
			);

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

			// Main finale discharge at bolt's visual end
			// Use the commit section's direction (end - drift_end)
			let final_direction = (ray_result.ray_end_visual - ray_result.drift_end).normalize_or_zero();
			execute_aoe_explosion(
				ray_result.ray_end_visual,
				aoe_radius,
				actual_damage * 0.5,
				enemies,
				hit_events,
				commands,
				Some(final_direction),
				true, // Final zone - splitting bolt visual
			);
		} else {
			// No hit - full finale AoE at end of bolt
			let final_direction = (ray_result.ray_end_visual - ray_result.drift_end).normalize_or_zero();
			execute_aoe_explosion(
				ray_result.ray_end_visual,
				aoe_radius,
				actual_damage,
				enemies,
				hit_events,
				commands,
				Some(final_direction),
				true, // Final zone - splitting bolt visual
			);
		}
	}

	// Audio - main fire sounds (one per whip, staggered by 15ms)
	let fire_sound = if is_charged {
		"sounds/lightning/lightning_charged.ogg"
	} else {
		"sounds/lightning/lightning_standard.ogg"
	};

	for whip_index in 0..num_whips {
		let delay_secs = (whip_index as f32) * 0.015; // 15ms per whip
		commands.spawn(PendingSound {
			delay: Timer::from_seconds(delay_secs, TimerMode::Once),
			sound_path: fire_sound,
			volume: 0.42,
			fade_after: None,
			fade_duration: 0.0,
		});
	}

	// Pooled boom+crackle sounds (delayed, with fade after 300ms)
	// Boom: 50ms delay
	commands.spawn(PendingSound {
		delay: Timer::from_seconds(0.05, TimerMode::Once),
		sound_path: "sounds/lightning/deeper_boom_final.ogg",
		volume: 1.5,
		fade_after: Some(0.3),
		fade_duration: 0.2,
	});
	// Crackle: removed for now (may use on discharge later)

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
	bolt: &LightningBolt,
	displacement: f32,
) -> Vec<Vec2> {
	let mut rng = rand::thread_rng();
	let mut segments = vec![bolt.start];

	// Random phase offset for sine wave (different each frame = flowing effect)
	let phase = rng.gen_range(0.0..std::f32::consts::TAU);
	let wave_freq = rng.gen_range(2.5..4.0);

	// === PHASE 1: Bowed section (bezier curve from start to straight_end) ===
	let bow_segments = 16;
	for i in 1..bow_segments {
		let t = i as f32 / bow_segments as f32;
		let base_point = sample_bezier(bolt.start, bolt.bow_control, bolt.straight_end, t);

		// Add small jitter perpendicular to the curve
		let direction = (bolt.straight_end - bolt.start).normalize();
		let perpendicular = Vec2::new(-direction.y, direction.x);
		let wave = (t * wave_freq * std::f32::consts::TAU + phase).sin() * displacement * 0.5;
		let jitter = rng.gen_range(-displacement * 0.3..displacement * 0.3);

		segments.push(base_point + perpendicular * (wave + jitter));
	}
	segments.push(bolt.straight_end);

	// === PHASE 2: Drift section (straight_end to drift_end) ===
	let drift_segments = 5;
	for i in 1..drift_segments {
		let t = i as f32 / drift_segments as f32;
		let base_point = bolt.straight_end.lerp(bolt.drift_end, t);

		let direction = (bolt.drift_end - bolt.straight_end).normalize();
		let perpendicular = Vec2::new(-direction.y, direction.x);
		let wave = (t * wave_freq * 1.2 * std::f32::consts::TAU + phase).sin() * displacement * 0.7;
		let jitter = rng.gen_range(-displacement * 0.4..displacement * 0.4);

		segments.push(base_point + perpendicular * (wave + jitter));
	}
	segments.push(bolt.drift_end);

	// === PHASE 3: Commit section (drift_end to end) ===
	let commit_segments = 8;
	for i in 1..commit_segments {
		let t = i as f32 / commit_segments as f32;
		let base_point = bolt.drift_end.lerp(bolt.end, t);

		let direction = (bolt.end - bolt.drift_end).normalize();
		let perpendicular = Vec2::new(-direction.y, direction.x);
		let wave = (t * wave_freq * 1.5 * std::f32::consts::TAU + phase).sin() * displacement;
		let jitter = rng.gen_range(-displacement * 0.5..displacement * 0.5);

		segments.push(base_point + perpendicular * (wave + jitter));
	}
	segments.push(bolt.end);

	segments
}

pub fn render_lightning_bolts(
	mut gizmos: Gizmos,
	bolts: Query<&LightningBolt>,
) {
	for bolt in bolts.iter() {
		let alpha = bolt.lifetime.fraction_remaining();

		// Generate jagged path every frame
		let displacement = if bolt.is_baby { 10.0 } else { 15.0 };
		let segments = generate_curved_lightning_path(bolt, displacement);

		// Multi-layer lightning rendering - sci-fi power conduit style
		for (i, window) in segments.windows(2).enumerate() {
			let t = i as f32 / (segments.len() - 1) as f32;
			let thickness = bolt.thickness_start + (bolt.thickness_end - bolt.thickness_start) * t;

			// Wide outer glow (deep blue, ethereal)
			let outer_glow = Color::srgba(0.1, 0.3, 0.9, alpha * 0.15);
			for offset in (-5..=5).map(|o| o as f32 * 2.0) {
				gizmos.line_2d(
					window[0] + Vec2::new(offset, offset * 0.3),
					window[1] + Vec2::new(offset, offset * 0.3),
					outer_glow,
				);
			}

			// Mid-outer glow (electric blue)
			let mid_outer = Color::srgba(0.2, 0.5, 1.0, alpha * 0.25);
			for offset in (-3..=3).map(|o| o as f32 * 1.2) {
				gizmos.line_2d(
					window[0] + Vec2::new(offset, 0.0),
					window[1] + Vec2::new(offset, 0.0),
					mid_outer,
				);
			}

			// Inner glow (bright blue-white)
			let inner_glow = Color::srgba(0.4, 0.7, 1.0, alpha * 0.6);
			for offset in [-thickness * 0.5, 0.0, thickness * 0.5] {
				gizmos.line_2d(
					window[0] + Vec2::new(offset, 0.0),
					window[1] + Vec2::new(offset, 0.0),
					inner_glow,
				);
			}

			// Hot core (near-white with blue tint)
			let core_color = Color::srgba(0.7, 0.85, 1.0, alpha * bolt.intensity);
			gizmos.line_2d(window[0], window[1], core_color);

			// Blazing center (pure white-blue)
			let center = Color::srgba(0.9, 0.95, 1.0, alpha * bolt.intensity * 0.9);
			gizmos.line_2d(
				window[0] + Vec2::new(0.3, 0.0),
				window[1] + Vec2::new(0.3, 0.0),
				center,
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

			// Draw with blue glow layers
			for w in points.windows(2) {
				let p1 = w[0];
				let p2 = w[1];

				// Outer glow (deep blue)
				let outer_color = Color::srgba(0.15, 0.4, 0.95, alpha * impact.intensity * 0.25);
				gizmos.line_2d(p1 + Vec2::new(-2.5, 0.0), p2 + Vec2::new(-2.5, 0.0), outer_color);
				gizmos.line_2d(p1 + Vec2::new(2.5, 0.0), p2 + Vec2::new(2.5, 0.0), outer_color);
				gizmos.line_2d(p1 + Vec2::new(0.0, -2.5), p2 + Vec2::new(0.0, -2.5), outer_color);
				gizmos.line_2d(p1 + Vec2::new(0.0, 2.5), p2 + Vec2::new(0.0, 2.5), outer_color);

				// Middle glow (electric blue)
				let mid_color = Color::srgba(0.3, 0.6, 1.0, alpha * impact.intensity * 0.5);
				gizmos.line_2d(p1, p2, mid_color);

				// Bright core (blue-white)
				let core_color = Color::srgba(0.7, 0.85, 1.0, alpha * impact.intensity * 0.85);
				gizmos.line_2d(
					p1 + Vec2::new(0.3, 0.3),
					p2 + Vec2::new(0.3, 0.3),
					core_color
				);
			}
		}

		// Central flash (blue-white)
		let center_color = Color::srgba(0.8, 0.9, 1.0, alpha * impact.intensity);
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
		let progress = 1.0 - alpha;

		if aoe.is_final_zone {
			// === FINAL ZONE: Splitting bolt with fireworks burst ===
			let base_dir = aoe.incoming_direction.unwrap_or(Vec2::Y);
			let finger_count = 5;
			let max_spread = 50.0_f32.to_radians(); // ±50° spread for wide oval

			// Darkening void at center
			let dark_radius = aoe.radius * 0.4;
			for i in 0..8 {
				let ring_t = i as f32 / 8.0;
				let darkness = Color::srgba(0.0, 0.0, 0.15, alpha * 0.5 * (1.0 - ring_t));
				gizmos.circle_2d(aoe.position, dark_radius * ring_t, darkness);
			}

			// Splitting bolt fingers
			for i in 0..finger_count {
				let spread_t = (i as f32 / (finger_count - 1) as f32) - 0.5; // -0.5 to 0.5
				let angle_offset = spread_t * 2.0 * max_spread;

				// Rotate base direction by angle offset
				let cos_a = angle_offset.cos();
				let sin_a = angle_offset.sin();
				let finger_dir = Vec2::new(
					base_dir.x * cos_a - base_dir.y * sin_a,
					base_dir.x * sin_a + base_dir.y * cos_a,
				);

				let finger_length = aoe.radius * (0.7 + progress * 0.3);
				let finger_end = aoe.position + finger_dir * finger_length;

				// Generate curvy path for this finger
				let perpendicular = Vec2::new(-finger_dir.y, finger_dir.x);
				let segment_count = 6;
				let mut points = vec![aoe.position];
				let phase = rng.gen_range(0.0..std::f32::consts::TAU);
				let wave_amp = rng.gen_range(8.0..16.0);

				for j in 1..segment_count {
					let t = j as f32 / segment_count as f32;
					let base_point = aoe.position.lerp(finger_end, t);
					let wave = (t * 2.5 * std::f32::consts::TAU + phase).sin() * wave_amp * (1.0 - t * 0.5);
					let jitter = rng.gen_range(-4.0..4.0);
					points.push(base_point + perpendicular * (wave + jitter));
				}
				points.push(finger_end);

				// Draw finger with multi-layer glow
				for w in points.windows(2) {
					let p1 = w[0];
					let p2 = w[1];

					// Wide outer glow
					let outer = Color::srgba(0.1, 0.3, 0.9, alpha * 0.25);
					gizmos.line_2d(p1 + perpendicular * 3.0, p2 + perpendicular * 3.0, outer);
					gizmos.line_2d(p1 - perpendicular * 3.0, p2 - perpendicular * 3.0, outer);

					// Mid glow
					let mid = Color::srgba(0.3, 0.6, 1.0, alpha * 0.5);
					gizmos.line_2d(p1, p2, mid);

					// Bright core
					let core = Color::srgba(0.7, 0.9, 1.0, alpha * 0.8);
					gizmos.line_2d(p1 + perpendicular * 0.5, p2 + perpendicular * 0.5, core);
				}

				// Sparkles along finger path
				for point in &points {
					if rng.gen_bool(0.6) {
						let spark_color = Color::srgba(0.8, 0.9, 1.0, alpha * rng.gen_range(0.5..1.0));
						let spark_dir = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero();
						let spark_len = rng.gen_range(3.0..8.0);
						gizmos.line_2d(*point, *point + spark_dir * spark_len, spark_color);
					}
				}

				// Fireworks burst at finger tip
				let burst_count = 6;
				for _ in 0..burst_count {
					let burst_angle = rng.gen_range(0.0..std::f32::consts::TAU);
					let burst_len = rng.gen_range(6.0..18.0) * alpha;
					let burst_end = finger_end + Vec2::new(burst_angle.cos(), burst_angle.sin()) * burst_len;
					let burst_color = Color::srgba(0.6, 0.8, 1.0, alpha * 0.7);
					gizmos.line_2d(finger_end, burst_end, burst_color);

					// Tiny spark at burst end
					if rng.gen_bool(0.5) {
						let tiny_color = Color::srgba(0.9, 0.95, 1.0, alpha * 0.9);
						gizmos.circle_2d(burst_end, 1.5, tiny_color);
					}
				}
			}

			// Central flash
			let flash_color = Color::srgba(0.8, 0.9, 1.0, (alpha * 3.0).min(1.0) * 0.6);
			gizmos.circle_2d(aoe.position, 4.0, flash_color);

		} else {
			// === INTERMEDIATE ZONE: Small confetti (existing but smaller) ===
			let chaos_radius = aoe.radius * (0.5 + progress * 0.5);
			let arc_count = 8 + (4.0 * alpha) as usize;

			for _ in 0..arc_count {
				let start_angle = rng.gen_range(0.0..std::f32::consts::TAU);
				let start_dist = rng.gen_range(0.0..chaos_radius * 0.2);
				let start = aoe.position + Vec2::new(start_angle.cos(), start_angle.sin()) * start_dist;

				let end_angle = rng.gen_range(0.0..std::f32::consts::TAU);
				let end_dist = rng.gen_range(0.4..1.0) * chaos_radius;
				let end = aoe.position + Vec2::new(end_angle.cos(), end_angle.sin()) * end_dist;

				// Simple 2-segment arc
				let mid = start.lerp(end, 0.5) + Vec2::new(rng.gen_range(-5.0..5.0), rng.gen_range(-5.0..5.0));

				let arc_color = Color::srgba(0.4, 0.6, 1.0, alpha * 0.5);
				gizmos.line_2d(start, mid, arc_color);
				gizmos.line_2d(mid, end, arc_color);
			}

			// Small center flash
			let flash_color = Color::srgba(0.6, 0.8, 1.0, alpha * 0.4);
			gizmos.circle_2d(aoe.position, 3.0, flash_color);
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
				bow_control: ray_result.bow_control,
				straight_end: ray_result.straight_end,
				drift_end: ray_result.drift_end,
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

			// Baby whip AoE is 60% of parent size
			let baby_aoe_radius = pending_whip.parent_aoe_radius * 0.6;

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

				// Baby discharge at hit point (25% damage)
				execute_aoe_explosion(
					ray_result.hit_position,
					baby_aoe_radius,
					pending_whip.parent_damage * 0.25,
					&enemies,
					&mut hit_events,
					&mut commands,
					None,
					false,
				);

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
					baby_aoe_radius,
					pending_whip.baby_spawn_chance,
					pending_whip.recursion_depth,
					&audio,
					&asset_server,
				);

				// Baby discharge at bolt visual end (15% damage)
				execute_aoe_explosion(
					ray_result.ray_end_visual,
					baby_aoe_radius * 0.8,
					pending_whip.parent_damage * 0.15,
					&enemies,
					&mut hit_events,
					&mut commands,
					None,
					false,
				);
			} else {
				// No hit - discharge at end of baby bolt
				execute_aoe_explosion(
					ray_result.ray_end_visual,
					baby_aoe_radius,
					pending_whip.parent_damage * 0.5,
					&enemies,
					&mut hit_events,
					&mut commands,
					None,
					false,
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
	mut glitter: Query<(Entity, &mut LightningGlitter)>,
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

	for (entity, mut g) in glitter.iter_mut() {
		g.lifetime.tick(time.delta());
		if g.lifetime.finished() {
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

pub fn update_lightning_glitter(
	time: Res<Time>,
	mut glitter: Query<&mut LightningGlitter>,
) {
	let dt = time.delta_secs();
	for mut g in glitter.iter_mut() {
		// Drift outward
		let vel = g.velocity;
		g.position += vel * dt;
		// Slow down over time (air resistance feel)
		g.velocity *= 0.97;
	}
}

pub fn render_lightning_glitter(
	mut gizmos: Gizmos,
	glitter: Query<&LightningGlitter>,
) {
	let mut rng = rand::thread_rng();

	for g in glitter.iter() {
		let life_remaining = g.lifetime.fraction_remaining();

		// Subtle flicker (less aggressive - the color shift is the star)
		let flicker = if rng.gen_bool(0.08) {
			0.0 // Brief blink off
		} else {
			rng.gen_range(0.75..1.0)
		};

		let base_alpha = (life_remaining * 0.7 + 0.3) * g.initial_intensity * flicker;

		if base_alpha < 0.05 {
			continue;
		}

		// ELECTRIC BURN: white-blue hot → cyan → deep blue (with rare yellow sparks)
		// life_remaining: 1.0 = just spawned (hot), 0.0 = about to die (fading)
		let is_yellow_spark = g.color_temp > 0.88; // ~12% are yellow outliers

		let (r, g_col, b) = if is_yellow_spark {
			// Rare yellow/gold spark (electrical arc color)
			if life_remaining > 0.5 {
				(1.0, 0.95, 0.5) // Bright yellow
			} else {
				let t = life_remaining / 0.5;
				(0.9 * t + 0.3, 0.8 * t + 0.2, 0.3 * t) // Fade to dim gold
			}
		} else if life_remaining > 0.7 {
			// White-blue hot (freshly spawned)
			let t = (life_remaining - 0.7) / 0.3;
			(0.85 + t * 0.15, 0.95 + t * 0.05, 1.0)
		} else if life_remaining > 0.4 {
			// Electric cyan
			let t = (life_remaining - 0.4) / 0.3;
			(0.4 + t * 0.45, 0.8 + t * 0.15, 1.0)
		} else if life_remaining > 0.15 {
			// Cooling to deeper blue
			let t = (life_remaining - 0.15) / 0.25;
			(0.2 + t * 0.2, 0.5 + t * 0.3, 0.8 + t * 0.2)
		} else {
			// Dim blue ember (dying)
			let t = life_remaining / 0.15;
			(0.1 + t * 0.1, 0.2 + t * 0.3, 0.4 + t * 0.4)
		};

		let color = Color::srgba(r, g_col, b, base_alpha);

		// Micro spark: just a tiny cross, shrinks as it cools
		let arm = g.size * (0.5 + life_remaining * 0.5);

		if arm < 0.3 {
			// Too small - render as a point (circle)
			gizmos.circle_2d(g.position, 0.3, color);
		} else {
			// Tiny cross
			gizmos.line_2d(
				g.position + Vec2::new(0.0, -arm),
				g.position + Vec2::new(0.0, arm),
				color,
			);
			gizmos.line_2d(
				g.position + Vec2::new(-arm, 0.0),
				g.position + Vec2::new(arm, 0.0),
				color,
			);
		}
	}
}
