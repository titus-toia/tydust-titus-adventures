use bevy::prelude::*;
use rand::Rng;
use crate::components::{ParallaxLayer, ParallaxEntity, ScrollingBackground};
use super::world::{HALF_WORLD_HEIGHT, HALF_PLAY_WIDTH, parallax};
use super::level::DebugSpeed;

#[derive(Resource)]
pub struct ParallaxSpawnTimers {
	pub near_background: Timer,
	pub foreground: Timer,
}

impl Default for ParallaxSpawnTimers {
	fn default() -> Self {
		Self {
			near_background: Timer::from_seconds(parallax::spawn_rates::NEAR_BACKGROUND_INTERVAL, TimerMode::Repeating),
			foreground: Timer::from_seconds(parallax::spawn_rates::FOREGROUND_INTERVAL, TimerMode::Repeating),
		}
	}
}

pub fn init_parallax_timers(mut commands: Commands) {
	commands.insert_resource(ParallaxSpawnTimers::default());
}

pub fn spawn_procedural_parallax(
	mut commands: Commands,
	time: Res<Time>,
	debug_speed: Res<DebugSpeed>,
	mut timers: ResMut<ParallaxSpawnTimers>,
	asset_server: Res<AssetServer>,
) {
	let mut rng = rand::thread_rng();
	let spawn_y = HALF_WORLD_HEIGHT + 100.0;
	let x_range = -HALF_PLAY_WIDTH * 1.2..HALF_PLAY_WIDTH * 1.2;

	let tick_delta = if debug_speed.enabled {
		time.delta().mul_f32(debug_speed.multiplier)
	} else {
		time.delta()
	};
	timers.near_background.tick(tick_delta);
	timers.foreground.tick(tick_delta);

	if timers.near_background.just_finished() {
		spawn_near_background(&mut commands, &asset_server, &mut rng, spawn_y, x_range.clone());
	}

	if timers.foreground.just_finished() {
		spawn_foreground(&mut commands, &asset_server, &mut rng, spawn_y, x_range);
	}
}

fn spawn_near_background(
	commands: &mut Commands,
	asset_server: &Res<AssetServer>,
	rng: &mut impl Rng,
	spawn_y: f32,
	x_range: std::ops::Range<f32>,
) {
	let layer = ParallaxLayer::NearBackground;
	let x = rng.gen_range(x_range);
	let speed = parallax::BASE_SCROLL_SPEED * layer.speed_multiplier();

	let sprites = [
		("parallax/passing_rock_1.png", parallax::sizes::PASSING_ROCK),
		("parallax/passing_rock_2.png", parallax::sizes::PASSING_ROCK),
		("parallax/passing_rock_3.png", parallax::sizes::PASSING_ROCK),
		("parallax/metal_chunk_1.png", parallax::sizes::METAL_CHUNK),
		("parallax/metal_chunk_2.png", parallax::sizes::METAL_CHUNK),
		("parallax/dust_cloud_1.png", parallax::sizes::DUST_CLOUD),
	];

	let (sprite_path, size) = sprites[rng.gen_range(0..sprites.len())];
	let size_variation = rng.gen_range(0.7..1.3);

	commands.spawn((
		Sprite {
			image: asset_server.load(sprite_path),
			custom_size: Some(Vec2::splat(size * size_variation)),
			..default()
		},
		Transform::from_xyz(x, spawn_y, layer.z_depth())
			.with_rotation(Quat::from_rotation_z(rng.gen_range(0.0..std::f32::consts::TAU))),
		ScrollingBackground { speed },
		ParallaxEntity { layer },
	));
}

fn spawn_foreground(
	commands: &mut Commands,
	asset_server: &Res<AssetServer>,
	rng: &mut impl Rng,
	spawn_y: f32,
	x_range: std::ops::Range<f32>,
) {
	let layer = ParallaxLayer::Foreground;
	let x = rng.gen_range(x_range);
	let speed = parallax::BASE_SCROLL_SPEED * layer.speed_multiplier();

	let sprites = [
		("parallax/streak_dust_1.png", parallax::sizes::STREAK_DUST),
		("parallax/streak_dust_2.png", parallax::sizes::STREAK_DUST),
		("parallax/spark_streak_1.png", parallax::sizes::SPARK_STREAK),
		("parallax/micro_rock_1.png", parallax::sizes::MICRO_ROCK),
	];

	let (sprite_path, size) = sprites[rng.gen_range(0..sprites.len())];
	let size_variation = rng.gen_range(0.8..1.5);
	let alpha = rng.gen_range(0.4..0.9);

	commands.spawn((
		Sprite {
			image: asset_server.load(sprite_path),
			custom_size: Some(Vec2::splat(size * size_variation)),
			color: Color::srgba(1.0, 1.0, 1.0, alpha),
			..default()
		},
		Transform::from_xyz(x, spawn_y, layer.z_depth()),
		ScrollingBackground { speed },
		ParallaxEntity { layer },
	));
}

pub fn scroll_parallax(
	time: Res<Time>,
	debug_speed: Res<DebugSpeed>,
	mut query: Query<(&mut Transform, &ScrollingBackground), With<ParallaxEntity>>,
) {
	let multiplier = if debug_speed.enabled { debug_speed.multiplier } else { 1.0 };
	for (mut transform, bg) in query.iter_mut() {
		transform.translation.y -= bg.speed * multiplier * time.delta_secs();
	}
}

pub fn cleanup_parallax(
	mut commands: Commands,
	query: Query<(Entity, &Transform), With<ParallaxEntity>>,
) {
	let despawn_y = -(HALF_WORLD_HEIGHT + 200.0);
	for (entity, transform) in query.iter() {
		if transform.translation.y < despawn_y {
			commands.entity(entity).despawn();
		}
	}
}
