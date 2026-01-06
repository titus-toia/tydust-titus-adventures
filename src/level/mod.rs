use serde::{Deserialize, Serialize};
use bevy::prelude::Vec2;
use crate::components::{Behavior, BehaviorType, SineAxis, TransitionType};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelData {
	pub name: String,
	pub total_distance: f32,
	#[serde(default)]
	pub phases: Vec<Phase>,
	#[serde(default)]
	pub backdrop: Vec<BackdropItem>,
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DoodadLayer {
	DeepSpace,
	DeepStructures,
	FarField,
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
	pub position: [f32; 2],
	pub velocity: [f32; 2],
	pub rotation_speed: f32,
	#[serde(default)]
	pub layer: DoodadLayer,
	#[serde(default)]
	pub size: Option<[f32; 2]>,  // Explicit [width, height] override
	#[serde(default)]
	pub z_depth: Option<f32>,   // Explicit z-depth override
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
