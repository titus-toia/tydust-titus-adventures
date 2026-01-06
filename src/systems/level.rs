use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::fs;
use crate::level::LevelData;
use super::world::{sizes, doodad_sizes};

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

impl MusicState {
	pub fn pick_random_genre() -> String {
		let genres = vec!["chiptune", "darkwave", "industrial", "orchestral-rock", "synthwave"];
		genres[rand::random::<usize>() % genres.len()].to_string()
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
	pub processed_events: Vec<usize>,
	pub processed_waves: Vec<usize>,
	pub processed_doodads: Vec<usize>,
}

impl CurrentLevel {
	pub fn new(data: LevelData) -> Self {
		Self {
			data,
			distance: 0.0,
			processed_events: Vec::new(),
			processed_waves: Vec::new(),
			processed_doodads: Vec::new(),
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

pub fn load_level(mut commands: Commands, asset_server: Res<AssetServer>) {
	let yaml_path = "assets/level-defs/level1.yaml";

	match fs::read_to_string(yaml_path) {
		Ok(yaml_str) => {
			match serde_yaml::from_str::<LevelData>(&yaml_str) {
				Ok(level) => {
					info!("✓ Loaded level: {}", level.name);

					// Spawn backdrop items (static deep space elements)
					for (i, item) in level.backdrop.iter().enumerate() {
						let sprite_path = format!("parallax/{}", item.sprite);
						let size = item.size.map(|s| Vec2::new(s[0], s[1]))
							.unwrap_or(Vec2::new(800.0, 800.0));

						commands.spawn((
							Sprite {
								image: asset_server.load(&sprite_path),
								custom_size: Some(size),
								color: Color::srgba(1.0, 1.0, 1.0, item.alpha),
								..default()
							},
							// z=-9.5: behind stars (-9.0) but in front of background tiles (-10.0)
						Transform::from_xyz(item.position[0], item.position[1], -9.5 + (i as f32 * 0.01)),
							BackdropEntity,
						));
						info!("Spawned backdrop: {} at ({}, {})", item.sprite, item.position[0], item.position[1]);
					}

					commands.insert_resource(CurrentLevel::new(level));
				}
				Err(e) => {
					error!("Failed to parse level YAML: {}", e);
				}
			}
		}
		Err(e) => {
			error!("Failed to read level file: {}", e);
		}
	}
}

pub fn update_level_timer(
	mut level: ResMut<CurrentLevel>,
	time: Res<Time>,
	debug_speed: Res<DebugSpeed>,
) {
	let mut scroll_speed = level.get_scroll_speed();
	if debug_speed.enabled {
		scroll_speed *= debug_speed.multiplier;
	}
	level.distance += scroll_speed * time.delta_secs();
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
	let distance_threshold = scroll_speed * 0.1;

	let mut waves_to_process = Vec::new();
	for (wave_idx, wave) in level.data.enemy_waves.iter().enumerate() {
		if !level.processed_waves.contains(&wave_idx)
			&& current_distance >= wave.spawn_distance
			&& current_distance < wave.spawn_distance + distance_threshold
		{
			waves_to_process.push((wave_idx, wave.clone()));
		}
	}

	for (wave_idx, wave) in waves_to_process {
		for enemy in &wave.enemies {
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
				let mut entity_commands = commands.spawn((
					Sprite {
						image: asset_server.load(sprite_path),
						custom_size: Some(Vec2::splat(size)),
						..default()
					},
					Transform::from_xyz(enemy.position[0], enemy.position[1], 0.5),
					Enemy { enemy_type },
					EnemyBehavior {
						behaviors: behaviors.clone(),
						current_index: 0,
						behavior_start_time: 0.0,
						total_time_alive: 0.0,
						spawn_position: Vec2::new(enemy.position[0], enemy.position[1]),
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

				info!(
					"Spawned {:?} at ({:.1}, {:.1}) with {} behaviors",
					enemy_type, enemy.position[0], enemy.position[1], behaviors.len()
				);
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

				info!(
					"Spawned {:?} at ({:.1}, {:.1}) with {:?} (legacy)",
					enemy_type, enemy.position[0], enemy.position[1], movement_pattern
				);
			}
		}

		level.processed_waves.push(wave_idx);
	}
}

pub fn process_doodads(
	mut level: ResMut<CurrentLevel>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	use crate::components::{ScrollingBackground, ParallaxLayer, ParallaxEntity};
	use crate::level::DoodadLayer;
	use super::world::parallax;

	let current_distance = level.distance;
	let scroll_speed = level.get_scroll_speed();
	let distance_threshold = scroll_speed * 0.1;

	// Collect doodads to process
	let mut doodads_to_process = Vec::new();
	for (doodad_idx, doodad) in level.data.doodads.iter().enumerate() {
		if !level.processed_doodads.contains(&doodad_idx)
			&& current_distance >= doodad.spawn_distance
			&& current_distance < doodad.spawn_distance + distance_threshold
		{
			doodads_to_process.push((doodad_idx, doodad.clone()));
		}
	}

	// Process collected doodads
	for (doodad_idx, doodad) in doodads_to_process {
		// Determine sprite path based on layer
		let sprite_path = match doodad.layer {
			DoodadLayer::Gameplay | DoodadLayer::FarField => format!("doodads/{}", doodad.sprite),
			DoodadLayer::MegaStructures | DoodadLayer::StructureDetails => {
				format!("structures/{}", doodad.sprite)
			}
			// DeepSpace and DeepStructures use parallax assets (nebulae, station silhouettes)
			_ => format!("parallax/{}", doodad.sprite),
		};

		// Use explicit size if provided, otherwise auto-detect from sprite prefix
		let sprite_size = if let Some([w, h]) = doodad.size {
			Vec2::new(w, h)
		} else {
			let auto_size = match doodad.sprite.split('_').next().unwrap_or("") {
				"asteroid" => doodad_sizes::ASTEROID,
				"distant" => doodad_sizes::DISTANT,
				"satellite" => doodad_sizes::SATELLITE,
				"cargo" => doodad_sizes::CARGO,
				"solar" => doodad_sizes::SOLAR,
				"hull" => doodad_sizes::HULL,
				"wreckage" => doodad_sizes::WRECKAGE,
				"drone" => doodad_sizes::DRONE,
				"escape" => doodad_sizes::ESCAPE,
				"fuel" => doodad_sizes::FUEL,
				"gas" => doodad_sizes::GAS,
				"beacon" => doodad_sizes::BEACON,
				"nav" => doodad_sizes::NAV,
				"antenna" => doodad_sizes::ANTENNA,
				"trail" => doodad_sizes::TRAIL,
				"sparking" => doodad_sizes::SPARKING,
				"nebula" => parallax::sizes::NEBULA_LARGE,
				"gas_wisp" => parallax::sizes::GAS_WISP,
				"station" => parallax::sizes::STATION_SILHOUETTE,
				"planet" => parallax::sizes::DISTANT_PLANET,
				_ => doodad_sizes::DEFAULT,
			};
			Vec2::splat(auto_size)
		};

		// Convert DoodadLayer to ParallaxLayer and get z-depth/speed
		let (layer_z, layer_speed) = match doodad.layer {
			DoodadLayer::DeepSpace => {
				let layer = ParallaxLayer::DeepSpace;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::DeepStructures => {
				let layer = ParallaxLayer::DeepStructures;
				(layer.z_depth(), parallax::BASE_SCROLL_SPEED * layer.speed_multiplier())
			}
			DoodadLayer::FarField => {
				let layer = ParallaxLayer::FarField;
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

		// Use explicit z_depth if provided, otherwise use layer default
		let z_depth = doodad.z_depth.unwrap_or(layer_z);
		let scroll_speed = layer_speed;

		// Vary drift speed based on doodad index for visual variety
		let drift_speed = 0.5 + ((doodad_idx % 7) as f32 * 0.3);

		let mut entity = commands.spawn((
			Sprite {
				image: asset_server.load(sprite_path),
				custom_size: Some(sprite_size),
				..default()
			},
			Transform::from_xyz(doodad.position[0], doodad.position[1], z_depth),
			ScrollingBackground { speed: scroll_speed },
		));

		// Only add drift component for gameplay layer doodads
		if doodad.layer == DoodadLayer::Gameplay {
			entity.insert(DoodadEntity {
				spawn_x: doodad.position[0],
				drift_speed,
			});
		} else {
			// Add parallax entity marker for non-gameplay layers
			let parallax_layer = match doodad.layer {
				DoodadLayer::DeepSpace => ParallaxLayer::DeepSpace,
				DoodadLayer::DeepStructures => ParallaxLayer::DeepStructures,
				DoodadLayer::FarField => ParallaxLayer::FarField,
				DoodadLayer::MegaStructures => ParallaxLayer::MegaStructures,
				DoodadLayer::MidDistance => ParallaxLayer::MidDistance,
				DoodadLayer::StructureDetails => ParallaxLayer::StructureDetails,
				DoodadLayer::NearBackground => ParallaxLayer::NearBackground,
				DoodadLayer::Foreground => ParallaxLayer::Foreground,
				DoodadLayer::Gameplay => unreachable!(),
			};
			entity.insert(ParallaxEntity { layer: parallax_layer });
		}

		info!(
			"Spawned {:?} doodad {} at ({:.1}, {:.1}) z={:.1}",
			doodad.layer, doodad.sprite, doodad.position[0], doodad.position[1], z_depth
		);

		level.processed_doodads.push(doodad_idx);
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
) {
	if let Some(phase) = level.get_current_phase() {
		if music_state.current_track.as_ref() != Some(&phase.music) {
			// Crossfade: fade out current music
			if let Some(handle) = &music_state.handle {
				if let Some(instance) = audio_instances.get_mut(handle) {
					let tween = AudioTween::linear(std::time::Duration::from_secs_f32(music_state.crossfade_duration));
					instance.stop(tween);
				}
			}

			// Play new phase music from selected genre
			let path = format!("music/{}/{}", music_state.selected_genre, phase.music);
			let handle = audio.play(asset_server.load(&path))
				.looped()
				.with_volume(1.0)  // Start at full volume (fade-in removed for now)
				.handle();

			music_state.handle = Some(handle);
			music_state.current_track = Some(phase.music.clone());
			info!("🎵 Playing: {}/{} ({})", music_state.selected_genre, phase.music, phase.name);
		}
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
