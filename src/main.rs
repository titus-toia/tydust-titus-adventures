use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_kira_audio::prelude::*;
use bevy::window::{WindowMode, PresentMode};

mod components;
mod systems;
mod level;
mod resources;

use systems::background::{scroll_background, spawn_background};
use systems::player::{spawn_player, player_movement};
use systems::weapons::{fire_weapons, move_projectiles_straight, move_projectiles_sine, move_angled_projectiles, move_homing_projectiles, manage_orbital_entities, cleanup_projectiles};
use systems::level::{load_level, update_level_timer, process_enemy_waves, process_doodads, process_level_events, process_tutorials, process_phases, apply_doodad_drift, MusicState, DebugSpeed, toggle_debug_speed};
use systems::parallax::{init_parallax_timers, spawn_procedural_parallax, scroll_parallax, cleanup_parallax};
use systems::enemies::{update_enemy_movement, rotate_enemies_toward_player, cleanup_enemies, execute_enemy_behaviors, update_formations};
use systems::menu::{setup_ship_selection_menu, handle_ship_selection, handle_weapon_selection, handle_start_game, cleanup_menu};
use systems::weapon_upgrade::{handle_weapon_switch, handle_weapon_upgrade, handle_player_hit, debug_weapon_controls};
use systems::pickups::{collect_pickups, move_pickups, cleanup_pickups};
use components::{FormationRegistry, WeaponSwitchEvent, WeaponUpgradeEvent, PlayerHitEvent};
use systems::particles::{spawn_engine_particles, update_particles};
use systems::visual::apply_atmospheric_tint;
use systems::world::WORLD_HEIGHT;
use resources::{SelectedShip, SelectedWeapon, GameState};

fn main() {
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
		.init_state::<GameState>()
		.init_resource::<SelectedShip>()
		.init_resource::<SelectedWeapon>()
		.init_resource::<MusicState>()
		.init_resource::<FormationRegistry>()
		.insert_resource(DebugSpeed::new())
		.add_event::<WeaponSwitchEvent>()
		.add_event::<WeaponUpgradeEvent>()
		.add_event::<PlayerHitEvent>()
		// Startup: camera only
		.add_systems(Startup, (setup, spawn_exit_button).chain())
		// Menu state systems
		.add_systems(OnEnter(GameState::ShipSelection), setup_ship_selection_menu)
		.add_systems(
			Update,
			(handle_ship_selection, handle_weapon_selection, handle_start_game)
				.run_if(in_state(GameState::ShipSelection))
		)
		.add_systems(OnExit(GameState::ShipSelection), cleanup_menu)
		// Playing state: spawn game on enter
		.add_systems(
			OnEnter(GameState::Playing),
			(spawn_background, init_parallax_timers, spawn_player, load_level).chain()
		)
		// Playing state: all game systems
		.add_systems(Update, (
			scroll_background,
			scroll_parallax,
			spawn_procedural_parallax,
			cleanup_parallax,
			player_movement,
			toggle_debug_speed,
			exit_button_system,
			update_level_timer,
		).run_if(in_state(GameState::Playing)))
		.add_systems(Update, (
			fire_weapons,
			move_projectiles_straight,
			move_projectiles_sine,
			move_angled_projectiles,
			move_homing_projectiles,
			manage_orbital_entities,
			cleanup_projectiles,
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
			rotate_enemies_toward_player,
			cleanup_enemies,
			process_doodads,
			apply_doodad_drift,
			process_level_events,
			process_tutorials,
			spawn_engine_particles,
			update_particles,
			apply_atmospheric_tint,
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

fn spawn_exit_button(mut commands: Commands) {
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
