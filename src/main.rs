use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_kira_audio::prelude::*;
use bevy::window::{WindowMode, PresentMode};
use rand::Rng;

mod components;
mod systems;
mod level;
mod resources;

use systems::background::{scroll_background, spawn_background};
use systems::player::{spawn_player, player_movement};
use systems::weapons::{fire_weapons, move_projectiles_straight, move_projectiles_sine, move_angled_projectiles, move_homing_projectiles, manage_orbital_entities, orbital_auto_fire, cleanup_projectiles};
use systems::lightning::{update_charge_meter, render_lightning_bolts, render_lightning_arcs, spawn_pending_baby_whips, cleanup_lightning_visuals, render_lightning_impacts, render_lightning_aoe};
use systems::level::{load_level, update_level_timer, process_enemy_waves, process_doodads, update_distance_locked, process_level_events, process_tutorials, process_phases, apply_doodad_drift, scroll_doodads, cleanup_doodads, MusicState, TitleMusicState, MusicEnabled, DebugSpeed, toggle_debug_speed, SelectedLevel, GamePaused, toggle_pause, InfoOverlayEnabled, toggle_info_overlay, play_title_music, stop_title_music};
use systems::parallax::{init_parallax_timers, spawn_procedural_parallax, scroll_parallax, cleanup_parallax};
use systems::enemies::{update_enemy_movement, cleanup_enemies, execute_enemy_behaviors, update_formations, setup_enemy_shooters, enemy_shooting, move_enemy_projectiles, init_enemy_rotation, rotate_enemies_to_movement, shimmer_enemies};
use systems::menu::{setup_ship_selection_menu, handle_ship_selection, handle_weapon_selection, handle_start_game, cleanup_menu};
use systems::weapon_upgrade::{handle_weapon_switch, handle_weapon_upgrade, handle_player_hit, debug_weapon_controls};
use systems::pickups::{collect_pickups, move_pickups, cleanup_pickups};
use components::{FormationRegistry, WeaponSwitchEvent, WeaponUpgradeEvent, PlayerHitEvent, EnemyHitEvent, EnemyDeathEvent, ShipType, WeaponType, ChargeMeter};
use systems::particles::{spawn_engine_particles, update_particles, spawn_explosion_particles, spawn_player_hit_particles, spawn_enemy_hit_particles};
use systems::collision::{check_projectile_enemy_collisions, apply_enemy_damage, check_player_enemy_collisions, update_invincibility, check_enemy_projectile_player_collisions, play_enemy_hit_sound, play_enemy_death_sound};
use systems::visual::{apply_atmospheric_tint, apply_ambient_occlusion};
use systems::world::WORLD_HEIGHT;
use systems::info_overlay::{spawn_info_overlay, update_info_overlay, toggle_info_overlay_visibility};
use resources::{SelectedShip, SelectedWeapon, GameState};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

