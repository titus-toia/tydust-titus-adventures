use bevy::prelude::*;
use bevy_kira_audio::AudioInstance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component)]
pub struct ScrollingBackground {
	pub speed: f32,
}

#[derive(Component)]
pub struct DistanceLocked {
	pub spawn_distance: f32,
	pub base_y: f32,
	pub speed_ratio: f32,
	pub y_offset: f32,  // Additional Y offset for tiling
}

#[derive(Component)]
pub struct BackgroundTile;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParallaxLayer {
	DeepSpace,
	FarField,
	DeepStructures,
	MegaStructures,
	MidDistance,
	StructureDetails,
	NearBackground,
	Foreground,
}

impl ParallaxLayer {
	pub fn z_depth(&self) -> f32 {
		match self {
			ParallaxLayer::DeepSpace => -9.5,
			ParallaxLayer::FarField => -8.0,
			ParallaxLayer::DeepStructures => -7.5,
			ParallaxLayer::MegaStructures => -7.0,
			ParallaxLayer::MidDistance => -6.0,
			ParallaxLayer::StructureDetails => -4.0,
			ParallaxLayer::NearBackground => -3.0,
			ParallaxLayer::Foreground => 2.5,
		}
	}

	pub fn speed_multiplier(&self) -> f32 {
		match self {
			ParallaxLayer::DeepSpace => 0.0,       // Static - infinitely far (nebulae, stars)
			ParallaxLayer::FarField => 0.075,      // Very distant objects
			ParallaxLayer::DeepStructures => 0.15, // Distant structures with visible movement
			ParallaxLayer::MegaStructures => 0.4,  // Large distant structures
			ParallaxLayer::MidDistance => 0.6,
			ParallaxLayer::StructureDetails => 0.8,
			ParallaxLayer::NearBackground => 1.0,  // Same speed as gameplay scroll
			ParallaxLayer::Foreground => 1.5,      // Closer than player = faster
		}
	}
}

#[derive(Component)]
#[allow(dead_code)]
pub struct ParallaxEntity {
	pub layer: ParallaxLayer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShipType {
	Wraith,
	Tempest,
	Anvil,
	Talon,
	Bastion,
}

impl ShipType {
	pub fn get_stats(&self) -> ShipStats {
		match self {
			ShipType::Wraith => ShipStats {
				speed: 620.0,
				fire_cooldown: 0.18,
				size: 84.5,
				description: "Speed-focused interceptor from Apex Dynamics",
			},
			ShipType::Tempest => ShipStats {
				speed: 520.0,
				fire_cooldown: 0.15,
				size: 84.5,
				description: "Balanced performance fighter from Vortex Dynamics",
			},
			ShipType::Anvil => ShipStats {
				speed: 450.0,
				fire_cooldown: 0.12,
				size: 88.4,
				description: "Heavy weapons platform from Forge Industrial",
			},
			ShipType::Talon => ShipStats {
				speed: 580.0,
				fire_cooldown: 0.16,
				size: 84.5,
				description: "Agile strike fighter from Helix Aerospace",
			},
			ShipType::Bastion => ShipStats {
				speed: 490.0,
				fire_cooldown: 0.14,
				size: 85.8,
				description: "Reliable combat platform from Sentinel Systems",
			},
		}
	}

	pub fn sprite_path(&self) -> &'static str {
		match self {
			ShipType::Wraith => "sprites/ships/wraith.png",
			ShipType::Tempest => "sprites/ships/tempest.png",
			ShipType::Anvil => "sprites/ships/anvil.png",
			ShipType::Talon => "sprites/ships/talon.png",
			ShipType::Bastion => "sprites/ships/bastion.png",
		}
	}

	pub fn all() -> [ShipType; 5] {
		[
			ShipType::Wraith,
			ShipType::Tempest,
			ShipType::Anvil,
			ShipType::Talon,
			ShipType::Bastion,
		]
	}
}

#[derive(Clone, Copy, Debug)]
pub struct ShipStats {
	pub speed: f32,
	pub fire_cooldown: f32,
	pub size: f32,
	pub description: &'static str,
}

#[derive(Component)]
pub struct Player {
	pub fire_cooldown: Timer,
	pub ship_type: ShipType,
}

// === Weapon System Components ===

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeaponType {
	BasicBlaster,
	PlasmaCannon,
	WaveGun,
	SpreadShot,
	MissilePods,
	LaserArray,
	OrbitalDefense,
	LightningChain,
}

