use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::fs;
use crate::level::{LevelData, LevelDataV2, LevelDataV3};
use crate::components::DistanceLocked;
use super::world::{sizes, doodad_sizes};

#[derive(Resource, Default)]
pub struct SelectedLevel {
	pub level_number: u32,
	pub start_distance: f32,
}

impl SelectedLevel {
	pub fn new(level: u32) -> Self {
		Self { level_number: level, start_distance: 0.0 }
	}

	pub fn with_start_distance(level: u32, start_distance: f32) -> Self {
		Self { level_number: level, start_distance }
	}
}

#[derive(Resource)]
pub struct MusicState {
	pub current_track: Option<String>,
	pub handle: Option<Handle<AudioInstance>>,
	pub selected_genre: String,
	pub crossfade_duration: f32,
}

impl Default for MusicState {
	fn default() -> Self {
		Self {
			current_track: None,
			handle: None,
			selected_genre: Self::pick_random_genre(),
			crossfade_duration: 1.0,
		}
	}
}

#[derive(Resource, Default)]
pub struct TitleMusicState {
	pub handle: Option<Handle<AudioInstance>>,
	pub current_track: Option<String>,
}

impl MusicState {
	pub fn pick_random_genre() -> String {
		// Scan assets/music/ for available genre directories
		// Skip directories starting with _ (reserved for technical music)
		if let Ok(entries) = fs::read_dir("assets/music") {
			let genres: Vec<String> = entries
				.filter_map(|entry| entry.ok())
				.filter(|entry| entry.path().is_dir())
				.filter_map(|entry| {
					entry.file_name().to_str().map(|s| s.to_string())
				})
				.filter(|name| !name.starts_with('_'))  // Skip _prefixed dirs
				.collect();

			if !genres.is_empty() {
				return genres[rand::random::<usize>() % genres.len()].clone();
			}
		}

		// Fallback to default if scan fails
		"orchestral-rock".to_string()
	}

	// Pick random variant of a track (e.g., phase1_calm.mp3 vs phase1_calm_2.mp3)
	fn pick_track_variant(base_path: &str) -> String {
		use std::path::Path;

		// Extract directory and filename
		let path = Path::new(base_path);
		let parent = path.parent().unwrap_or(Path::new(""));
		let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
		let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("mp3");

		// Look for variants: base.mp3, base_2.mp3, base_3.mp3, etc.
		let mut variants = vec![base_path.to_string()];

		for i in 2..=10 {  // Check up to _10
			let variant_path = parent.join(format!("{}_{}.{}", file_stem, i, extension));
			if variant_path.exists() {
				variants.push(variant_path.to_str().unwrap_or(base_path).to_string());
			}
		}

		// Pick random variant
		variants[rand::random::<usize>() % variants.len()].clone()
	}
}

#[derive(Resource, Default)]
pub struct DebugSpeed {
	pub enabled: bool,
	pub multiplier: f32,
}

impl DebugSpeed {
	pub fn new() -> Self {
		Self { enabled: false, multiplier: 10.0 }
	}
}

#[derive(Resource)]
pub struct MusicEnabled {
	pub enabled: bool,
}

impl Default for MusicEnabled {
	fn default() -> Self {
		Self { enabled: true }
	}
}

impl MusicEnabled {
	pub fn new(enabled: bool) -> Self {
		Self { enabled }
	}
}

#[derive(Resource, Default)]
pub struct GamePaused(pub bool);

#[derive(Resource, Default)]
pub struct InfoOverlayEnabled(pub bool);

pub fn toggle_pause(
	keyboard: Res<ButtonInput<KeyCode>>,
	mut paused: ResMut<GamePaused>,
) {
	if keyboard.just_pressed(KeyCode::KeyQ) {
		paused.0 = !paused.0;
		if paused.0 {
			info!("⏸ PAUSED - press Q to resume");
		} else {
			info!("▶ RESUMED");
		}
	}
}

pub fn toggle_info_overlay(
	keyboard: Res<ButtonInput<KeyCode>>,
	mut info_enabled: ResMut<InfoOverlayEnabled>,
) {
	if keyboard.just_pressed(KeyCode::F3) {
		info_enabled.0 = !info_enabled.0;
	}
}

pub fn toggle_debug_speed(
	keyboard: Res<ButtonInput<KeyCode>>,
	mut debug_speed: ResMut<DebugSpeed>,
) {
	if keyboard.just_pressed(KeyCode::KeyZ) {
		debug_speed.enabled = !debug_speed.enabled;
		if debug_speed.enabled {
			info!("⚡ Debug speed: 10x");
		} else {
			info!("⚡ Debug speed: normal");
		}
	}
}

#[derive(Resource)]
pub struct CurrentLevel {
	pub data: LevelData,
	pub distance: f32,  // Total distance traveled in GU
	pub last_milestone: u32,  // Last logged 1000 GU milestone
	pub time_elapsed: f32,  // Time elapsed since level start (seconds)
	pub processed_events: Vec<usize>,
	pub processed_waves: Vec<usize>,
	pub processed_doodads: Vec<usize>,
	pub processed_structures: Vec<usize>,
	pub spawned_enemies: std::collections::HashSet<(usize, usize)>,  // (wave_idx, enemy_idx)
}