fn main() {
	// Parse command-line arguments
	let args: Vec<String> = std::env::args().collect();

	// Show help and exit
	if args.iter().any(|arg| arg == "--help" || arg == "-h") {
		println!("Tydust - Space Shooter Game");
		println!();
		println!("USAGE:");
		println!("  tydust [OPTIONS]");
		println!();
		println!("OPTIONS:");
		println!("  --skip-menu, --random    Skip menu and start with random ship/weapon");
		println!("  --start=N                Start level at distance N (e.g. --start=5000)");
		println!("  --no-music               Disable music");
		println!("  --help, -h               Show this help message");
		return;
	}

	let skip_menu = args.iter().any(|arg| arg == "--skip-menu" || arg == "--random");
	let no_music = args.iter().any(|arg| arg == "--no-music");

	// Parse --start=N argument
	let start_distance: f32 = args.iter()
		.find(|arg| arg.starts_with("--start="))
		.and_then(|arg| arg.strip_prefix("--start="))
		.and_then(|val| val.parse().ok())
		.unwrap_or(0.0);

	// If skipping menu, select random ship and weapon
	let (initial_ship, initial_weapon, initial_state) = if skip_menu {
		let mut rng = rand::thread_rng();

		let ships = ShipType::all();
		let random_ship = ships[rng.gen_range(0..ships.len())];

		let weapons = [
			WeaponType::PlasmaCannon,
			WeaponType::WaveGun,
			WeaponType::SpreadShot,
			WeaponType::MissilePods,
			WeaponType::LaserArray,
			WeaponType::OrbitalDefense,
			WeaponType::LightningChain,
		];
		let random_weapon = weapons[rng.gen_range(0..weapons.len())];

		println!("🎲 Random selection: {:?} with {:?}", random_ship, random_weapon);

		(Some(random_ship), random_weapon, GameState::Playing)
	} else {
		(None, WeaponType::BasicBlaster, GameState::ShipSelection)
	};

	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "Tydust - Titus' Space Adventure".to_string(),
				mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
				present_mode: PresentMode::AutoVsync,
				..default()
			}),
			..default()
		}))
		.add_plugins(AudioPlugin)
		.add_plugins(FrameTimeDiagnosticsPlugin)
		.insert_state(initial_state)
		.insert_resource(SelectedShip { ship_type: initial_ship })
		.insert_resource(SelectedWeapon { weapon_type: initial_weapon })
		.init_resource::<MusicState>()
		.init_resource::<TitleMusicState>()
		.insert_resource(MusicEnabled::new(!no_music))
		.insert_resource(SelectedLevel::with_start_distance(3, start_distance))
		.init_resource::<FormationRegistry>()
		.insert_resource(DebugSpeed::new())
		.init_resource::<GamePaused>()
		.init_resource::<InfoOverlayEnabled>()
		.init_resource::<ChargeMeter>()
		.add_event::<WeaponSwitchEvent>()
		.add_event::<WeaponUpgradeEvent>()
		.add_event::<PlayerHitEvent>()
		.add_event::<EnemyHitEvent>()
		.add_event::<EnemyDeathEvent>()
		// Startup: camera only
		.add_systems(Startup, (setup, spawn_exit_button).chain())
		// Menu state systems
		.add_systems(OnEnter(GameState::ShipSelection), (setup_ship_selection_menu, play_title_music))
		.add_systems(
			Update,
			(handle_ship_selection, handle_weapon_selection, handle_start_game)
				.run_if(in_state(GameState::ShipSelection))
		)
		.add_systems(OnExit(GameState::ShipSelection), (cleanup_menu, stop_title_music))
		// Playing state: spawn game on enter
		.add_systems(
			OnEnter(GameState::Playing),
			(spawn_background, init_parallax_timers, spawn_player, load_level, spawn_info_overlay).chain()
		)
		// Exit button and info button work in all states
		.add_systems(Update, (exit_button_system, info_button_system))
		// Playing state: all game systems
		.add_systems(Update, (
			scroll_background,
			scroll_parallax,
			spawn_procedural_parallax,
			cleanup_parallax,
			player_movement,
			toggle_debug_speed,
			toggle_pause,
			toggle_info_overlay,
			update_level_timer,
			update_info_overlay,
			toggle_info_overlay_visibility,
		).run_if(in_state(GameState::Playing)))
		.add_systems(Update, (
			update_charge_meter,
			fire_weapons,
			move_projectiles_straight,
			move_projectiles_sine,
			move_angled_projectiles,
			move_homing_projectiles,
			manage_orbital_entities,
			orbital_auto_fire,
			cleanup_projectiles,
			spawn_pending_baby_whips,
			cleanup_lightning_visuals,
		).run_if(in_state(GameState::Playing)))
		.add_systems(Update, (
			collect_pickups,
			move_pickups,
			cleanup_pickups,
			handle_weapon_switch,
			handle_weapon_upgrade,
			handle_player_hit,
			debug_weapon_controls,
		).run_if(in_state(GameState::Playing)))
		.add_systems(Update, (
			process_phases,
			process_enemy_waves,
			update_enemy_movement,
			execute_enemy_behaviors,
			update_formations,
			init_enemy_rotation,
			rotate_enemies_to_movement,
			shimmer_enemies,
			setup_enemy_shooters,
			enemy_shooting,
			move_enemy_projectiles,
		).run_if(in_state(GameState::Playing)))
		.add_systems(Update, (
			cleanup_enemies,
			process_doodads,
			update_distance_locked,
			scroll_doodads,
			apply_doodad_drift,
			cleanup_doodads,
			process_level_events,
			process_tutorials,
			spawn_engine_particles,
			update_particles,
		).run_if(in_state(GameState::Playing)))
		// Visual effects must run AFTER process_doodads to tint newly spawned structures
		.add_systems(Update, (
			apply_atmospheric_tint,
			apply_ambient_occlusion,
		).chain().after(process_doodads).run_if(in_state(GameState::Playing)))
		// Collision systems
		.add_systems(Update, (
			check_projectile_enemy_collisions,
			apply_enemy_damage,
			play_enemy_hit_sound,
			spawn_enemy_hit_particles,
			play_enemy_death_sound,
			spawn_explosion_particles,
			check_player_enemy_collisions,
			check_enemy_projectile_player_collisions,
			spawn_player_hit_particles,
			update_invincibility,
		).chain().run_if(in_state(GameState::Playing)))
		// Lightning visual rendering
		.add_systems(Update, (
			render_lightning_bolts,
			render_lightning_impacts,
			render_lightning_aoe,
			render_lightning_arcs,
		).run_if(in_state(GameState::Playing)))
		.run();
}