pub struct WeaponConfig {
	pub base_damage: f32,
	pub damage_per_level: f32,
	pub base_cooldown: f32,
	pub cooldown_reduction_per_level: f32,
	pub projectile_speed: f32,
	pub projectile_size: Vec2,
	pub projectile_color: Color,
	pub max_level: u8,
}

impl WeaponType {
	pub fn config(&self) -> WeaponConfig {
		match self {
			WeaponType::BasicBlaster => WeaponConfig {
				base_damage: 10.0,
				damage_per_level: 0.0,
				base_cooldown: 0.25,
				cooldown_reduction_per_level: 0.0,
				projectile_speed: 1300.0,
				projectile_size: Vec2::new(5.0, 20.0),
				projectile_color: Color::srgb(0.2, 0.7, 1.0),
				max_level: 0,
			},
			WeaponType::PlasmaCannon => WeaponConfig {
				base_damage: 25.0,
				damage_per_level: 8.0,
				base_cooldown: 0.4,
				cooldown_reduction_per_level: 0.05,
				projectile_speed: 1500.0,
				projectile_size: Vec2::new(8.0, 28.0),
				projectile_color: Color::srgb(0.8, 0.2, 1.0),
				max_level: 6,
			},
			WeaponType::WaveGun => WeaponConfig {
				base_damage: 15.0,
				damage_per_level: 5.0,
				base_cooldown: 0.18,
				cooldown_reduction_per_level: 0.02,
				projectile_speed: 1100.0,
				projectile_size: Vec2::new(6.0, 22.0),
				projectile_color: Color::srgb(0.2, 1.0, 0.5),
				max_level: 6,
			},
			WeaponType::SpreadShot => WeaponConfig {
				base_damage: 12.0,
				damage_per_level: 3.0,
				base_cooldown: 0.3,
				cooldown_reduction_per_level: 0.03,
				projectile_speed: 1200.0,
				projectile_size: Vec2::new(4.0, 16.0),
				projectile_color: Color::srgb(1.0, 0.5, 0.2),
				max_level: 6,
			},
			WeaponType::MissilePods => WeaponConfig {
				base_damage: 30.0,
				damage_per_level: 10.0,
				base_cooldown: 0.6,
				cooldown_reduction_per_level: 0.06,
				projectile_speed: 800.0,
				projectile_size: Vec2::new(8.0, 20.0),
				projectile_color: Color::srgb(1.0, 0.2, 0.2),
				max_level: 6,
			},
			WeaponType::LaserArray => WeaponConfig {
				base_damage: 8.0,
				damage_per_level: 2.0,
				base_cooldown: 0.08,
				cooldown_reduction_per_level: 0.01,
				projectile_speed: 2000.0,
				projectile_size: Vec2::new(3.0, 40.0),
				projectile_color: Color::srgb(0.0, 0.8, 1.0),
				max_level: 6,
			},
			WeaponType::OrbitalDefense => WeaponConfig {
				base_damage: 15.0,
				damage_per_level: 5.0,
				base_cooldown: 0.25,
				cooldown_reduction_per_level: 0.02,
				projectile_speed: 1000.0,
				projectile_size: Vec2::new(6.0, 18.0),
				projectile_color: Color::srgb(1.0, 0.8, 0.0),
				max_level: 6,
			},
			WeaponType::LightningChain => WeaponConfig {
				base_damage: 25.0,
				damage_per_level: 2.5,
				base_cooldown: 0.4,
				cooldown_reduction_per_level: 0.01,
				projectile_speed: 1200.0,
				projectile_size: Vec2::new(30.0, 70.0),
				projectile_color: Color::srgb(0.6, 0.8, 1.0),
				max_level: 10,
			},
		}
	}
}

#[derive(Component)]
pub struct Weapon {
	pub weapon_type: WeaponType,
	pub level: u8,
	pub fire_cooldown: Timer,
}

#[derive(Component)]
pub struct Projectile {
	pub weapon_type: WeaponType,
	pub level: u8,
	pub speed: f32,
	pub damage: f32,
	pub lifetime: Timer,
}

#[derive(Component)]
pub struct SineMotion {
	pub amplitude: f32,
	pub frequency: f32,
	pub time_offset: f32,
	pub base_x: f32,
}

#[derive(Component)]
pub struct HomingProjectile {
	pub turn_speed: f32,
}

#[derive(Component)]
pub struct AngledShot {
	pub velocity: Vec2,
}