impl CurrentLevel {
	pub fn new(data: LevelData) -> Self {
		Self::with_start_distance(data, 0.0)
	}

	pub fn with_start_distance(data: LevelData, start_distance: f32) -> Self {
		// Pre-mark items before start_distance as processed (skip them)
		let processed_doodads: Vec<usize> = data.doodads.iter()
			.enumerate()
			.filter(|(_, d)| d.spawn_distance < start_distance)
			.map(|(i, _)| i)
			.collect();

		let processed_waves: Vec<usize> = data.enemy_waves.iter()
			.enumerate()
			.filter(|(_, w)| w.spawn_distance < start_distance)
			.map(|(i, _)| i)
			.collect();

		let processed_events: Vec<usize> = data.events.iter()
			.enumerate()
			.filter(|(_, e)| e.distance < start_distance)
			.map(|(i, _)| i)
			.collect();

		let processed_structures: Vec<usize> = data.structures.iter()
			.enumerate()
			.filter(|(_, s)| s.spawn_distance < start_distance)
			.map(|(i, _)| i)
			.collect();

		if start_distance > 0.0 {
			info!("Starting at distance {}: skipping {} doodads, {} waves, {} events, {} structures",
				start_distance, processed_doodads.len(), processed_waves.len(),
				processed_events.len(), processed_structures.len());
		}

		Self {
			data,
			distance: start_distance,
			last_milestone: (start_distance / 1000.0) as u32,
			time_elapsed: 0.0,
			processed_events,
			processed_waves,
			processed_doodads,
			processed_structures,
			spawned_enemies: std::collections::HashSet::new(),
		}
	}

	pub fn get_current_phase(&self) -> Option<&crate::level::Phase> {
		self.data.phases.iter().find(|p| {
			self.distance >= p.start_distance && self.distance < p.end_distance
		})
	}

	pub fn get_scroll_speed(&self) -> f32 {
		self.get_current_phase()
			.map(|p| p.scroll_speed)
			.unwrap_or(100.0)
	}
}

pub fn load_level(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	selected_level: Option<Res<SelectedLevel>>,
) {
	let (level_num, start_distance) = selected_level
		.map(|l| (l.level_number, l.start_distance))
		.unwrap_or((1, 0.0));
	let yaml_path = format!("assets/level-defs/level{}.yaml", level_num);

	// Try working directory first (for cargo run), then exe directory (for distribution)
	let yaml_content = fs::read_to_string(&yaml_path).or_else(|_| {
		let exe_dir = std::env::current_exe()
			.ok()
			.and_then(|p| p.parent().map(|p| p.to_path_buf()))
			.unwrap_or_default();
		fs::read_to_string(exe_dir.join(&yaml_path))
	});

	match yaml_content {
		Ok(yaml_str) => {
			// Try V3 (geography/sections), then V2 (zones), then V1 (raw doodads)
			let level = match serde_yaml::from_str::<LevelDataV3>(&yaml_str) {
				Ok(v3_level) if !v3_level.sections.is_empty() => {
					info!("✓ Loaded V3 geography-based level with {} sections", v3_level.sections.len());
					v3_level.to_level_data()
				}
				_ => {
					match serde_yaml::from_str::<LevelDataV2>(&yaml_str) {
						Ok(v2_level) if !v2_level.zones.is_empty() => {
							info!("✓ Loaded V2 zone-based level with {} zones", v2_level.zones.len());
							v2_level.to_level_data()
						}
						_ => serde_yaml::from_str::<LevelData>(&yaml_str).expect("Failed to parse level YAML")
					}
				}
			};

			// Expand geography into doodads (tiles are doodads)
			let mut expanded_level = level.clone();
			let mut geo_doodads = Vec::new();
			for geo in &expanded_level.geography {
				geo_doodads.extend(geo.expand_to_doodads());
			}
			expanded_level.doodads.extend(geo_doodads);
			info!("✓ Expanded {} geography elements into doodads", expanded_level.geography.len());

			// Expand structure_grids into structures
			let grid_count = expanded_level.structure_grids.len();
			let mut grid_structures = Vec::new();
			for grid in &expanded_level.structure_grids {
				grid_structures.extend(grid.expand_to_structures());
			}
			let grid_structure_count = grid_structures.len();
			expanded_level.structures.extend(grid_structures);
			if grid_count > 0 {
				info!("✓ Expanded {} structure grids into {} structures",
					grid_count, grid_structure_count);
			}

			// Keep structures separate - don't merge into doodads
			info!("✓ Loaded {} structures (processed separately)", expanded_level.structures.len());

			info!("✓ Loaded level: {} ({} doodads from {} geography + manual, {} waves)",
				expanded_level.name, expanded_level.doodads.len(), expanded_level.geography.len(), expanded_level.enemy_waves.len());

			// Spawn backdrop items (static deep space elements)
			for (i, item) in expanded_level.backdrop.iter().enumerate() {
				let sprite_path = format!("backdrop/{}", item.sprite);
				let size = item.size.map(|s| Vec2::new(s[0], s[1]))
					.unwrap_or(Vec2::new(800.0, 800.0));

				commands.spawn((
					Sprite {
						image: asset_server.load(&sprite_path),
						custom_size: Some(size),
						color: Color::srgba(1.0, 1.0, 1.0, item.alpha),
						..default()
					},
					Transform::from_xyz(item.position[0], item.position[1], -9.5 + (i as f32 * 0.01)),
					BackdropEntity,
				));
				info!("Spawned backdrop: {} at ({}, {})", item.sprite, item.position[0], item.position[1]);
			}

			commands.insert_resource(CurrentLevel::with_start_distance(expanded_level, start_distance));
		}
		Err(e) => {
			error!("Failed to read level file {}: {}", yaml_path, e);
		}
	}
}

