use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelData {
	pub name: String,
	pub total_distance: f32,  // Total level length in game units
	#[serde(default)]
	pub phases: Vec<Phase>,
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
	pub movement: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DoodadLayer {
	DeepSpace,
	FarField,
	MidDistance,
	NearBackground,
	#[default]
	Gameplay,
	Foreground,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DoodadSpawn {
	pub spawn_distance: f32,  // Spawn when player has traveled this far
	pub sprite: String,
	pub position: [f32; 2],
	pub velocity: [f32; 2],
	pub rotation_speed: f32,
	#[serde(default)]
	pub layer: DoodadLayer,
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
