use serde::{Deserialize, Serialize};
use bevy::prelude::Vec2;
use crate::components::{Behavior, BehaviorType, SineAxis, TransitionType, ParallaxLayer};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelData {
	pub name: String,
	pub total_distance: f32,
	#[serde(default)]
	pub phases: Vec<Phase>,
	#[serde(default)]
	pub backdrop: Vec<BackdropItem>,
	#[serde(default)]
	pub structures: Vec<Structure>,
	#[serde(default)]
	pub structure_grids: Vec<StructureGrid>,
	#[serde(default)]
	pub geography: Vec<Geography>,
	#[serde(default)]
	pub enemy_waves: Vec<EnemyWave>,
	#[serde(default)]
	pub doodads: Vec<DoodadSpawn>,
	#[serde(default)]
	pub events: Vec<LevelEvent>,
	#[serde(default)]
	pub tutorials: Vec<Tutorial>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackdropItem {
	pub sprite: String,
	#[serde(default)]
	pub position: [f32; 2],
	#[serde(default)]
	pub size: Option<[f32; 2]>,
	#[serde(default = "default_alpha")]
	pub alpha: f32,
}

fn default_alpha() -> f32 { 1.0 }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Phase {
	pub name: String,
	pub start_distance: f32,  // Phase starts at this distance
	pub end_distance: f32,    // Phase ends at this distance
	pub music: String,
	pub scroll_speed: f32,    // GU per second during this phase
	pub background: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnemyWave {
	pub spawn_distance: f32,  // Spawn when player has traveled this far
	#[serde(default)]
	pub enemies: Vec<EnemySpawn>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnemySpawn {
	pub enemy_type: String,
	pub position: [f32; 2],

	#[serde(default)]
	pub movement: Option<String>,

	#[serde(default)]
	pub behaviors: Vec<Behavior>,

	#[serde(default)]
	pub formation_id: Option<String>,
	#[serde(default)]
	pub formation_role: Option<FormationRole>,
	#[serde(default)]
	pub formation_offset: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FormationRole {
	Leader,
	Member,
}

impl EnemySpawn {
	pub fn get_behaviors(&self) -> Vec<Behavior> {
		if !self.behaviors.is_empty() {
			return self.behaviors.clone();
		}

		if let Some(movement) = &self.movement {
			vec![self.legacy_movement_to_behavior(movement)]
		} else {
			vec![]
		}
	}

	fn legacy_movement_to_behavior(&self, movement: &str) -> Behavior {
		match movement {
			"SineWave" => Behavior {
				behavior_type: BehaviorType::MoveSineWave {
					base_velocity: Vec2::new(0.0, -100.0),
					amplitude: 100.0,
					frequency: 2.0,
					axis: SineAxis::Horizontal,
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			},
			"PassBy" => Behavior {
				behavior_type: BehaviorType::MoveStraight {
					velocity: Vec2::new(0.0, -150.0),
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			},
			"Circle" => Behavior {
				behavior_type: BehaviorType::MoveCircular {
					center_offset: Vec2::ZERO,
					radius: 80.0,
					angular_speed: 1.5,
					clockwise: true,
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			},
			"Straight" => Behavior {
				behavior_type: BehaviorType::MoveStraight {
					velocity: Vec2::new(0.0, -100.0),
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			},
			_ => Behavior {
				behavior_type: BehaviorType::MoveStraight {
					velocity: Vec2::new(0.0, -100.0),
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			},
		}
	}

	pub fn get_default_behavior_for_type(enemy_type: &str) -> Vec<Behavior> {
		match enemy_type {
			"Scout" => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -120.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"Fighter" => vec![Behavior {
				behavior_type: BehaviorType::MoveSineWave {
					base_velocity: Vec2::new(0.0, -100.0),
					amplitude: 80.0,
					frequency: 2.0,
					axis: SineAxis::Horizontal,
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"Interceptor" => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -250.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"Drone" => vec![Behavior {
				behavior_type: BehaviorType::Drift {
					velocity: Vec2::new(0.0, -80.0),
					variance: 20.0,
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"Bomber" => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -60.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"HeavyGunship" => vec![Behavior {
				behavior_type: BehaviorType::MoveCircular {
					center_offset: Vec2::ZERO,
					radius: 100.0,
					angular_speed: 1.0,
					clockwise: true,
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"Corvette" => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -90.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"SmallAsteroid" => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -200.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"MediumAsteroid" => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -150.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"LargeAsteroid" => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -100.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			"StationDebris" => vec![Behavior {
				behavior_type: BehaviorType::Drift {
					velocity: Vec2::new(0.0, -120.0),
					variance: 30.0,
				},
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
			_ => vec![Behavior {
				behavior_type: BehaviorType::MoveStraight { velocity: Vec2::new(0.0, -100.0) },
				duration: None,
				transition: TransitionType::WaitForCompletion,
			}],
		}
	}
}

/// Position can be either just X (number) or [X, Y] (array)
/// If just X, Y defaults to 800 (spawn off-screen top)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Position {
	XOnly(f32),
	XY([f32; 2]),
}

impl Position {
	pub fn x(&self) -> f32 {
		match self {
			Position::XOnly(x) => *x,
			Position::XY([x, _]) => *x,
		}
	}

	pub fn y(&self) -> f32 {
		match self {
			Position::XOnly(_) => 800.0,  // Default: spawn off-screen
			Position::XY([_, y]) => *y,
		}
	}

	pub fn to_array(&self) -> [f32; 2] {
		[self.x(), self.y()]
	}
}

impl Default for Position {
	fn default() -> Self {
		Position::XY([0.0, 800.0])
	}
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DoodadLayer {
	DeepSpace,
	FarField,
	DeepStructures,
	MegaStructures,
	MidDistance,
	StructureDetails,
	NearBackground,
	#[default]
	Gameplay,
	Foreground,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DoodadSpawn {
	pub spawn_distance: f32,  // Spawn when player has traveled this far
	pub sprite: String,
	#[serde(default)]
	pub position: Position,   // X only or [X, Y] - defaults to [0, 800]
	#[serde(default = "default_velocity")]
	pub velocity: [f32; 2],
	#[serde(default)]
	pub rotation: f32,        // Initial rotation in degrees (0, 90, 180, 270, etc.)
	#[serde(default)]
	pub rotation_speed: f32,  // Continuous rotation (radians/sec)
	#[serde(default)]
	pub layer: DoodadLayer,
	#[serde(default)]
	pub size: Option<[f32; 2]>,  // Explicit [width, height] override
	#[serde(default)]
	pub z_depth: Option<f32>,   // Explicit z-depth override
	#[serde(default)]
	pub z_order: i32,           // Relative z-order within layer (-1=behind, 0=default, 1=front)
}

fn default_velocity() -> [f32; 2] { [0.0, -100.0] }

/// Structure - Large background buildings/stations at various depths
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Structure {
	pub sprite: String,
	pub layer: DoodadLayer,
	#[serde(default)]
	pub position: Position,         // X only or [X, Y] - defaults to [0, 800]
	pub spawn_distance: f32,
	#[serde(default)]
	pub velocity: Option<[f32; 2]>,  // Defaults to layer velocity if None
	#[serde(default)]
	pub rotation: f32,
	#[serde(default)]
	pub size: Option<[f32; 2]>,
	#[serde(default)]
	pub z_depth: Option<f32>,
	#[serde(default)]
	pub z_order: i32,               // Relative z-order within layer (-1=behind, 0=default, 1=front)
}

impl Structure {
	/// Convert this structure to a DoodadSpawn for rendering
	pub fn to_doodad(&self) -> DoodadSpawn {
		// Use explicit velocity or default to scrolling down at layer speed
		let velocity = self.velocity.unwrap_or([0.0, -100.0]);

		// Prepend structures/ unless already has a complete path
		let sprite = if self.sprite.starts_with("structures/")
			|| self.sprite.starts_with("doodads/")
			|| self.sprite.starts_with("far/")
			|| self.sprite.starts_with("tiles/")
		{
			self.sprite.clone()
		} else {
			format!("structures/{}", self.sprite)
		};

		DoodadSpawn {
			spawn_distance: self.spawn_distance,
			sprite,
			position: self.position.clone(),
			velocity,
			rotation: self.rotation,
			rotation_speed: 0.0,
			layer: self.layer.clone(),
			size: self.size,
			z_depth: self.z_depth,
			z_order: self.z_order,
		}
	}
}

/// StructureGrid - Efficiently define grids of tiled structures
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructureGrid {
	pub layer: DoodadLayer,
	pub spawn_distance: f32,
	pub init_position: [f32; 2],  // Starting X, Y position
	#[serde(default)]
	pub tile_size: Option<[f32; 2]>,  // Optional: render size per tile (None = use native sprite size)
	#[serde(default)]
	pub tile_offsets: Option<[f32; 2]>,  // Optional [X, Y] offset between tiles (defaults to [64, 64] if tile_size is None)
	pub columns: usize,
	pub rows: usize,
	#[serde(default)]
	pub default_sprite: Option<String>,  // Sprite to use when tile entry is just "default"
	#[serde(default)]
	pub tiles: Vec<TileEntry>,  // Optional: if empty and default_sprite is set, auto-fills entire grid
}

impl StructureGrid {
	/// Expand this grid into individual Structure objects
	pub fn expand_to_structures(&self) -> Vec<Structure> {
		let mut structures = Vec::new();

		// Auto-fill tiles if empty but default_sprite is set
		let tiles = if self.tiles.is_empty() {
			if self.default_sprite.is_some() {
				// Fill entire grid with default tiles
				vec![TileEntry::Default; self.columns * self.rows]
			} else {
				// No tiles and no default = empty grid
				return vec![];
			}
		} else {
			self.tiles.clone()
		};

		// Calculate layer depth scale (same logic as in process_doodads)
		let speed_multiplier = match self.layer {
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

		let depth_scale = if speed_multiplier <= 0.1 {
			1.0  // Farthest layers
		} else {
			0.2 + (speed_multiplier * 0.8)
		};

		// Use tile_offsets if specified, otherwise tile_size, otherwise [64, 64]
		// Apply depth_scale to offsets to account for layer scaling
		let base_offsets = self.tile_offsets
			.or(self.tile_size)
			.unwrap_or([64.0, 64.0]);
		let offsets = [
			base_offsets[0] * depth_scale,
			base_offsets[1] * depth_scale,
		];

		for (index, tile_entry) in tiles.iter().enumerate() {
			// Skip empty tiles
			if matches!(tile_entry, TileEntry::Skip) {
				continue;
			}

			// Calculate grid position
			let col = index % self.columns;
			let row = index / self.columns;

			// Calculate world position using offsets
			let x = self.init_position[0] + (col as f32 * offsets[0]);
			let y = self.init_position[1] + (row as f32 * offsets[1]);

			// Get sprite and rotation from tile entry
			let (sprite, rotation) = match tile_entry {
				TileEntry::Skip => continue,
				TileEntry::Default => {
					if let Some(ref default) = self.default_sprite {
						(default.clone(), 0.0)
					} else {
						eprintln!("Warning: 'default' tile used but no default_sprite specified");
						continue;
					}
				},
				TileEntry::Sprite(sprite) => (sprite.clone(), 0.0),
				TileEntry::Detailed { sprite, rotation } => (sprite.clone(), rotation.unwrap_or(0.0)),
			};

			// Incremental z-ordering: tiles render in grid order (row-by-row, left-to-right)
			// This prevents z-fighting and ensures proper layering
			let z_order = index as i32;

			structures.push(Structure {
				sprite,
				layer: self.layer.clone(),
				position: Position::XY([x, y]),
				spawn_distance: self.spawn_distance,
				velocity: None,
				rotation,
				size: self.tile_size,  // None = use native sprite size
				z_depth: None,
				z_order,
			});
		}

		structures
	}
}

/// TileEntry - Individual tile in a grid
#[derive(Debug, Clone)]
pub enum TileEntry {
	Skip,  // Leave tile empty
	Default,  // Use default_sprite
	Sprite(String),  // Just sprite path
	Detailed {
		sprite: String,
		rotation: Option<f32>,
	},
}

// Custom deserialization to handle "skip", "default", sprite strings, and detailed objects
impl<'de> Deserialize<'de> for TileEntry {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use serde::de::{self, Visitor, MapAccess};
		use std::fmt;

		struct TileEntryVisitor;

		impl<'de> Visitor<'de> for TileEntryVisitor {
			type Value = TileEntry;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a tile entry (string or object)")
			}

			fn visit_str<E>(self, value: &str) -> Result<TileEntry, E>
			where
				E: de::Error,
			{
				Ok(match value {
					"skip" => TileEntry::Skip,
					"default" => TileEntry::Default,
					sprite => TileEntry::Sprite(sprite.to_string()),
				})
			}

			fn visit_map<M>(self, mut map: M) -> Result<TileEntry, M::Error>
			where
				M: MapAccess<'de>,
			{
				let mut sprite = None;
				let mut rotation = None;

				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"sprite" => sprite = Some(map.next_value()?),
						"rotation" => rotation = Some(map.next_value()?),
						_ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
					}
				}

				match sprite {
					Some(s) => Ok(TileEntry::Detailed { sprite: s, rotation }),
					None => Err(de::Error::missing_field("sprite")),
				}
			}
		}

		deserializer.deserialize_any(TileEntryVisitor)
	}
}

impl Serialize for TileEntry {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeMap;

		match self {
			TileEntry::Skip => serializer.serialize_str("skip"),
			TileEntry::Default => serializer.serialize_str("default"),
			TileEntry::Sprite(s) => serializer.serialize_str(s),
			TileEntry::Detailed { sprite, rotation } => {
				let mut map = serializer.serialize_map(Some(2))?;
				map.serialize_entry("sprite", sprite)?;
				if let Some(r) = rotation {
					map.serialize_entry("rotation", r)?;
				}
				map.end()
			}
		}
	}
}

/// Geography - structured level elements that expand into doodads
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Geography {
	#[serde(flatten)]
	pub geo_type: GeographyType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GeographyType {
	LeftWall {
		tileset: String,
		x: f32,
		from: f32,
		to: f32,
		#[serde(default)]
		tile_size: f32,  // Default 64
	},
	RightWall {
		tileset: String,
		x: f32,
		from: f32,
		to: f32,
		#[serde(default)]
		tile_size: f32,
	},
	Platform {
		tileset: String,
		x: f32,
		y: f32,
		width: u32,  // In tiles
		#[serde(default)]
		tile_size: f32,
	},
	Corridor {
		tileset: String,
		left_x: f32,
		right_x: f32,
		from: f32,
		to: f32,
		#[serde(default)]
		tile_size: f32,
	},
}

impl Default for GeographyType {
	fn default() -> Self {
		GeographyType::LeftWall {
			tileset: String::new(),
			x: 0.0,
			from: 0.0,
			to: 0.0,
			tile_size: 64.0,
		}
	}
}

impl Geography {
	/// Expand this geography element into individual doodad spawns
	pub fn expand_to_doodads(&self) -> Vec<DoodadSpawn> {
		match &self.geo_type {
			GeographyType::LeftWall { tileset, x, from, to, tile_size } => {
				expand_wall(tileset, *x, *from, *to, *tile_size, "wall_l")
			},
			GeographyType::RightWall { tileset, x, from, to, tile_size } => {
				expand_wall(tileset, *x, *from, *to, *tile_size, "wall_r")
			},
			GeographyType::Platform { tileset, x, y, width, tile_size } => {
				expand_platform(tileset, *x, *y, *width, *tile_size)
			},
			GeographyType::Corridor { tileset, left_x, right_x, from, to, tile_size } => {
				let mut doodads = expand_wall(tileset, *left_x, *from, *to, *tile_size, "wall_l");
				doodads.extend(expand_wall(tileset, *right_x, *from, *to, *tile_size, "wall_r"));
				doodads
			},
		}
	}
}

fn expand_wall(tileset: &str, x: f32, from: f32, to: f32, tile_size: f32, tile_name: &str) -> Vec<DoodadSpawn> {
	let tile_size = if tile_size > 0.0 { tile_size } else { 64.0 };
	let count = ((to - from) / tile_size).ceil() as i32;
	let sprite = format!("tiles/{}/{}.png", tileset, tile_name);

	(0..count)
		.map(|i| DoodadSpawn {
			spawn_distance: from + (i as f32 * tile_size),
			sprite: sprite.clone(),
			position: Position::XY([x, 800.0]),
			velocity: [0.0, -100.0],
			rotation: 0.0,
			rotation_speed: 0.0,
			layer: DoodadLayer::Gameplay,
			size: Some([tile_size, tile_size]),
			z_depth: Some(0.0),
				z_order: 0,
		})
		.collect()
}

fn expand_platform(tileset: &str, x: f32, y: f32, width: u32, tile_size: f32) -> Vec<DoodadSpawn> {
	let tile_size = if tile_size > 0.0 { tile_size } else { 64.0 };
	let mut doodads = Vec::new();

	// Left cap
	doodads.push(DoodadSpawn {
		spawn_distance: y,
		sprite: format!("tiles/{}/platform_l.png", tileset),
		position: Position::XY([x, 800.0]),
		velocity: [0.0, -100.0],
		rotation: 0.0,
		rotation_speed: 0.0,
		layer: DoodadLayer::Gameplay,
		size: Some([tile_size, tile_size]),
		z_depth: Some(0.0),
				z_order: 0,
	});

	// Center tiles
	for i in 1..width-1 {
		doodads.push(DoodadSpawn {
			spawn_distance: y,
			sprite: format!("tiles/{}/platform_c.png", tileset),
			position: Position::XY([x + (i as f32 * tile_size), 800.0]),
			velocity: [0.0, -100.0],
			rotation: 0.0,
			rotation_speed: 0.0,
			layer: DoodadLayer::Gameplay,
			size: Some([tile_size, tile_size]),
			z_depth: Some(0.0),
				z_order: 0,
		});
	}

	// Right cap
	doodads.push(DoodadSpawn {
		spawn_distance: y,
		sprite: format!("tiles/{}/platform_r.png", tileset),
		position: Position::XY([x + ((width - 1) as f32 * tile_size), 800.0]),
		velocity: [0.0, -100.0],
		rotation: 0.0,
		rotation_speed: 0.0,
		layer: DoodadLayer::Gameplay,
		size: Some([tile_size, tile_size]),
		z_depth: Some(0.0),
				z_order: 0,
	});

	doodads
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelEvent {
	pub distance: f32,  // Trigger at this distance
	#[serde(flatten)]
	pub event_type: EventType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum EventType {
	#[serde(rename = "RadioChatter")]
	RadioChatter { message: String },
	#[serde(rename = "ScreenShake")]
	ScreenShake { intensity: f32, duration: f32 },
	#[serde(rename = "BackgroundExplosion")]
	BackgroundExplosion { position: [f32; 2] },
	#[serde(rename = "MusicChange")]
	MusicChange { music: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tutorial {
	pub distance: f32,  // Show at this distance
	pub message: String,
	pub display_distance: f32,  // How long to show (in GU traveled)
}

// ============ GEOGRAPHY-BASED LEVEL SYSTEM (Level 2+) ============
// Philosophy: Levels are PLACES with architecture, not sprite collections.
// Structures define playable space. Objects mount on structures.

/// A Section defines the GEOGRAPHY of a portion of the level.
/// Instead of "scatter doodads here", it says "this is a canyon with walls".
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Section {
	pub name: String,
	pub start_distance: f32,
	pub end_distance: f32,
	/// The shape of playable space in this section
	#[serde(default)]
	pub shape: SectionShape,
	/// Continuous wall structures (if any)
	#[serde(default)]
	pub walls: Vec<WallDefinition>,
	/// Obstacles that cross the playable space
	#[serde(default)]
	pub obstacles: Vec<ObstacleDefinition>,
	/// Minimal floating debris (NOT the main content)
	#[serde(default)]
	pub floating: Option<FloatingDebris>,
	/// Background/parallax for this section
	#[serde(default)]
	pub backdrop: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SectionShape {
	#[default]
	Open,              // Full width, no constraints
	LeftCorridor,      // Wall on left, open right
	RightCorridor,     // Wall on right, open left
	Canyon,            // Walls on both sides, narrow center
	Split,             // Obstacle in center, two lanes
	Weave,             // Alternating obstacles
}

/// Defines a continuous wall/structure along one side
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WallDefinition {
	pub side: WallSide,
	/// Base sprite for wall segments
	pub structure_sprite: String,
	/// Base X position (negative for left, positive for right)
	pub x_position: f32,
	/// How much the wall can wobble inward/outward
	#[serde(default)]
	pub wobble: f32,
	/// Height of each wall segment in GU
	#[serde(default = "default_segment_height")]
	pub segment_height: f32,
	/// Objects mounted ON this wall
	#[serde(default)]
	pub mounted: Vec<MountedObject>,
}

fn default_segment_height() -> f32 { 300.0 }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WallSide {
	Left,
	Right,
}

/// Objects that attach TO structures (not floating)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MountedObject {
	/// What type of object
	pub object_type: MountedType,
	/// How often to place (in GU)
	pub interval: f32,
	/// Offset from wall toward center (positive = toward playable area)
	#[serde(default)]
	pub offset: f32,
	/// Random variation in placement
	#[serde(default)]
	pub jitter: f32,
}

/// Using untagged enum - serde tries each variant in order by structure
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MountedType {
	Turret(TurretWrapper),
	Light(LightWrapper),
	Vent(VentWrapper),
	Pipe(PipeWrapper),
	Debris(DebrisWrapper),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TurretWrapper { pub turret: TurretConfig }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LightWrapper { pub light: LightConfig }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VentWrapper { pub vent: VentConfig }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipeWrapper { pub pipe: PipeConfig }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DebrisWrapper { pub debris: DebrisConfig }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TurretConfig { pub sprite: String, #[serde(default)] pub damage: f32 }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LightConfig { pub sprite: String }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VentConfig { pub sprite: String }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipeConfig { pub sprite: String, #[serde(default)] pub length: f32 }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DebrisConfig { pub sprites: Vec<String> }

/// Obstacles that cross the playable space
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObstacleDefinition {
	pub obstacle_type: ObstacleType,
	/// How often obstacles appear (in GU)
	pub interval: f32,
	/// Variation in interval
	#[serde(default)]
	pub interval_jitter: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ObstacleType {
	Platform(PlatformWrapper),
	Asteroid(AsteroidWrapper),
	Conduit(ConduitWrapper),
	Barrier(BarrierWrapper),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlatformWrapper { pub platform: PlatformConfig }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsteroidWrapper { pub asteroid: AsteroidConfig }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConduitWrapper { pub conduit: ConduitConfig }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BarrierWrapper { pub barrier: BarrierConfig }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlatformConfig { pub sprite: String, #[serde(default)] pub width: f32 }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsteroidConfig { pub sprites: Vec<String>, #[serde(default)] pub size_range: [f32; 2] }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConduitConfig { pub sprite: String }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BarrierConfig { #[serde(default)] pub damage: f32 }

/// Minimal floating debris - NOT the main content
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FloatingDebris {
	/// Sprites to use
	pub sprites: Vec<String>,
	/// How many per 1000 GU (keep LOW - this isn't the focus)
	pub density: f32,
	/// Size range
	#[serde(default)]
	pub size_range: Option<[f32; 2]>,
}

// Keep old Zone struct for backwards compatibility
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Zone {
	pub name: String,
	pub start_distance: f32,
	pub end_distance: f32,
	#[serde(default)]
	pub theme: String,
	#[serde(default)]
	pub doodad_spawns: Vec<ZoneDoodadSpawn>,
	#[serde(default)]
	pub hazards: Vec<ZoneHazard>,
	#[serde(default)]
	pub structures: Vec<ZoneStructure>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZoneDoodadSpawn {
	pub pool: String,
	pub count: u32,
	#[serde(default)]
	pub distribution: Distribution,
	#[serde(default)]
	pub layer: DoodadLayer,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Distribution {
	#[default]
	Scattered,
	Clustered,
	Edges,
	Grid { spacing: f32 },
	Increasing,
	Decreasing,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZoneHazard {
	pub hazard_type: HazardType,
	#[serde(default)]
	pub count: u32,
	#[serde(default)]
	pub distribution: Distribution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardType {
	SpaceMine { damage: f32, trigger_radius: f32 },
	DamagingDebris { damage: f32, sprite: String },
	AsteroidField { density: f32 },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZoneStructure {
	pub sprite: String,
	pub side: StructureSide,
	#[serde(default)]
	pub continuous: bool,
	#[serde(default)]
	pub size: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureSide {
	Left,
	Right,
	Center,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DoodadPool {
	pub name: String,
	pub sprites: Vec<String>,
	#[serde(default)]
	pub size_range: Option<[f32; 2]>,
	#[serde(default)]
	pub velocity_range: Option<[f32; 2]>,
	#[serde(default)]
	pub rotation: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelDataV2 {
	pub name: String,
	pub total_distance: f32,
	#[serde(default)]
	pub phases: Vec<Phase>,
	#[serde(default)]
	pub backdrop: Vec<BackdropItem>,
	#[serde(default)]
	pub zones: Vec<Zone>,
	#[serde(default)]
	pub doodad_pools: Vec<DoodadPool>,
	#[serde(default)]
	pub enemy_waves: Vec<EnemyWave>,
	#[serde(default)]
	pub events: Vec<LevelEvent>,
	#[serde(default)]
	pub tutorials: Vec<Tutorial>,
}

impl LevelDataV2 {
	/// Convert zone-based level data to the standard LevelData format
	/// by generating concrete doodad spawns from zone definitions
	pub fn to_level_data(&self) -> LevelData {
		use rand::Rng;
		let mut rng = rand::thread_rng();
		let mut doodads = Vec::new();

		for zone in &self.zones {
			let zone_length = zone.end_distance - zone.start_distance;

			for spawn_def in &zone.doodad_spawns {
				// Find the pool
				let pool = self.doodad_pools.iter()
					.find(|p| p.name == spawn_def.pool);

				if let Some(pool) = pool {
					if pool.sprites.is_empty() { continue; }
					let positions = generate_positions(
						spawn_def.count,
						&spawn_def.distribution,
						zone.start_distance,
						zone.end_distance,
						&mut rng,
					);

					for (spawn_dist, x_pos) in positions {
						let sprite = &pool.sprites[rng.gen_range(0..pool.sprites.len())];
						let size = pool.size_range.and_then(|[min, max]| {
							if min > 0.0 && max >= min {
								let s = rng.gen_range(min..=max);
								Some([s, s])
							} else { None }
						});
						let vel_y = pool.velocity_range
							.map(|[min, max]| rng.gen_range(min..=max))
							.unwrap_or(-80.0);

						doodads.push(DoodadSpawn {
							spawn_distance: spawn_dist,
							sprite: sprite.clone(),
							position: Position::XY([x_pos, 800.0]),
							velocity: [0.0, vel_y],
							rotation: 0.0,
							rotation_speed: if pool.rotation { rng.gen_range(-1.0..1.0) } else { 0.0 },
							layer: spawn_def.layer.clone(),
							size,
							z_depth: None,
							z_order: 0,
						});
					}
				}
			}

			// Generate hazards
			for hazard in &zone.hazards {
				match &hazard.hazard_type {
					HazardType::SpaceMine { damage, trigger_radius } => {
						let positions = generate_positions(
							hazard.count,
							&hazard.distribution,
							zone.start_distance,
							zone.end_distance,
							&mut rng,
						);
						for (spawn_dist, x_pos) in positions {
							doodads.push(DoodadSpawn {
								spawn_distance: spawn_dist,
								sprite: "mine_1.png".to_string(),
								position: Position::XY([x_pos, 800.0]),
								velocity: [0.0, -60.0],
								rotation: 0.0,
								rotation_speed: 0.0,
								layer: DoodadLayer::Gameplay,
								size: Some([40.0, 40.0]),
								z_depth: None,
							z_order: 0,
							});
						}
					}
					HazardType::AsteroidField { density } => {
						let count = (zone_length * density / 100.0) as u32;
						let positions = generate_positions(
							count,
							&Distribution::Scattered,
							zone.start_distance,
							zone.end_distance,
							&mut rng,
						);
						for (spawn_dist, x_pos) in positions {
							let asteroid_type = rng.gen_range(1..=3);
							doodads.push(DoodadSpawn {
								spawn_distance: spawn_dist,
								sprite: format!("asteroid_{}.png", asteroid_type),
								position: Position::XY([x_pos, 800.0]),
								velocity: [rng.gen_range(-20.0..20.0), rng.gen_range(-100.0..-60.0)],
								rotation: 0.0,
								rotation_speed: rng.gen_range(-2.0..2.0),
								layer: DoodadLayer::Gameplay,
								size: None,
								z_depth: None,
							z_order: 0,
							});
						}
					}
					HazardType::DamagingDebris { damage, sprite } => {
						let positions = generate_positions(
							hazard.count,
							&hazard.distribution,
							zone.start_distance,
							zone.end_distance,
							&mut rng,
						);
						for (spawn_dist, x_pos) in positions {
							doodads.push(DoodadSpawn {
								spawn_distance: spawn_dist,
								sprite: sprite.clone(),
								position: Position::XY([x_pos, 800.0]),
								velocity: [rng.gen_range(-30.0..30.0), rng.gen_range(-120.0..-60.0)],
								rotation: 0.0,
								rotation_speed: rng.gen_range(-1.5..1.5),
								layer: DoodadLayer::Gameplay,
								size: None,
								z_depth: None,
							z_order: 0,
							});
						}
					}
				}
			}

			// Generate continuous structures
			for structure in &zone.structures {
				if structure.continuous {
					let segment_height = 400.0;
					let segments = (zone_length / segment_height) as i32;
					let x_pos = match structure.side {
						StructureSide::Left => -350.0,
						StructureSide::Right => 350.0,
						StructureSide::Center => 0.0,
					};

					for i in 0..segments {
						doodads.push(DoodadSpawn {
							spawn_distance: zone.start_distance + (i as f32 * segment_height),
							sprite: structure.sprite.clone(),
							position: Position::XY([x_pos, 800.0]),
							velocity: [0.0, -80.0],
							rotation: 0.0,
							rotation_speed: 0.0,
							layer: DoodadLayer::MegaStructures,
							size: structure.size,
							z_depth: Some(-6.0),
							z_order: 0,
						});
					}
				}
			}
		}

		// Sort doodads by spawn distance
		doodads.sort_by(|a, b| a.spawn_distance.partial_cmp(&b.spawn_distance).unwrap());

		LevelData {
			name: self.name.clone(),
			total_distance: self.total_distance,
			phases: self.phases.clone(),
			backdrop: self.backdrop.clone(),
			structures: Vec::new(),
			structure_grids: Vec::new(),
			geography: Vec::new(),
			enemy_waves: self.enemy_waves.clone(),
			doodads,
			events: self.events.clone(),
			tutorials: self.tutorials.clone(),
		}
	}
}

fn generate_positions<R: rand::Rng>(
	count: u32,
	distribution: &Distribution,
	start_dist: f32,
	end_dist: f32,
	rng: &mut R,
) -> Vec<(f32, f32)> {
	let zone_length = end_dist - start_dist;
	let screen_width = 600.0; // Playable area width

	match distribution {
		Distribution::Scattered => {
			(0..count).map(|_| {
				let dist = start_dist + rng.gen_range(0.0..zone_length);
				let x = rng.gen_range(-screen_width/2.0..screen_width/2.0);
				(dist, x)
			}).collect()
		}
		Distribution::Clustered => {
			let cluster_count = (count / 3).max(1);
			let mut positions = Vec::new();
			for _ in 0..cluster_count {
				let cluster_center_dist = start_dist + rng.gen_range(0.0..zone_length);
				let cluster_center_x = rng.gen_range(-screen_width/3.0..screen_width/3.0);
				let items_in_cluster = rng.gen_range(2..=4).min(count - positions.len() as u32);
				for _ in 0..items_in_cluster {
					let dist = cluster_center_dist + rng.gen_range(-200.0..200.0);
					let x = cluster_center_x + rng.gen_range(-80.0..80.0);
					positions.push((dist.clamp(start_dist, end_dist), x.clamp(-screen_width/2.0, screen_width/2.0)));
				}
			}
			positions
		}
		Distribution::Edges => {
			(0..count).map(|i| {
				let dist = start_dist + (zone_length * (i as f32 + 0.5) / count as f32);
				let x = if i % 2 == 0 {
					rng.gen_range(-screen_width/2.0..-screen_width/4.0)
				} else {
					rng.gen_range(screen_width/4.0..screen_width/2.0)
				};
				(dist, x)
			}).collect()
		}
		Distribution::Grid { spacing } => {
			let rows = (zone_length / spacing) as i32;
			let cols = (screen_width / spacing) as i32;
			let mut positions = Vec::new();
			for row in 0..rows {
				for col in 0..cols {
					if positions.len() >= count as usize { break; }
					let dist = start_dist + (row as f32 * spacing) + rng.gen_range(-20.0..20.0);
					let x = -screen_width/2.0 + (col as f32 * spacing) + spacing/2.0 + rng.gen_range(-10.0..10.0);
					positions.push((dist, x));
				}
			}
			positions
		}
		Distribution::Increasing => {
			(0..count).map(|i| {
				let progress = (i as f32) / (count as f32);
				let bias = progress * progress; // Quadratic bias toward end
				let dist = start_dist + zone_length * bias + rng.gen_range(0.0..zone_length * 0.1);
				let x = rng.gen_range(-screen_width/2.0..screen_width/2.0);
				(dist.min(end_dist), x)
			}).collect()
		}
		Distribution::Decreasing => {
			(0..count).map(|i| {
				let progress = (i as f32) / (count as f32);
				let bias = 1.0 - (progress * progress); // Quadratic bias toward start
				let dist = start_dist + zone_length * bias + rng.gen_range(0.0..zone_length * 0.1);
				let x = rng.gen_range(-screen_width/2.0..screen_width/2.0);
				(dist.min(end_dist), x)
			}).collect()
		}
	}
}

// ============ GEOGRAPHY-FIRST LEVEL GENERATION (V3) ============
// This generates levels with STRUCTURE - walls define corridors,
// objects mount on walls, floating debris is minimal.

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelDataV3 {
	pub name: String,
	pub total_distance: f32,
	#[serde(default)]
	pub phases: Vec<Phase>,
	#[serde(default)]
	pub backdrop: Vec<BackdropItem>,
	/// Geography-based sections (the new way)
	#[serde(default)]
	pub sections: Vec<Section>,
	#[serde(default)]
	pub enemy_waves: Vec<EnemyWave>,
	#[serde(default)]
	pub events: Vec<LevelEvent>,
	#[serde(default)]
	pub tutorials: Vec<Tutorial>,
}

impl LevelDataV3 {
	/// Convert geography-based sections to concrete doodad spawns.
	/// This is where the magic happens: structures FIRST, then mounted objects,
	/// then minimal floating debris.
	pub fn to_level_data(&self) -> LevelData {
		use rand::Rng;
		let mut rng = rand::thread_rng();
		let mut doodads = Vec::new();

		for section in &self.sections {
			// 1. GENERATE CONTINUOUS WALL STRUCTURES
			for wall in &section.walls {
				let segment_count = ((section.end_distance - section.start_distance) / wall.segment_height) as i32;

				for i in 0..segment_count {
					let base_dist = section.start_distance + (i as f32 * wall.segment_height);

					// Wall wobble - slight variation makes it feel organic
					let wobble_offset = if wall.wobble > 0.0 {
						rng.gen_range(-wall.wobble..wall.wobble)
					} else {
						0.0
					};

					let x_pos = match wall.side {
						WallSide::Left => wall.x_position - wobble_offset,
						WallSide::Right => wall.x_position + wobble_offset,
					};

					// Spawn wall segment
					doodads.push(DoodadSpawn {
						spawn_distance: base_dist,
						sprite: wall.structure_sprite.clone(),
						position: Position::XY([x_pos, 800.0]),
						velocity: [0.0, -80.0],
						rotation: 0.0,
						rotation_speed: 0.0,
						layer: DoodadLayer::MegaStructures,
						size: Some([200.0, wall.segment_height + 50.0]),
						z_depth: Some(-6.0),
						z_order: 0,
					});

					// 2. GENERATE MOUNTED OBJECTS on this wall segment
					for mounted in &wall.mounted {
						// Check if this segment should have a mounted object
						let segment_start = base_dist;
						let segment_end = base_dist + wall.segment_height;

						// Place mounted objects at their interval
						let mut mount_dist = section.start_distance + (mounted.interval / 2.0);
						while mount_dist < section.end_distance {
							if mount_dist >= segment_start && mount_dist < segment_end {
								let jitter = if mounted.jitter > 0.0 {
									rng.gen_range(-mounted.jitter..mounted.jitter)
								} else {
									0.0
								};

								// Offset toward playable area
								let mount_x = match wall.side {
									WallSide::Left => x_pos + mounted.offset,
									WallSide::Right => x_pos - mounted.offset,
								};

								let sprite_size = match &mounted.object_type {
									MountedType::Turret(w) => Some((w.turret.sprite.clone(), [50.0, 50.0])),
									MountedType::Light(w) => Some((w.light.sprite.clone(), [30.0, 30.0])),
									MountedType::Vent(w) => Some((w.vent.sprite.clone(), [60.0, 60.0])),
									MountedType::Pipe(w) => Some((w.pipe.sprite.clone(), [w.pipe.length.max(40.0), 40.0])),
									MountedType::Debris(w) if !w.debris.sprites.is_empty() => {
										let s = &w.debris.sprites[rng.gen_range(0..w.debris.sprites.len())];
										Some((s.clone(), [40.0, 40.0]))
									}
									_ => None,
								};
								let Some((sprite, size)) = sprite_size else { continue; };

								doodads.push(DoodadSpawn {
									spawn_distance: mount_dist + jitter,
									sprite,
									position: Position::XY([mount_x, 800.0]),
									velocity: [0.0, -80.0],
									rotation: 0.0,
									rotation_speed: 0.0,
									layer: DoodadLayer::StructureDetails,
									size: Some(size),
									z_depth: Some(-4.0),
									z_order: 0,
								});
							}
							mount_dist += mounted.interval;
						}
					}
				}
			}

			// 3. GENERATE OBSTACLES that cross the playable space
			for obstacle in &section.obstacles {
				let mut obs_dist = section.start_distance + obstacle.interval / 2.0;

				while obs_dist < section.end_distance {
					let jitter = if obstacle.interval_jitter > 0.0 {
						rng.gen_range(-obstacle.interval_jitter..obstacle.interval_jitter)
					} else {
						0.0
					};

					let obs_result = match &obstacle.obstacle_type {
						ObstacleType::Platform(w) => {
							let x = rng.gen_range(-100.0..100.0);
							Some((w.platform.sprite.clone(), [w.platform.width.max(60.0), 60.0], x))
						}
						ObstacleType::Asteroid(w) if !w.asteroid.sprites.is_empty() => {
							let cfg = &w.asteroid;
							let s = &cfg.sprites[rng.gen_range(0..cfg.sprites.len())];
							let sz = if cfg.size_range[0] > 0.0 && cfg.size_range[1] > 0.0 {
								rng.gen_range(cfg.size_range[0]..=cfg.size_range[1])
							} else { 50.0 };
							let x = rng.gen_range(-200.0..200.0);
							Some((s.clone(), [sz, sz], x))
						}
						ObstacleType::Conduit(w) => {
							Some((w.conduit.sprite.clone(), [400.0, 40.0], 0.0))
						}
						ObstacleType::Barrier(_) => {
							Some(("energy_barrier.png".to_string(), [300.0, 20.0], 0.0))
						}
						_ => None,
					};
					let Some((sprite, size, x_pos)) = obs_result else { continue; };

					doodads.push(DoodadSpawn {
						spawn_distance: obs_dist + jitter,
						sprite,
						position: Position::XY([x_pos, 800.0]),
						velocity: [0.0, -100.0],
						rotation: 0.0,
						rotation_speed: 0.0,
						layer: DoodadLayer::Gameplay,
						size: Some(size),
						z_depth: Some(-0.5),
						z_order: 0,
					});

					obs_dist += obstacle.interval;
				}
			}

			// 4. MINIMAL FLOATING DEBRIS (not the focus!)
			if let Some(floating) = &section.floating {
				if floating.sprites.is_empty() { continue; }
				let section_length = section.end_distance - section.start_distance;
				if section_length <= 0.0 { continue; }
				let debris_count = ((section_length / 1000.0) * floating.density) as u32;

				for _ in 0..debris_count {
					let sprite = &floating.sprites[rng.gen_range(0..floating.sprites.len())];
					let dist = section.start_distance + rng.gen_range(0.0..section_length);
					let x = rng.gen_range(-250.0..250.0);
					let size = floating.size_range
						.map(|[min, max]| {
							if min > 0.0 && max >= min {
								let s = rng.gen_range(min..=max);
								[s, s]
							} else { [40.0, 40.0] }
						})
						.unwrap_or([40.0, 40.0]);

					doodads.push(DoodadSpawn {
						spawn_distance: dist,
						sprite: sprite.clone(),
						position: Position::XY([x, 800.0]),
						velocity: [rng.gen_range(-20.0..20.0), rng.gen_range(-100.0..-60.0)],
						rotation: 0.0,
						rotation_speed: rng.gen_range(-1.0..1.0),
						layer: DoodadLayer::Gameplay,
						size: Some(size),
						z_depth: None,
							z_order: 0,
					});
				}
			}
		}

		// Sort by spawn distance
		doodads.sort_by(|a, b| a.spawn_distance.partial_cmp(&b.spawn_distance).unwrap());

		LevelData {
			name: self.name.clone(),
			total_distance: self.total_distance,
			phases: self.phases.clone(),
			backdrop: self.backdrop.clone(),
			structures: Vec::new(),
			structure_grids: Vec::new(),
			geography: Vec::new(),
			enemy_waves: self.enemy_waves.clone(),
			doodads,
			events: self.events.clone(),
			tutorials: self.tutorials.clone(),
		}
	}
}
