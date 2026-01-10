// World coordinate system constants
// All values in game units (gu) - 1000 gu = full screen height

pub const WORLD_HEIGHT: f32 = 1000.0;
#[allow(dead_code)]
pub const PLAY_WIDTH: f32 = 1333.0;  // 4:3 aspect ratio
pub const HALF_WORLD_HEIGHT: f32 = 500.0;
pub const HALF_PLAY_WIDTH: f32 = 666.5;  // 4:3 aspect ratio

pub mod sizes {
	pub const SCOUT: f32 = 60.0;
	pub const FIGHTER: f32 = 80.0;
	pub const HEAVY_GUNSHIP: f32 = 150.0;
	pub const BOSS: f32 = 300.0;
	pub const INTERCEPTOR: f32 = 50.0;
	pub const DRONE: f32 = 40.0;
	pub const BOMBER: f32 = 120.0;
	pub const CORVETTE: f32 = 180.0;
	pub const SMALL_ASTEROID: f32 = 30.0;
	pub const MEDIUM_ASTEROID: f32 = 60.0;
	pub const LARGE_ASTEROID: f32 = 120.0;
	pub const STATION_DEBRIS: f32 = 80.0;
	#[allow(dead_code)]
	pub const POWER_UP: f32 = 40.0;
}

pub mod doodad_sizes {
	pub const ASTEROID: f32 = 65.0;
	pub const DISTANT: f32 = 45.0;
	pub const SATELLITE: f32 = 60.0;
	pub const CARGO: f32 = 72.0;
	pub const SOLAR: f32 = 62.0;
	pub const HULL: f32 = 68.0;
	pub const WRECKAGE: f32 = 75.0;
	pub const DRONE: f32 = 57.0;
	pub const ESCAPE: f32 = 52.0;
	pub const FUEL: f32 = 65.0;
	pub const GAS: f32 = 50.0;
	pub const BEACON: f32 = 47.0;
	pub const NAV: f32 = 55.0;
	pub const ANTENNA: f32 = 60.0;
	pub const TRAIL: f32 = 42.0;
	pub const SPARKING: f32 = 39.0;
	pub const DEFAULT: f32 = 52.0;
}

pub mod speeds {
	#[allow(dead_code)]
	pub const BACKGROUND_SLOW: f32 = 65.0;
	#[allow(dead_code)]
	pub const BACKGROUND_FAST: f32 = 104.0;
}

pub mod player_bounds {
	use super::*;
	pub const SPAWN_Y: f32 = 0.0;  // Center of visible area
	// 4:3 playable area: X ±666.5 (1333 wide), Y ±500 (1000 tall)
	pub const MIN_Y: f32 = -500.0;  // Bottom of visible area
	pub const MAX_Y: f32 = 500.0;   // Top of visible area
	pub const MIN_X: f32 = -HALF_PLAY_WIDTH;
	pub const MAX_X: f32 = HALF_PLAY_WIDTH;
}

pub mod parallax {
	pub const BASE_SCROLL_SPEED: f32 = 100.0;

	pub mod spawn_rates {
		pub const NEAR_BACKGROUND_INTERVAL: f32 = 2.5;
		pub const FOREGROUND_INTERVAL: f32 = 0.7;
	}

	pub mod sizes {
		pub const NEBULA_LARGE: f32 = 600.0;
		#[allow(dead_code)]
		pub const NEBULA_MEDIUM: f32 = 450.0;
		pub const DISTANT_PLANET: f32 = 800.0;
		pub const STATION_SILHOUETTE: f32 = 700.0;
		pub const GAS_WISP: f32 = 250.0;
		#[allow(dead_code)]
		pub const ASTEROID_CLUSTER: f32 = 200.0;
		pub const PASSING_ROCK: f32 = 100.0;
		pub const METAL_CHUNK: f32 = 65.0;
		pub const DUST_CLOUD: f32 = 80.0;
		pub const STREAK_DUST: f32 = 45.0;
		pub const SPARK_STREAK: f32 = 40.0;
		pub const MICRO_ROCK: f32 = 25.0;
	}
}