#[derive(Component)]
pub struct OrbitalEntity {
	pub angle: f32,
	pub radius: f32,
	pub rotation_speed: f32,
	pub fire_timer: Timer,
}

// === Lightning Chain Components ===

#[derive(Component)]
pub struct LightningBolt {
	pub start: Vec2,
	pub end: Vec2,
	pub curve_point: Vec2,
	pub lifetime: Timer,
	pub thickness_start: f32,
	pub thickness_end: f32,
	pub intensity: f32,
	pub is_baby: bool,
	pub recursion_depth: u8,
}

#[derive(Component)]
pub struct LightningImpact {
	pub position: Vec2,
	pub lifetime: Timer,
	pub branch_count: u8,
	pub radius: f32,
	pub intensity: f32,
}

#[derive(Component)]
pub struct LightningAoeEffect {
	pub position: Vec2,
	pub radius: f32,
	pub lifetime: Timer,
	pub intensity: f32,
}

#[derive(Component)]
pub struct PendingBabyWhip {
	pub delay_timer: Timer,
	pub spawn_from: Vec2,
	pub direction: Vec2,
	pub parent_chain_dir: Vec2, // For highway-ramp curve effect
	pub parent_damage: f32,
	pub parent_level: u8,
	pub parent_chain_range: f32,
	pub parent_aoe_radius: f32,
	pub recursion_depth: u8,
	pub baby_spawn_chance: f32,
}

#[derive(Component)]
pub struct LightningArc {
	pub start: Vec2,
	pub end: Vec2,
	pub lifetime: Timer,
	pub thickness: f32,
	pub intensity: f32,
}

#[derive(Component)]
pub struct PendingSound {
	pub delay: Timer,
	pub sound_path: &'static str,
	pub volume: f32,
	pub fade_after: Option<f32>, // Start fading after this many seconds
	pub fade_duration: f32,
}

#[derive(Component)]
pub struct FadingSound {
	pub fade_timer: Timer,
	pub instance: Handle<AudioInstance>,
}

#[derive(Component)]
pub struct WeaponPickup {
	pub weapon_type: WeaponType,
}

#[derive(Component)]
pub struct PowerUp {
	pub upgrade_amount: i8,
}

#[derive(Event)]
pub struct WeaponSwitchEvent {
	pub new_weapon: WeaponType,
}

#[derive(Event)]
pub struct WeaponUpgradeEvent {
	pub level_change: i8,
}

#[derive(Event)]
pub struct PlayerHitEvent;

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
	Interceptor,
	Drone,
	Bomber,
	Corvette,
	SmallAsteroid,
	MediumAsteroid,
	LargeAsteroid,
	StationDebris,
}