pub fn update_level_timer(
	mut level: ResMut<CurrentLevel>,
	time: Res<Time>,
	debug_speed: Res<DebugSpeed>,
	paused: Res<GamePaused>,
) {
	if paused.0 { return; }

	let mut scroll_speed = level.get_scroll_speed();
	if debug_speed.enabled {
		scroll_speed *= debug_speed.multiplier;
	}
	level.distance += scroll_speed * time.delta_secs();
	level.time_elapsed += time.delta_secs();

	// Log milestone every 1000 GU
	let milestone = (level.distance / 1000.0) as u32;
	if milestone > level.last_milestone {
		info!("📍 Distance: {} GU", milestone * 1000);
		level.last_milestone = milestone;
	}
}

pub fn process_enemy_waves(
	mut level: ResMut<CurrentLevel>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut formation_registry: ResMut<crate::components::FormationRegistry>,
) {
	use crate::components::{Enemy, EnemyType, EnemyMovement, MovementPattern, EnemyBehavior, FormationLeader, FormationMember, Health, Collider};
	use crate::level::{FormationRole, EnemySpawn};

	let current_distance = level.distance;
	let scroll_speed = level.get_scroll_speed();

	let mut waves_to_process = Vec::new();
	for (wave_idx, wave) in level.data.enemy_waves.iter().enumerate() {
		// Check if we should spawn any enemies in this wave
		// Each enemy may have a different spawn offset based on its Y position
		if !level.processed_waves.contains(&wave_idx) {
			// Check if ANY enemy in this wave should spawn
			let should_spawn = wave.enemies.iter().any(|enemy| {
				// Calculate adjusted spawn distance based on target Y position
				// YAML Y position determines WHEN to spawn (higher Y = spawn later/closer to spawn_distance)
				// All enemies spawn at Y=600, then scroll down with velocity=-100
				let target_y = enemy.position[1];
				let spawn_y = 600.0;
				let distance_to_travel = spawn_y - target_y;  // How far to scroll before reaching target Y
				let velocity_y: f32 = 100.0;  // Scroll velocity (abs value)
				let time_to_reach_target = distance_to_travel / velocity_y;
				let distance_offset = time_to_reach_target * scroll_speed;  // Convert time to GU distance

				// Spawn early enough so enemy reaches target Y position when player reaches spawn_distance
				let adjusted_spawn_distance = wave.spawn_distance - distance_offset;

				current_distance >= adjusted_spawn_distance
			});

			if should_spawn {
				waves_to_process.push((wave_idx, wave.clone()));
			}
		}
	}

	for (wave_idx, wave) in waves_to_process {
		let mut all_enemies_spawned = true;

		for (enemy_idx, enemy) in wave.enemies.iter().enumerate() {
			// Skip if this specific enemy already spawned
			if level.spawned_enemies.contains(&(wave_idx, enemy_idx)) {
				continue;
			}

			// Calculate if THIS specific enemy should spawn based on its Y position
			let target_y = enemy.position[1];
			let spawn_y = 600.0;
			let distance_to_travel = spawn_y - target_y;
			let velocity_y: f32 = 100.0;  // Scroll velocity (abs value)
			let time_to_reach_target = distance_to_travel / velocity_y;
			let distance_offset = time_to_reach_target * scroll_speed;
			let adjusted_spawn_distance = wave.spawn_distance - distance_offset;

			// Skip this enemy if it's not time to spawn yet
			if current_distance < adjusted_spawn_distance {
				all_enemies_spawned = false;
				continue;
			}

			let (sprite_path, size, enemy_type) = match enemy.enemy_type.as_str() {
				"Scout" => ("enemies/scout.png", sizes::SCOUT, EnemyType::Scout),
				"Fighter" => ("enemies/fighter.png", sizes::FIGHTER, EnemyType::Fighter),
				"HeavyGunship" => ("enemies/heavy_gunship.png", sizes::HEAVY_GUNSHIP, EnemyType::HeavyGunship),
				"Boss" => ("enemies/boss.png", sizes::BOSS, EnemyType::Boss),
				"Interceptor" => ("enemies/interceptor.png", sizes::INTERCEPTOR, EnemyType::Interceptor),
				"Drone" => ("enemies/drone.png", sizes::DRONE, EnemyType::Drone),
				"Bomber" => ("enemies/bomber.png", sizes::BOMBER, EnemyType::Bomber),
				"Corvette" => ("enemies/corvette.png", sizes::CORVETTE, EnemyType::Corvette),
				"SmallAsteroid" => ("enemies/small_asteroid.png", sizes::SMALL_ASTEROID, EnemyType::SmallAsteroid),
				"MediumAsteroid" => ("enemies/medium_asteroid.png", sizes::MEDIUM_ASTEROID, EnemyType::MediumAsteroid),
				"LargeAsteroid" => ("enemies/large_asteroid.png", sizes::LARGE_ASTEROID, EnemyType::LargeAsteroid),
				"StationDebris" => ("enemies/station_debris.png", sizes::STATION_DEBRIS, EnemyType::StationDebris),
				_ => ("enemies/scout.png", sizes::SCOUT, EnemyType::Scout),
			};

			let mut behaviors = enemy.get_behaviors();
			if behaviors.is_empty() {
				behaviors = EnemySpawn::get_default_behavior_for_type(&enemy.enemy_type);
			}

			if !behaviors.is_empty() {
				// Always spawn enemies above viewport (Y=600) so they scroll into view naturally
				// YAML Y position is ignored - enemies enter from top of screen
				let spawn_y = 600.0;  // Above viewport top edge (Y=500)

				let mut entity_commands = commands.spawn((
					Sprite {
						image: asset_server.load(sprite_path),
						custom_size: Some(Vec2::splat(size)),
						..default()
					},
					Transform::from_xyz(enemy.position[0], spawn_y, 0.5),
					Enemy { enemy_type },
					EnemyBehavior {
						behaviors: behaviors.clone(),
						current_index: 0,
						behavior_start_time: 0.0,
						total_time_alive: 0.0,
						spawn_position: Vec2::new(enemy.position[0], spawn_y),
					},
					Health::for_enemy_type(enemy_type),
					Collider::for_enemy_type(enemy_type),
				));

				if let Some(ref formation_id) = enemy.formation_id {
					match enemy.formation_role {
						Some(FormationRole::Leader) => {
							let entity_id = entity_commands.id();
							entity_commands.insert(FormationLeader {
								formation_id: formation_id.clone(),
								member_offsets: Vec::new(),
							});
							formation_registry.formations.insert(formation_id.clone(), entity_id);
						}
						Some(FormationRole::Member) => {
							if let Some(leader_entity) = formation_registry.formations.get(formation_id) {
								let offset = enemy.formation_offset
									.map(|o| Vec2::new(o[0], o[1]))
									.unwrap_or(Vec2::ZERO);
								entity_commands.insert(FormationMember {
									formation_id: formation_id.clone(),
									leader: *leader_entity,
									offset,
								});
							}
						}
						None => {}
					}
				}

				// Mark this enemy as spawned
				level.spawned_enemies.insert((wave_idx, enemy_idx));

				info!(
					"✨ Spawned {:?} at X={:.1}, Y=600 (scroll to target Y={:.1}) - dist={:.1}→{:.1}",
					enemy_type, enemy.position[0], target_y, wave.spawn_distance, adjusted_spawn_distance
				);
				if enemy_type == EnemyType::Boss {
					info!("🎯 BOSS SPAWNED! Position: ({:.1}, {:.1}), Behaviors: {:?}",
						enemy.position[0], enemy.position[1], behaviors);
				}
			} else {
				let movement_pattern = if let Some(ref movement) = enemy.movement {
					match movement.as_str() {
						"SineWave" => MovementPattern::SineWave { amplitude: 100.0, frequency: 2.0 },
						"PassBy" => MovementPattern::PassBy { speed: 150.0 },
						"Circle" => MovementPattern::Circle { radius: 80.0, angular_speed: 1.5 },
						"Straight" => MovementPattern::Straight { speed: 100.0 },
						_ => MovementPattern::Straight { speed: 100.0 },
					}
				} else {
					MovementPattern::Straight { speed: 100.0 }
				};

				commands.spawn((
					Sprite {
						image: asset_server.load(sprite_path),
						custom_size: Some(Vec2::splat(size)),
						..default()
					},
					Transform::from_xyz(enemy.position[0], enemy.position[1], 0.5),
					Enemy { enemy_type },
					EnemyMovement {
						pattern: movement_pattern,
						spawn_x: enemy.position[0],
						time_alive: 0.0,
					},
					Health::for_enemy_type(enemy_type),
					Collider::for_enemy_type(enemy_type),
				));

				// Mark this enemy as spawned
				level.spawned_enemies.insert((wave_idx, enemy_idx));

				info!(
					"Spawned {:?} at ({:.1}, {:.1}) with {:?} (legacy)",
					enemy_type, enemy.position[0], enemy.position[1], movement_pattern
				);
			}
		}

		// Only mark wave as processed if ALL enemies in it have spawned
		if all_enemies_spawned {
			level.processed_waves.push(wave_idx);
		}
	}
}

