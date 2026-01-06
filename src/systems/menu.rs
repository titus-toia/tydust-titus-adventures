use bevy::prelude::*;
use crate::components::{ShipType, WeaponType};
use crate::resources::{SelectedShip, SelectedWeapon, GameState};

#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub struct ShipButton {
	pub ship_type: ShipType,
}

#[derive(Component)]
pub struct WeaponButton {
	pub weapon_type: WeaponType,
}

#[derive(Component)]
pub struct StartGameButton;

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 280.0;
const BUTTON_SPACING: f32 = 20.0;
const SHIP_PREVIEW_SIZE: f32 = 150.0;

pub fn setup_ship_selection_menu(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	// Root container
	commands
		.spawn((
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				flex_direction: FlexDirection::Column,
				..default()
			},
			BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
			MenuUI,
		))
		.with_children(|parent| {
			// Title
			parent.spawn((
				Text::new("SELECT YOUR FIGHTER"),
				TextFont {
					font_size: 48.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					margin: UiRect::all(Val::Px(40.0)),
					..default()
				},
			));

			// Ship selection container (horizontal row)
			parent
				.spawn(Node {
					width: Val::Auto,
					height: Val::Auto,
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					column_gap: Val::Px(BUTTON_SPACING),
					..default()
				})
				.with_children(|row| {
					for ship_type in ShipType::all() {
						spawn_ship_button(row, ship_type, &asset_server);
					}
				});

			// Weapon selection title
			parent.spawn((
				Text::new("SELECT YOUR WEAPON"),
				TextFont {
					font_size: 32.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					margin: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(40.0), Val::Px(20.0)),
					..default()
				},
			));

			// Weapon selection container (top row)
			parent
				.spawn(Node {
					width: Val::Auto,
					height: Val::Auto,
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					column_gap: Val::Px(BUTTON_SPACING),
					..default()
				})
				.with_children(|row| {
					spawn_weapon_button(row, WeaponType::BasicBlaster, &asset_server);
					spawn_weapon_button(row, WeaponType::PlasmaCannon, &asset_server);
					spawn_weapon_button(row, WeaponType::WaveGun, &asset_server);
					spawn_weapon_button(row, WeaponType::SpreadShot, &asset_server);
				});

			// Weapon selection container (bottom row)
			parent
				.spawn(Node {
					width: Val::Auto,
					height: Val::Auto,
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					column_gap: Val::Px(BUTTON_SPACING),
					margin: UiRect::top(Val::Px(10.0)),
					..default()
				})
				.with_children(|row| {
					spawn_weapon_button(row, WeaponType::MissilePods, &asset_server);
					spawn_weapon_button(row, WeaponType::LaserArray, &asset_server);
					spawn_weapon_button(row, WeaponType::OrbitalDefense, &asset_server);
				});

			// Start button (initially hidden until selection made)
			parent.spawn((
				Node {
					width: Val::Px(300.0),
					height: Val::Px(60.0),
					margin: UiRect::all(Val::Px(40.0)),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					display: Display::None,
					..default()
				},
				BackgroundColor(Color::srgba(0.2, 0.6, 0.3, 0.8)),
				Button,
				StartGameButton,
			))
			.with_children(|button| {
				button.spawn((
					Text::new("START MISSION"),
					TextFont {
						font_size: 24.0,
						..default()
					},
					TextColor(Color::srgb(1.0, 1.0, 1.0)),
				));
			});
		});
}

fn spawn_ship_button(
	parent: &mut ChildBuilder,
	ship_type: ShipType,
	asset_server: &Res<AssetServer>,
) {
	let stats = ship_type.get_stats();
	let ship_name = format!("{:?}", ship_type);

	parent
		.spawn((
			Node {
				width: Val::Px(BUTTON_WIDTH),
				height: Val::Px(BUTTON_HEIGHT),
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				padding: UiRect::all(Val::Px(10.0)),
				border: UiRect::all(Val::Px(2.0)),
				..default()
			},
			BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
			BorderColor(Color::srgb(0.3, 0.3, 0.4)),
			Button,
			ShipButton { ship_type },
		))
		.with_children(|button| {
			// Ship name
			button.spawn((
				Text::new(ship_name.to_uppercase()),
				TextFont {
					font_size: 18.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					margin: UiRect::bottom(Val::Px(10.0)),
					..default()
				},
			));

			// Ship preview image
			button.spawn((
				ImageNode {
					image: asset_server.load(ship_type.sprite_path()),
					..default()
				},
				Node {
					width: Val::Px(SHIP_PREVIEW_SIZE),
					height: Val::Px(SHIP_PREVIEW_SIZE),
					margin: UiRect::all(Val::Px(5.0)),
					..default()
				},
			));

			// Stats display
			let stats_text = format!(
				"Speed: {:.0}\nFire Rate: {:.2}s\nSize: {:.0}\n\n{}",
				stats.speed, stats.fire_cooldown, stats.size, stats.description
			);
			button.spawn((
				Text::new(stats_text),
				TextFont {
					font_size: 12.0,
					..default()
				},
				TextColor(Color::srgb(0.7, 0.7, 0.8)),
				TextLayout::new_with_justify(JustifyText::Center),
				Node {
					margin: UiRect::top(Val::Px(10.0)),
					..default()
				},
			));
		});
}