impl EnemyType {
	/// Returns (projectile_type, fire_rate) if this enemy can shoot
	pub fn shooting_config(&self) -> Option<(EnemyProjectileType, f32)> {
		match self {
			// Non-shooters
			EnemyType::Scout => None,
			EnemyType::SmallAsteroid => None,
			EnemyType::MediumAsteroid => None,
			EnemyType::LargeAsteroid => None,
			EnemyType::StationDebris => None,

			// Basic shooters
			EnemyType::Fighter => Some((EnemyProjectileType::BasicShot, 2.0)),
			EnemyType::Interceptor => Some((EnemyProjectileType::Burst, 1.8)),
			EnemyType::Drone => Some((EnemyProjectileType::Stream, 0.3)),

			// Heavy shooters
			EnemyType::Bomber => Some((EnemyProjectileType::PlasmaBall, 2.5)),
			EnemyType::Corvette => Some((EnemyProjectileType::SpreadShot, 1.5)),
			EnemyType::HeavyGunship => Some((EnemyProjectileType::SpreadShot, 1.2)),

			// Boss
			EnemyType::Boss => Some((EnemyProjectileType::Ring, 1.0)),
		}
	}
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

// === New Behavior System ===

#[derive(Component)]
pub struct EnemyBehavior {
	pub behaviors: Vec<Behavior>,
	pub current_index: usize,
	pub behavior_start_time: f32,
	pub total_time_alive: f32,
	pub spawn_position: Vec2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Behavior {
	#[serde(flatten)]
	pub behavior_type: BehaviorType,
	pub duration: Option<f32>,
	pub transition: TransitionType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum BehaviorType {
	MoveStraight { velocity: Vec2 },
	MoveSineWave { base_velocity: Vec2, amplitude: f32, frequency: f32, axis: SineAxis },
	MoveCircular { center_offset: Vec2, radius: f32, angular_speed: f32, clockwise: bool },
	MoveToPosition { target: Vec2, speed: f32, easing: EasingType },
	FollowPlayer { speed: f32, max_distance: Option<f32>, offset: Vec2 },
	FollowFormation { formation_id: String, position_index: usize, follow_speed: f32 },
	Drift { velocity: Vec2, variance: f32 },
	Accelerate { initial_velocity: Vec2, acceleration: Vec2 },
	Wait { maintain_velocity: bool },
	FacePlayer { rotation_speed: f32 },
	FaceDirection { direction: Vec2, rotation_speed: f32 },
	FaceVelocity,
	FadeOut { fade_speed: f32 },
	FadeIn { fade_speed: f32 },
	Flash { color: [f32; 4], frequency: f32 },
	Parallel { behaviors: Vec<Behavior> },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum EnemyProjectileType {
	BasicShot,    // Single aimed shot
	SpreadShot,   // 3 projectiles in a cone
	Burst,        // Rapid 3-shot burst
	PlasmaBall,   // Slow, large projectile
	Ring,         // 8 projectiles in a circle
	Stream,       // Continuous rapid fire
}

impl EnemyProjectileType {
	pub fn config(&self) -> EnemyProjectileConfig {
		match self {
			EnemyProjectileType::BasicShot => EnemyProjectileConfig {
				damage: 10.0,
				speed: 350.0,
				size: Vec2::new(8.0, 8.0),
				color: Color::srgb(1.0, 0.3, 0.3),
				count: 1,
				spread_angle: 0.0,
				burst_count: 1,
				burst_delay: 0.0,
			},
			EnemyProjectileType::SpreadShot => EnemyProjectileConfig {
				damage: 8.0,
				speed: 300.0,
				size: Vec2::new(6.0, 6.0),
				color: Color::srgb(1.0, 0.5, 0.2),
				count: 3,
				spread_angle: 0.4, // ~23 degrees total spread
				burst_count: 1,
				burst_delay: 0.0,
			},
			EnemyProjectileType::Burst => EnemyProjectileConfig {
				damage: 8.0,
				speed: 400.0,
				size: Vec2::new(5.0, 10.0),
				color: Color::srgb(1.0, 1.0, 0.3),
				count: 1,
				spread_angle: 0.0,
				burst_count: 3,
				burst_delay: 0.1,
			},
			EnemyProjectileType::PlasmaBall => EnemyProjectileConfig {
				damage: 20.0,
				speed: 180.0,
				size: Vec2::new(20.0, 20.0),
				color: Color::srgb(0.8, 0.2, 1.0),
				count: 1,
				spread_angle: 0.0,
				burst_count: 1,
				burst_delay: 0.0,
			},
			EnemyProjectileType::Ring => EnemyProjectileConfig {
				damage: 6.0,
				speed: 250.0,
				size: Vec2::new(8.0, 8.0),
				color: Color::srgb(0.3, 1.0, 1.0),
				count: 8,
				spread_angle: std::f32::consts::TAU, // Full circle
				burst_count: 1,
				burst_delay: 0.0,
			},
			EnemyProjectileType::Stream => EnemyProjectileConfig {
				damage: 5.0,
				speed: 450.0,
				size: Vec2::new(4.0, 12.0),
				color: Color::srgb(1.0, 0.8, 0.2),
				count: 1,
				spread_angle: 0.0,
				burst_count: 1,
				burst_delay: 0.0,
			},
		}
	}
}

pub struct EnemyProjectileConfig {
	pub damage: f32,
	pub speed: f32,
	pub size: Vec2,
	pub color: Color,
	pub count: u8,        // Projectiles per shot
	pub spread_angle: f32, // Angle spread for multiple projectiles
	pub burst_count: u8,  // Shots per burst
	pub burst_delay: f32, // Delay between burst shots
}

#[derive(Component)]
pub struct EnemyProjectile {
	pub damage: f32,
	pub velocity: Vec2,
	pub lifetime: Timer,
}

#[derive(Component)]
pub struct EnemyShooter {
	pub projectile_type: EnemyProjectileType,
	pub fire_timer: Timer,
	pub burst_remaining: u8,
	pub burst_timer: Timer,
}

#[derive(Component)]
pub struct EnemyPreviousPosition(pub Vec3);

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SineAxis {
	Horizontal,
	Vertical,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum EasingType {
	Linear,
	EaseIn,
	EaseOut,
	EaseInOut,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TransitionType {
	Immediate,
	WaitForCompletion,
}

// === Formation System ===

#[derive(Component)]
pub struct FormationLeader {
	pub formation_id: String,
	pub member_offsets: Vec<Vec2>,
}

#[derive(Component)]
pub struct FormationMember {
	pub formation_id: String,
	pub leader: Entity,
	pub offset: Vec2,
}

#[derive(Resource, Default)]
pub struct FormationRegistry {
	pub formations: HashMap<String, Entity>,
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

// === Collision & Health Components ===

#[derive(Component)]
pub struct Health {
	pub current: f32,
	pub max: f32,
}

impl Health {
	pub fn new(max: f32) -> Self {
		Self { current: max, max }
	}

	pub fn for_enemy_type(enemy_type: EnemyType) -> Self {
		let max = match enemy_type {
			EnemyType::Scout => 10.0,
			EnemyType::Fighter => 25.0,
			EnemyType::Interceptor => 15.0,
			EnemyType::Drone => 8.0,
			EnemyType::Bomber => 40.0,
			EnemyType::Corvette => 60.0,
			EnemyType::HeavyGunship => 100.0,
			EnemyType::Boss => 500.0,
			EnemyType::SmallAsteroid => 5.0,
			EnemyType::MediumAsteroid => 15.0,
			EnemyType::LargeAsteroid => 225.0,
			EnemyType::StationDebris => 20.0,
		};
		Self::new(max)
	}
}

#[derive(Component)]
pub struct PlayerHealth {
	pub current: f32,
	pub max: f32,
}

impl Default for PlayerHealth {
	fn default() -> Self {
		Self { current: 100.0, max: 100.0 }
	}
}

#[derive(Component)]
pub struct Invincible {
	pub timer: Timer,
}

impl Invincible {
	pub fn new(duration: f32) -> Self {
		Self {
			timer: Timer::from_seconds(duration, TimerMode::Once),
		}
	}
}

#[derive(Component)]
pub struct Collider {
	pub radius: f32,
}

impl Collider {
	pub fn new(radius: f32) -> Self {
		Self { radius }
	}

	pub fn for_enemy_type(enemy_type: EnemyType) -> Self {
		let radius = match enemy_type {
			EnemyType::Scout => 30.0,
			EnemyType::Fighter => 40.0,
			EnemyType::Interceptor => 25.0,
			EnemyType::Drone => 20.0,
			EnemyType::Bomber => 45.0,
			EnemyType::Corvette => 50.0,
			EnemyType::HeavyGunship => 60.0,
			EnemyType::Boss => 150.0,
			EnemyType::SmallAsteroid => 20.0,
			EnemyType::MediumAsteroid => 35.0,
			EnemyType::LargeAsteroid => 50.0,
			EnemyType::StationDebris => 40.0,
		};
		Self::new(radius)
	}
}

#[derive(Clone, Copy, Debug)]
pub struct ContactDamage;

impl ContactDamage {
	pub fn for_enemy_type(enemy_type: EnemyType) -> f32 {
		match enemy_type {
			EnemyType::Scout => 10.0,
			EnemyType::Fighter => 15.0,
			EnemyType::Interceptor => 20.0,
			EnemyType::Drone => 5.0,
			EnemyType::Bomber => 25.0,
			EnemyType::Corvette => 20.0,
			EnemyType::HeavyGunship => 25.0,
			EnemyType::Boss => 30.0,
			EnemyType::SmallAsteroid => 15.0,
			EnemyType::MediumAsteroid => 15.0,
			EnemyType::LargeAsteroid => 15.0,
			EnemyType::StationDebris => 10.0,
		}
	}
}

#[derive(Event)]
pub struct EnemyHitEvent {
	pub enemy: Entity,
	pub damage: f32,
	pub hit_sound: Option<&'static str>, // Optional custom hit sound (None = default)
}

#[derive(Event)]
pub struct EnemyDeathEvent {
	pub position: Vec2,
	pub enemy_type: EnemyType,
}

#[derive(Resource)]
pub struct ChargeMeter {
	pub current: f32,
	pub max: f32,
	pub recharge_rate: f32,
	pub is_charging: bool,
	pub charge_consumed_this_frame: bool,
}

impl Default for ChargeMeter {
	fn default() -> Self {
		Self {
			current: 2.0,
			max: 2.0,
			recharge_rate: 0.5,
			is_charging: false,
			charge_consumed_this_frame: false,
		}
	}
}