pub fn process_doodads(
	mut level: ResMut<CurrentLevel>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	use crate::components::{ScrollingBackground, ParallaxLayer, ParallaxEntity, DistanceLocked};
	use crate::level::DoodadLayer;
	use super::world::parallax;

	let current_distance = level.distance;
	let scroll_speed = level.get_scroll_speed();
	let base_threshold = scroll_speed * 10.0;  // Buffer for asset loading

	// ========== PROCESS STRUCTURES (from structures: section) ==========
	// Convert to doodads with structures/ path prefix, then process together
	let mut structures_to_process = Vec::new();
	for (idx, structure) in level.data.structures.iter().enumerate() {
		if !level.processed_structures.contains(&idx) {
			let size_buffer = structure.size.map(|[w, h]| w.max(h) * 2.0).unwrap_or(0.0);
			let distance_threshold = base_threshold + size_buffer;

			if structure.spawn_distance <= current_distance + distance_threshold
				&& structure.spawn_distance > current_distance - distance_threshold
			{
				// Resolve path: structures default to structures/
				let sprite_path = if structure.sprite.starts_with("structures/")
					|| structure.sprite.starts_with("doodads/")
					|| structure.sprite.starts_with("far/")
					|| structure.sprite.starts_with("tiles/")
				{
					structure.sprite.clone()
				} else {
					format!("structures/{}", structure.sprite)
				};

				let mut doodad = structure.to_doodad();
				doodad.sprite = sprite_path;  // Override with resolved path
				structures_to_process.push((idx, doodad));
			}
		}
	}

	// Mark structures as processed
	for (idx, _) in &structures_to_process {
		level.processed_structures.push(*idx);
	}

	// ========== PROCESS DOODADS (from doodads: section) ==========
	let mut doodads_to_process: Vec<(usize, crate::level::DoodadSpawn)> = Vec::new();
	for (doodad_idx, doodad) in level.data.doodads.iter().enumerate() {
		if !level.processed_doodads.contains(&doodad_idx) {
			let size_buffer = doodad.size.map(|[w, h]| w.max(h) * 2.0).unwrap_or(0.0);
			let distance_threshold = base_threshold + size_buffer;

			if doodad.spawn_distance <= current_distance + distance_threshold
				&& doodad.spawn_distance > current_distance - distance_threshold
			{
				// Resolve path: doodads default to doodads/
				let sprite_path = if doodad.sprite.starts_with("doodads/")
					|| doodad.sprite.starts_with("structures/")
					|| doodad.sprite.starts_with("far/")
					|| doodad.sprite.starts_with("tiles/")
					|| doodad.sprite.starts_with("backdrop/")
				{
					doodad.sprite.clone()
				} else {
					format!("doodads/{}", doodad.sprite)
				};

				let mut resolved_doodad = doodad.clone();
				resolved_doodad.sprite = sprite_path;
				doodads_to_process.push((doodad_idx, resolved_doodad));
			}
		}
	}

	// Mark doodads as processed
	for (idx, _) in &doodads_to_process {
		level.processed_doodads.push(*idx);
	}

	// ========== SPAWN ALL (structures + doodads combined) ==========
	// Track source: true = structure (always Y=800), false = doodad (can use custom Y)
	let all_to_spawn: Vec<(crate::level::DoodadSpawn, bool)> = structures_to_process.into_iter()
		.map(|(_, d)| (d, true))  // structures
		.chain(doodads_to_process.into_iter().map(|(_, d)| (d, false)))  // doodads
		.collect();

	for (doodad, is_structure) in all_to_spawn {
		let sprite_path = doodad.sprite.clone();

		// Get layer speed multiplier first (needed for spawn Y calculation)
		let speed_multiplier = match doodad.layer {
			DoodadLayer::DeepSpace => ParallaxLayer::DeepSpace.speed_multiplier(),
			DoodadLayer::FarField => ParallaxLayer::FarField.speed_multiplier(),
			DoodadLayer::DeepStructures => ParallaxLayer::DeepStructures.speed_multiplier(),
			DoodadLayer::MegaStructures => ParallaxLayer::MegaStructures.speed_multiplier(),
			DoodadLayer::MidDistance => ParallaxLayer::MidDistance.speed_multiplier(),
			DoodadLayer::StructureDetails => ParallaxLayer::StructureDetails.speed_multiplier(),
			DoodadLayer::NearBackground => ParallaxLayer::NearBackground.speed_multiplier(),
			DoodadLayer::Gameplay => 1.0,
			DoodadLayer::Foreground => ParallaxLayer::Foreground.speed_multiplier(),
		};

		// Calculate depth-based scale (farthest 2 layers stay 1.0x, others scale by speed)
		let depth_scale = if speed_multiplier <= 0.1 {
			1.0  // Farthest layers (DeepSpace, FarField) = full size backdrops
		} else {
			// Linear interpolation: 0.2x at speed=0.2 to 1.0x at speed=1.0
			0.2 + (speed_multiplier * 0.8)
		};

		// Calculate spawn Y position
		// For structures: compensate for any distance already traveled past spawn_distance
		// Use RENDERED height (source height * depth_scale) for proper positioning
		let spawn_y = if is_structure {
			let distance_past = (current_distance - doodad.spawn_distance).max(0.0);
			let source_height = doodad.size.map(|[_, h]| h).unwrap_or(0.0);
			let rendered_height = source_height * depth_scale;
			// Push up by half rendered height so bottom edge starts at 800, then apply Y offset
			800.0 + (rendered_height / 2.0) - distance_past * speed_multiplier + doodad.position.y()
		} else if doodad.spawn_distance > 1000.0 {
			800.0
		} else {
			doodad.position.y()
		};

		// Determine custom_size (if explicit size provided or auto-detected)
		let custom_size = if let Some([w, h]) = doodad.size {
			// Explicit size provided - use it directly (scaling via Transform)
			Some(Vec2::new(w, h))
		} else {
			// Try auto-detection for known sprite types
			match doodad.sprite.split('_').next().unwrap_or("") {
				"asteroid" => Some(Vec2::splat(doodad_sizes::ASTEROID)),
				"distant" => Some(Vec2::splat(doodad_sizes::DISTANT)),
				"satellite" => Some(Vec2::splat(doodad_sizes::SATELLITE)),
				"cargo" => Some(Vec2::splat(doodad_sizes::CARGO)),
				"solar" => Some(Vec2::splat(doodad_sizes::SOLAR)),
				"hull" => Some(Vec2::splat(doodad_sizes::HULL)),
				"wreckage" => Some(Vec2::splat(doodad_sizes::WRECKAGE)),
				"drone" => Some(Vec2::splat(doodad_sizes::DRONE)),
				"escape" => Some(Vec2::splat(doodad_sizes::ESCAPE)),
				"fuel" => Some(Vec2::splat(doodad_sizes::FUEL)),
				"gas" => Some(Vec2::splat(doodad_sizes::GAS)),
				"beacon" => Some(Vec2::splat(doodad_sizes::BEACON)),
				"nav" => Some(Vec2::splat(doodad_sizes::NAV)),
				"antenna" => Some(Vec2::splat(doodad_sizes::ANTENNA)),
				"trail" => Some(Vec2::splat(doodad_sizes::TRAIL)),
				"sparking" => Some(Vec2::splat(doodad_sizes::SPARKING)),
				"nebula" => Some(Vec2::splat(parallax::sizes::NEBULA_LARGE)),
				"gas_wisp" => Some(Vec2::splat(parallax::sizes::GAS_WISP)),
				"station" => Some(Vec2::splat(parallax::sizes::STATION_SILHOUETTE)),
				"planet" => Some(Vec2::splat(parallax::sizes::DISTANT_PLANET)),
				_ => None,  // Unknown - use native image size
			}
		};

		// Convert DoodadLayer to ParallaxLayer and get z-depth/speed
		let (layer_z, layer_speed) = match doodad.layer {
			DoodadLayer::DeepSpace => {
				let layer = ParallaxLayer::DeepSpace;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::FarField => {
				let layer = ParallaxLayer::FarField;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::DeepStructures => {
				let layer = ParallaxLayer::DeepStructures;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::MegaStructures => {
				let layer = ParallaxLayer::MegaStructures;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::MidDistance => {
				let layer = ParallaxLayer::MidDistance;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::StructureDetails => {
				let layer = ParallaxLayer::StructureDetails;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::NearBackground => {
				let layer = ParallaxLayer::NearBackground;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::Gameplay => (-0.5, doodad.velocity[1].abs()),
			DoodadLayer::Foreground => {
				let layer = ParallaxLayer::Foreground;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
		};

		// Calculate z-depth: explicit z_depth > z_order > auto-calculation
		let z_order_offset = doodad.z_order as f32 * 0.001;  // Each z_order step = 0.001 z-depth
		let z_depth = if let Some(explicit_z) = doodad.z_depth {
			explicit_z + z_order_offset
		} else if doodad.sprite.starts_with("isometric/") {
			// Isometric: X-based depth + z_order
			let iso_offset = doodad.position.x() * 0.001;
			layer_z + iso_offset + z_order_offset
		} else {
			// Non-isometric: layer z + z_order + small unique offset
			let unique_offset = (doodad.spawn_distance * 0.00001) % 0.05;
			layer_z + z_order_offset + unique_offset
		};
		let scroll_speed = layer_speed;

		// Static tiles (from geography) should not drift, other doodads should
		let is_tile = sprite_path.starts_with("tiles/");
		let drift_speed = if is_tile {
			0.0  // Static structures don't drift
		} else {
			0.5 + ((doodad.spawn_distance as u32 % 7) as f32 * 0.3)  // Vary drift for visual variety
		};

		let mut entity = commands.spawn((
			Sprite {
				image: asset_server.load(sprite_path.clone()),
				custom_size,  // None = use native image size
				..default()
			},
			Transform::from_xyz(doodad.position.x(), spawn_y, z_depth)
				.with_rotation(Quat::from_rotation_z(doodad.rotation.to_radians()))
				.with_scale(Vec3::splat(depth_scale)),
		));

		// STRUCTURES: use distance-locked positioning for object permanence
		// DOODADS: use time-based scrolling (transient, don't need permanence)
		if is_structure {
			entity.insert(DistanceLocked {
				spawn_distance: doodad.spawn_distance,
				base_y: 800.0,  // Horizon line where bottom edge spawns
				speed_ratio: speed_multiplier,  // How fast structure scrolls vs distance
				y_offset: doodad.position.y(),  // Vertical offset for tiling
			});
		} else {
			entity.insert(ScrollingBackground { speed: scroll_speed });
		}

		// Add drift component for gameplay layer doodads
		if doodad.layer == DoodadLayer::Gameplay {
			entity.insert(DoodadEntity {
				spawn_x: doodad.position.x(),
				drift_speed,
			});
		} else {
			// Add parallax entity marker for non-gameplay layers (used for visual effects)
			// Note: scroll_parallax requires BOTH ParallaxEntity AND ScrollingBackground,
			// so structures (which have DistanceLocked instead) won't be double-scrolled
			let parallax_layer = match doodad.layer {
				DoodadLayer::DeepSpace => ParallaxLayer::DeepSpace,
				DoodadLayer::FarField => ParallaxLayer::FarField,
				DoodadLayer::DeepStructures => ParallaxLayer::DeepStructures,
				DoodadLayer::MegaStructures => ParallaxLayer::MegaStructures,
				DoodadLayer::MidDistance => ParallaxLayer::MidDistance,
				DoodadLayer::StructureDetails => ParallaxLayer::StructureDetails,
				DoodadLayer::NearBackground => ParallaxLayer::NearBackground,
				DoodadLayer::Foreground => ParallaxLayer::Foreground,
				DoodadLayer::Gameplay => unreachable!(),
			};
			entity.insert(ParallaxEntity { layer: parallax_layer });
		}

		if let Some(size) = custom_size {
			info!(
				"Spawned {:?} doodad '{}' at ({:.1}, {:.1}) z={:.1} | custom_size=({:.0}, {:.0}) × depth_scale={:.2}",
				doodad.layer,
				doodad.sprite,
				doodad.position.x(),
				spawn_y,
				z_depth,
				size.x,
				size.y,
				depth_scale
			);
		} else {
			info!(
				"Spawned {:?} doodad '{}' at ({:.1}, {:.1}) z={:.1} | native size × depth_scale={:.2}",
				doodad.layer,
				doodad.sprite,
				doodad.position.x(),
				spawn_y,
				z_depth,
				depth_scale
			);
		}
	}
}

pub fn update_distance_locked(
	level: Res<CurrentLevel>,
	mut query: Query<(&mut Transform, &DistanceLocked, &Sprite)>,
) {
	let current_distance = level.distance;

	for (mut transform, locked, sprite) in query.iter_mut() {
		// Calculate distance traveled past spawn point
		let distance_past = (current_distance - locked.spawn_distance).max(0.0);

		// Calculate rendered height for proper centering
		let rendered_height = if let Some(size) = sprite.custom_size {
			size.y * transform.scale.y
		} else {
			0.0
		};

		// Calculate Y position based on distance, with vertical offset for tiling
		// base_y is where bottom edge spawns, so push up by half rendered height, then add offset
		let target_y = locked.base_y + (rendered_height / 2.0) - (distance_past * locked.speed_ratio) + locked.y_offset;

		// Update Y position (keep X and Z unchanged)
		transform.translation.y = target_y;
	}
}

pub fn process_level_events(
	mut level: ResMut<CurrentLevel>,
	_audio: Res<bevy_kira_audio::prelude::Audio>,
	_asset_server: Res<AssetServer>,
) {
	let current_distance = level.distance;
	let scroll_speed = level.get_scroll_speed();
	let distance_threshold = scroll_speed * 0.1;

	// Collect events to process
	let mut events_to_process = Vec::new();
	for (event_idx, event) in level.data.events.iter().enumerate() {
		if !level.processed_events.contains(&event_idx)
			&& current_distance >= event.distance
			&& current_distance < event.distance + distance_threshold
		{
			events_to_process.push((event_idx, event.clone()));
		}
	}

	// Process collected events
	for (event_idx, event) in events_to_process {
		match &event.event_type {
			crate::level::EventType::RadioChatter { message } => {
				info!("📻 [{}gu] {}", event.distance, message);
			}
			crate::level::EventType::ScreenShake {
				intensity,
				duration,
			} => {
				info!(
					"📊 [{}gu] Screen shake - intensity: {}, duration: {}s",
					event.distance, intensity, duration
				);
			}
			crate::level::EventType::BackgroundExplosion { position } => {
				info!(
					"💥 [{}gu] Explosion at ({:.1}, {:.1})",
					event.distance, position[0], position[1]
				);
			}
			crate::level::EventType::MusicChange { music } => {
				info!("🎵 [{}gu] Music change to: {}", event.distance, music);
			}
		}

		level.processed_events.push(event_idx);
	}
}

pub fn process_tutorials(level: Res<CurrentLevel>) {
	let current_distance = level.distance;
	let scroll_speed = level.get_scroll_speed();
	let distance_threshold = scroll_speed * 0.1;

	for tutorial in &level.data.tutorials {
		if (current_distance - tutorial.distance).abs() < distance_threshold {
			info!("📚 Tutorial [{}gu]: {}", tutorial.distance, tutorial.message);
		}
	}
}

pub fn process_phases(
	level: Res<CurrentLevel>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	mut music_state: ResMut<MusicState>,
	mut audio_instances: ResMut<Assets<AudioInstance>>,
	music_enabled: Res<MusicEnabled>,
) {
	// Skip music if disabled
	if !music_enabled.enabled {
		return;
	}

	if let Some(phase) = level.get_current_phase() {
		// Build the expected full path for comparison
		let base_path = if phase.music.contains('/') {
			format!("music/{}", phase.music)
		} else {
			format!("music/{}/{}", music_state.selected_genre, phase.music)
		};

		// Check if we need to change tracks (compare against any variant)
		let needs_change = music_state.current_track.as_ref()
			.map(|current| !current.starts_with(&base_path.trim_end_matches(".mp3")))
			.unwrap_or(true);

		if needs_change {
			// Crossfade: fade out current music
			if let Some(handle) = &music_state.handle {
				if let Some(instance) = audio_instances.get_mut(handle) {
					let tween = AudioTween::linear(std::time::Duration::from_secs_f32(music_state.crossfade_duration));
					instance.stop(tween);
				}
			}

			// Play new phase music from selected genre
			let base_path = if phase.music.contains('/') {
				// Music path already includes genre (e.g., "orchestral-rock/phase1_calm.mp3")
				format!("music/{}", phase.music)
			} else {
				// Legacy format - prepend selected genre
				format!("music/{}/{}", music_state.selected_genre, phase.music)
			};

			// Pick random variant if multiple exist (_2, _3, etc.)
			let final_path = MusicState::pick_track_variant(&base_path);

			let handle = audio.play(asset_server.load(&final_path))
				.looped()
				.with_volume(1.0)  // Start at full volume (fade-in removed for now)
				.handle();

			music_state.handle = Some(handle);
			music_state.current_track = Some(final_path.clone());  // Store full path
			info!("🎵 Playing: {} ({})", final_path, phase.name);
		}
	}
}

pub fn play_title_music(
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	mut title_music_state: ResMut<TitleMusicState>,
	music_enabled: Res<MusicEnabled>,
) {
	if !music_enabled.enabled {
		return;
	}

	// Pick random track from _title directory
	if let Ok(entries) = fs::read_dir("assets/music/_title") {
		let tracks: Vec<String> = entries
			.filter_map(|entry| entry.ok())
			.filter(|entry| entry.path().is_file())
			.filter_map(|entry| {
				entry.file_name().to_str().map(|s| s.to_string())
			})
			.filter(|name| name.ends_with(".mp3") || name.ends_with(".ogg"))
			.collect();

		if !tracks.is_empty() {
			let selected = &tracks[rand::random::<usize>() % tracks.len()];
			let path = format!("music/_title/{}", selected);

			let handle = audio.play(asset_server.load(&path))
				.looped()
				.with_volume(1.0)
				.handle();

			title_music_state.handle = Some(handle);
			title_music_state.current_track = Some(selected.clone());
			info!("🎵 Title music: {}", path);
		}
	}
}

pub fn stop_title_music(
	mut title_music_state: ResMut<TitleMusicState>,
	mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
	if let Some(handle) = &title_music_state.handle {
		if let Some(instance) = audio_instances.get_mut(handle) {
			let tween = AudioTween::linear(std::time::Duration::from_secs_f32(0.5));
			instance.stop(tween);
		}
		title_music_state.handle = None;
		title_music_state.current_track = None;
	}
}

#[derive(Component)]
pub struct DoodadEntity {
	pub spawn_x: f32,
	pub drift_speed: f32,
}

#[derive(Component)]
pub struct BackdropEntity;

pub fn apply_doodad_drift(
	mut query: Query<(&mut Transform, &DoodadEntity)>,
	time: Res<Time>,
) {
	for (mut transform, doodad) in query.iter_mut() {
		// Apply sine wave horizontal drift/meander
		let drift = (time.elapsed_secs() * doodad.drift_speed).sin() * 80.0;
		transform.translation.x = doodad.spawn_x + drift;
	}
}

pub fn scroll_doodads(
	mut query: Query<(&mut Transform, &crate::components::ScrollingBackground), With<DoodadEntity>>,
	time: Res<Time>,
	debug_speed: Res<DebugSpeed>,
	paused: Res<GamePaused>,
) {
	if paused.0 { return; }
	let multiplier = if debug_speed.enabled { debug_speed.multiplier } else { 1.0 };
	for (mut transform, bg) in query.iter_mut() {
		transform.translation.y -= bg.speed * multiplier * time.delta_secs();
	}
}

pub fn cleanup_doodads(
	mut commands: Commands,
	query: Query<(Entity, &Transform, &Sprite), With<DoodadEntity>>,
) {
	let base_despawn_y = -(super::world::HALF_WORLD_HEIGHT + 200.0);
	for (entity, transform, sprite) in query.iter() {
		// Account for sprite height - despawn only when entire sprite is off-screen
		let sprite_half_height = if let Some(size) = sprite.custom_size {
			// Use custom_size height scaled by transform, plus extra buffer
			(size.y * transform.scale.y) * 0.5 + 300.0
		} else {
			// Unknown size - use very conservative buffer
			1000.0
		};

		let despawn_y = base_despawn_y - sprite_half_height;

		if transform.translation.y < despawn_y {
			commands.entity(entity).despawn();
		}
	}
}