fn spawn_weapon_button(
	parent: &mut ChildBuilder,
	weapon_type: WeaponType,
	asset_server: &Res<AssetServer>,
) {
	let config = weapon_type.config();
	let weapon_name = format!("{:?}", weapon_type);
	let (description, levels, icon_path) = match weapon_type {
		WeaponType::BasicBlaster => ("Unupgradeable default weapon", "None", "sprites/weapons/basic_blaster.png"),
		WeaponType::PlasmaCannon => ("Focused power, high damage", "1-6", "sprites/weapons/plasma_cannon.png"),
		WeaponType::WaveGun => ("Sine wave pattern, wide coverage", "1-6", "sprites/weapons/wave_gun.png"),
		WeaponType::SpreadShot => ("Multi-directional fan, crowd control", "1-6", "sprites/weapons/spread_shot.png"),
		WeaponType::MissilePods => ("Homing missiles, auto-targeting", "1-6", "sprites/weapons/missile_pods.png"),
		WeaponType::LaserArray => ("Rapid-fire beams, continuous DPS", "1-6", "sprites/weapons/laser_array.png"),
		WeaponType::OrbitalDefense => ("Rotating orbs, offense + defense", "1-6", "sprites/weapons/orbital_defense.png"),
	};

	parent
		.spawn((
			Node {
				width: Val::Px(BUTTON_WIDTH),
				height: Val::Px(BUTTON_HEIGHT),
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				padding: UiRect::all(Val::Px(10.0)),
				border: UiRect::all(Val::Px(2.0)),
				..default()
			},
			BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
			BorderColor(Color::srgb(0.3, 0.3, 0.4)),
			Button,
			WeaponButton { weapon_type },
		))
		.with_children(|button| {
			// Weapon name
			button.spawn((
				Text::new(weapon_name.to_uppercase()),
				TextFont {
					font_size: 18.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					margin: UiRect::bottom(Val::Px(10.0)),
					..default()
				},
			));

			// Weapon icon preview
			button.spawn((
				ImageNode {
					image: asset_server.load(icon_path),
					..default()
				},
				Node {
					width: Val::Px(120.0),
					height: Val::Px(120.0),
					margin: UiRect::all(Val::Px(5.0)),
					..default()
				},
			));

			// Stats display
			let stats_text = format!(
				"Damage: {:.0}\nSpeed: {:.0}\nLevels: {}\n\n{}",
				config.base_damage, config.projectile_speed, levels, description
			);
			button.spawn((
				Text::new(stats_text),
				TextFont {
					font_size: 12.0,
					..default()
				},
				TextColor(Color::srgb(0.7, 0.7, 0.8)),
				TextLayout::new_with_justify(JustifyText::Center),
				Node {
					margin: UiRect::top(Val::Px(10.0)),
					..default()
				},
			));
		});
}

pub fn handle_weapon_selection(
	mut interaction_query: Query<
		(&Interaction, &WeaponButton, &mut BackgroundColor, &mut BorderColor),
		Changed<Interaction>,
	>,
	mut selected_weapon: ResMut<SelectedWeapon>,
) {
	for (interaction, weapon_button, mut bg_color, mut border_color) in &mut interaction_query {
		match *interaction {
			Interaction::Pressed => {
				selected_weapon.weapon_type = weapon_button.weapon_type;
				info!("Selected weapon: {:?}", weapon_button.weapon_type);

				// Highlight selected
				*bg_color = BackgroundColor(Color::srgba(0.3, 0.4, 0.6, 0.9));
				*border_color = BorderColor(Color::srgb(0.5, 0.7, 1.0));
			}
			Interaction::Hovered => {
				*bg_color = BackgroundColor(Color::srgba(0.2, 0.25, 0.35, 0.9));
			}
			Interaction::None => {
				// Reset if not selected
				if selected_weapon.weapon_type != weapon_button.weapon_type {
					*bg_color = BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9));
					*border_color = BorderColor(Color::srgb(0.3, 0.3, 0.4));
				}
			}
		}
	}
}

pub fn handle_ship_selection(
	mut interaction_query: Query<
		(&Interaction, &ShipButton, &mut BackgroundColor, &mut BorderColor),
		Changed<Interaction>,
	>,
	mut selected_ship: ResMut<SelectedShip>,
	mut start_button_query: Query<&mut Node, With<StartGameButton>>,
) {
	for (interaction, ship_button, mut bg_color, mut border_color) in &mut interaction_query {
		match *interaction {
			Interaction::Pressed => {
				selected_ship.ship_type = Some(ship_button.ship_type);
				info!("Selected ship: {:?}", ship_button.ship_type);

				// Show start button
				if let Ok(mut node) = start_button_query.get_single_mut() {
					node.display = Display::Flex;
				}

				// Highlight selected
				*bg_color = BackgroundColor(Color::srgba(0.3, 0.4, 0.6, 0.9));
				*border_color = BorderColor(Color::srgb(0.5, 0.7, 1.0));
			}
			Interaction::Hovered => {
				*bg_color = BackgroundColor(Color::srgba(0.2, 0.25, 0.35, 0.9));
			}
			Interaction::None => {
				// Reset if not selected
				if selected_ship.ship_type != Some(ship_button.ship_type) {
					*bg_color = BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9));
					*border_color = BorderColor(Color::srgb(0.3, 0.3, 0.4));
				}
			}
		}
	}
}

pub fn handle_start_game(
	interaction_query: Query<
		&Interaction,
		(Changed<Interaction>, With<StartGameButton>),
	>,
	selected_ship: Res<SelectedShip>,
	mut next_state: ResMut<NextState<GameState>>,
) {
	for interaction in &interaction_query {
		if *interaction == Interaction::Pressed {
			if selected_ship.ship_type.is_some() {
				info!("Starting game with selected ship");
				next_state.set(GameState::Playing);
			}
		}
	}
}

pub fn cleanup_menu(
	mut commands: Commands,
	menu_query: Query<Entity, With<MenuUI>>,
) {
	for entity in &menu_query {
		commands.entity(entity).despawn_recursive();
	}
}
