use bevy::prelude::*;

#[derive(Component)]
pub struct ScrollingBackground {
	pub speed: f32,
}

#[derive(Component)]
pub struct BackgroundTile;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParallaxLayer {
	DeepSpace,
	FarField,
	MidDistance,
	NearBackground,
	Foreground,
}

impl ParallaxLayer {
	pub fn z_depth(&self) -> f32 {
		match self {
			ParallaxLayer::DeepSpace => -10.0,
			ParallaxLayer::FarField => -9.0,
			ParallaxLayer::MidDistance => -6.0,
			ParallaxLayer::NearBackground => -3.0,
			ParallaxLayer::Foreground => 2.5,
		}
	}

	pub fn speed_multiplier(&self) -> f32 {
		match self {
			ParallaxLayer::DeepSpace => 0.15,
			ParallaxLayer::FarField => 0.4,
			ParallaxLayer::MidDistance => 0.75,
			ParallaxLayer::NearBackground => 1.3,
			ParallaxLayer::Foreground => 3.0,
		}
	}
}

#[derive(Component)]
#[allow(dead_code)]
pub struct ParallaxEntity {
	pub layer: ParallaxLayer,
}

#[derive(Component)]
pub struct Player {
	pub fire_cooldown: Timer,
}

#[derive(Component)]
pub struct Laser {
	pub speed: f32,
}

// === Enemy Components ===

#[derive(Component)]
#[allow(dead_code)]
pub struct Enemy {
	pub enemy_type: EnemyType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyType {
	Scout,
	Fighter,
	HeavyGunship,
	Boss,
}

#[derive(Component)]
pub struct EnemyMovement {
	pub pattern: MovementPattern,
	pub spawn_x: f32,
	pub time_alive: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum MovementPattern {
	SineWave { amplitude: f32, frequency: f32 },
	PassBy { speed: f32 },
	Circle { radius: f32, angular_speed: f32 },
	Straight { speed: f32 },
}

// === Particle Components ===

#[derive(Component)]
pub struct Particle {
	pub lifetime: Timer,
	pub velocity: Vec2,
}

#[derive(Component)]
pub struct ParticleEmitter {
	pub spawn_timer: Timer,
	pub offset: Vec2,
}