fn setup(mut commands: Commands) {
	commands.spawn((
		Camera2d,
		Projection::Orthographic(OrthographicProjection {
			scaling_mode: ScalingMode::FixedVertical { viewport_height: WORLD_HEIGHT },
			..OrthographicProjection::default_2d()
		}),
	));

	info!("Tydust initialized - world height: {} gu", WORLD_HEIGHT);
}

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct InfoButton;

fn spawn_exit_button(mut commands: Commands) {
	// Exit button (X)
	commands
		.spawn((
			Node {
				position_type: PositionType::Absolute,
				right: Val::Px(10.0),
				top: Val::Px(10.0),
				width: Val::Px(40.0),
				height: Val::Px(40.0),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),  // Transparent
			Button,
			ExitButton,
		))
		.with_children(|parent| {
			parent.spawn((
				Text::new("X"),
				TextFont {
					font_size: 24.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
			));
		});

	// Info button (i)
	commands
		.spawn((
			Node {
				position_type: PositionType::Absolute,
				right: Val::Px(60.0),  // To the left of X button
				top: Val::Px(10.0),
				width: Val::Px(40.0),
				height: Val::Px(40.0),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),  // Transparent
			Button,
			InfoButton,
		))
		.with_children(|parent| {
			parent.spawn((
				Text::new("i"),
				TextFont {
					font_size: 24.0,
					..default()
				},
				TextColor(Color::srgb(0.7, 0.9, 1.0)),  // Light blue
			));
		});
}

fn exit_button_system(
	mut interaction_query: Query<
		(&Interaction, &mut BackgroundColor),
		(Changed<Interaction>, With<ExitButton>),
	>,
	mut exit: EventWriter<AppExit>,
) {
	for (interaction, mut color) in &mut interaction_query {
		match *interaction {
			Interaction::Pressed => {
				exit.send(AppExit::Success);
			}
			Interaction::Hovered => {
				*color = BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.7));
			}
			Interaction::None => {
				*color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.5));
			}
		}
	}
}

fn info_button_system(
	mut interaction_query: Query<
		(&Interaction, &mut BackgroundColor),
		(Changed<Interaction>, With<InfoButton>),
	>,
	mut info_enabled: ResMut<InfoOverlayEnabled>,
) {
	for (interaction, mut color) in &mut interaction_query {
		match *interaction {
			Interaction::Pressed => {
				info_enabled.0 = !info_enabled.0;
			}
			Interaction::Hovered => {
				*color = BackgroundColor(Color::srgba(0.2, 0.4, 0.5, 0.7));
			}
			Interaction::None => {
				*color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.5));
			}
		}
	}
}
