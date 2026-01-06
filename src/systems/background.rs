use bevy::prelude::*;
use crate::components::{ScrollingBackground, BackgroundTile};
use super::world::{WORLD_HEIGHT, HALF_WORLD_HEIGHT};
use super::level::DebugSpeed;

pub fn scroll_background(
	time: Res<Time>,
	debug_speed: Res<DebugSpeed>,
	mut query: Query<(&mut Transform, &ScrollingBackground), With<BackgroundTile>>,
) {
	let multiplier = if debug_speed.enabled { debug_speed.multiplier } else { 1.0 };
	for (mut transform, bg) in query.iter_mut() {
		transform.translation.y -= bg.speed * multiplier * time.delta_secs();

		// Wrap when tile goes below visible area (1000 gu tiles)
		if transform.translation.y < -(HALF_WORLD_HEIGHT + WORLD_HEIGHT / 2.0) {
			transform.translation.y += WORLD_HEIGHT * 2.0;
		}
	}
}

pub fn spawn_background(mut commands: Commands, windows: Query<&Window>) {
	let window = windows.single();
	let aspect_ratio = window.width() / window.height();

	// Calculate visible width in game units (camera shows WORLD_HEIGHT tall)
	let visible_width = WORLD_HEIGHT * aspect_ratio;
	let tile_size = Vec2::new(visible_width, WORLD_HEIGHT);

	// Spawn background tiles (5 tiles to ensure coverage during scroll)
	for i in 0..5 {
		let y_offset = (i as f32 - 2.0) * WORLD_HEIGHT;

		commands.spawn((
			Sprite {
				color: Color::srgb(0.02, 0.02, 0.05),
				custom_size: Some(tile_size),
				..default()
			},
			Transform::from_xyz(0.0, y_offset, -10.0),
			ScrollingBackground { speed: 65.0 },
			BackgroundTile,
		));
	}

	// Spawn stars across full visible area
	use rand::Rng;
	let mut rng = rand::thread_rng();

	let half_width = visible_width / 2.0;
	let vertical_range = WORLD_HEIGHT * 4.0;

	// Scale star count based on visible area
	let star_count = ((visible_width * WORLD_HEIGHT) / 1000.0) as u32;

	for _ in 0..star_count {
		let x = rng.gen_range(-half_width..half_width);
		let y = rng.gen_range(-vertical_range..vertical_range);
		let size = rng.gen_range(1.5..4.0);
		let brightness = rng.gen_range(0.5..1.0);
		let speed = rng.gen_range(26.0..104.0);

		commands.spawn((
			Sprite {
				color: Color::srgb(brightness, brightness, brightness * 0.9),
				custom_size: Some(Vec2::splat(size)),
				..default()
			},
			Transform::from_xyz(x, y, -9.0),
			ScrollingBackground { speed },
			BackgroundTile,
		));
	}

	info!("Background: {} gu wide, {} stars", visible_width as i32, star_count);
}
