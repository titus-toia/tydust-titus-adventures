use bevy::prelude::*;
use crate::components::{ShipType, WeaponType, EnemyType, CollisionShape};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Resource, Default)]
pub struct SelectedShip {
	pub ship_type: Option<ShipType>,
}

#[derive(Resource)]
pub struct SelectedWeapon {
	pub weapon_type: WeaponType,
}

impl Default for SelectedWeapon {
	fn default() -> Self {
		Self {
			weapon_type: WeaponType::BasicBlaster,
		}
	}
}

const ENEMY_MANIFEST_PATH: &str = "assets/enemies/enemy_manifest.yaml";

#[derive(Resource, Default)]
pub struct EnemyAssetRegistry {
	pub entries: HashMap<String, EnemySpriteMeta>,
}

impl EnemyAssetRegistry {
	pub fn load_from_disk() -> Self {
		let yaml = read_manifest();

		let yaml = match yaml {
			Ok(contents) => contents,
			Err(err) => {
				warn!("Enemy manifest missing ({}): {}", ENEMY_MANIFEST_PATH, err);
				if try_generate_manifest() {
					match read_manifest() {
						Ok(contents) => contents,
						Err(err) => {
							warn!("Enemy manifest still missing after generation: {}", err);
							return Self::default();
						}
					}
				} else {
					return Self::default();
				}
			}
		};

		match serde_yaml::from_str::<EnemyAssetManifest>(&yaml) {
			Ok(manifest) => {
				info!(
					"✓ Loaded enemy manifest: {} entries",
					manifest.enemies.len()
				);
				Self {
					entries: manifest.enemies,
				}
			}
			Err(err) => {
				error!("Failed to parse enemy manifest: {}", err);
				Self::default()
			}
		}
	}

	pub fn get(&self, enemy_type: EnemyType) -> Option<&EnemySpriteMeta> {
		self.entries.get(enemy_type.manifest_key())
	}
}

fn read_manifest() -> Result<String, std::io::Error> {
	if let Ok(contents) = fs::read_to_string(ENEMY_MANIFEST_PATH) {
		return Ok(contents);
	}
	let exe_dir = std::env::current_exe()
		.ok()
		.and_then(|path| path.parent().map(|p| p.to_path_buf()))
		.unwrap_or_default();
	fs::read_to_string(exe_dir.join(ENEMY_MANIFEST_PATH))
}

fn try_generate_manifest() -> bool {
	let script_path = Path::new("scripts/asset-processing/generate_enemy_manifest.py");
	let exe_dir_script = std::env::current_exe()
		.ok()
		.and_then(|path| path.parent().map(|p| p.to_path_buf()))
		.map(|dir| dir.join("scripts/asset-processing/generate_enemy_manifest.py"));

	let candidate = if script_path.exists() {
		Some(script_path.to_path_buf())
	} else {
		exe_dir_script.filter(|path| path.exists())
	};

	let Some(script) = candidate else {
		warn!("Enemy manifest script not found; skipping auto-generation");
		return false;
	};

	info!("Generating enemy manifest via {}", script.display());
	let status = Command::new("python3").arg(&script).status();
	match status {
		Ok(status) if status.success() => true,
		Ok(status) => {
			warn!("Manifest generation failed (exit {})", status);
			false
		}
		Err(err) => {
			warn!("Failed to run manifest generator: {}", err);
			false
		}
	}
}

#[derive(Deserialize)]
struct EnemyAssetManifest {
	version: u32,
	enemies: HashMap<String, EnemySpriteMeta>,
}

#[derive(Deserialize, Clone)]
pub struct EnemySpriteMeta {
	pub sprite_path: String,
	pub texture_px: [u32; 2],
	pub visual_bounds_px: [u32; 4],
	pub collision_bounds_px: [u32; 4],
	pub content_size_px: [u32; 2],
	pub content_center_offset_px: [f32; 2],
	pub collision_center_offset_px: [f32; 2],
	pub gameplay_height_gu: f32,
	#[serde(default)]
	pub fire_cooldown: Option<f32>,
	pub collision_shape: CollisionShape,
	#[serde(default = "default_collision_scale")]
	pub collision_scale: f32,
	#[serde(default)]
	pub frame_group: Option<EnemyFrameGroup>,
	#[serde(default)]
	pub sockets: Vec<EnemySocketDef>,
}

#[derive(Deserialize, Clone)]
pub struct EnemyFrameGroup {
	pub frames: Vec<String>,
	pub union_visual_bounds_px: [u32; 4],
	pub union_collision_bounds_px: [u32; 4],
}

#[derive(Deserialize, Clone)]
pub struct EnemySocketDef {
	pub id: String,
	pub offset_px: [f32; 2],
	#[serde(default)]
	pub angle_deg: Option<f32>,
	#[serde(default)]
	pub tags: Vec<String>,
}

fn default_collision_scale() -> f32 {
	1.0
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
	#[default]
	ShipSelection,
	Playing,
}

#[derive(Resource)]
pub struct BloomLevel {
	pub level: f32, // 0.0 = disabled, 0.01-1.0 = intensity
}

impl BloomLevel {
	pub fn new(percent: u32) -> Self {
		Self {
			level: (percent as f32) / 100.0,
		}
	}

	pub fn is_enabled(&self) -> bool {
		self.level > 0.0
	}
}

#[derive(Resource)]
pub struct DamageNumbersEnabled(pub bool);

impl Default for DamageNumbersEnabled {
	fn default() -> Self {
		Self(true)
	}
}
