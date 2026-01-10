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

const BUTTON_WIDTH: f32 = 180.0;
const BUTTON_HEIGHT: f32 = 240.0;
const BUTTON_SPACING: f32 = 15.0;
const SHIP_PREVIEW_SIZE: f32 = 120.0;

pub fn setup_ship_selection_menu(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	let font = asset_server.load("fonts/Orbitron-Variable.ttf");
	// Root scrollable container
	commands
		.spawn((
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				overflow: Overflow::scroll_y(),
				..default()
			},
			BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
			MenuUI,
		))
		.with_children(|root| {
			// Inner content container
			root.spawn(Node {
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				padding: UiRect::all(Val::Px(20.0)),
				..default()
			})
			.with_children(|parent| {
			// Title
			parent.spawn((
				Text::new("SELECT YOUR FIGHTER"),
				TextFont {
					font: font.clone(),
					font_size: 36.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					margin: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(20.0), Val::Px(15.0)),
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
						spawn_ship_button(row, ship_type, &asset_server, &font);
					}
				});

			// Weapon selection title
			parent.spawn((
				Text::new("SELECT YOUR WEAPON"),
				TextFont {
					font: font.clone(),
					font_size: 28.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					margin: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(20.0), Val::Px(15.0)),
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
					spawn_weapon_button(row, WeaponType::BasicBlaster, &asset_server, &font);
					spawn_weapon_button(row, WeaponType::PlasmaCannon, &asset_server, &font);
					spawn_weapon_button(row, WeaponType::WaveGun, &asset_server, &font);
					spawn_weapon_button(row, WeaponType::SpreadShot, &asset_server, &font);
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
					spawn_weapon_button(row, WeaponType::MissilePods, &asset_server, &font);
					spawn_weapon_button(row, WeaponType::LaserArray, &asset_server, &font);
					spawn_weapon_button(row, WeaponType::OrbitalDefense, &asset_server, &font);
					spawn_weapon_button(row, WeaponType::LightningChain, &asset_server, &font);
				});

			// Start button (initially hidden until selection made)
			parent.spawn((
				Node {
					width: Val::Px(280.0),
					height: Val::Px(55.0),
					margin: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(25.0), Val::Px(10.0)),
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
						font: font.clone(),
						font_size: 24.0,
						..default()
					},
					TextColor(Color::srgb(1.0, 1.0, 1.0)),
				));
			});
		});
		});
}

fn spawn_ship_button(
	parent: &mut ChildBuilder,
	ship_type: ShipType,
	asset_server: &Res<AssetServer>,
	font: &Handle<Font>,
) {
	let stats = ship_type.get_stats();
	let ship_name = format!("{:?}", ship_type);

	parent
		.spawn((
			Node {
				width: Val::Px(BUTTON_WIDTH),
				height: Val::Px(BUTTON_HEIGHT),
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::SpaceBetween,
				align_items: AlignItems::Center,
				padding: UiRect::all(Val::Px(12.0)),
				border: UiRect::all(Val::Px(2.0)),
				..default()
			},
			BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
			BorderColor(Color::srgb(0.3, 0.3, 0.4)),
			Button,
			ShipButton { ship_type },
		))
		.with_children(|button| {
			// Ship name at top
			button.spawn((
				Text::new(ship_name.to_uppercase()),
				TextFont {
					font: font.clone(),
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					align_self: AlignSelf::Center,
					..default()
				},
			));

			// Ship preview image - centered
			button.spawn((
				ImageNode {
					image: asset_server.load(ship_type.sprite_path()),
					..default()
				},
				Node {
					width: Val::Px(SHIP_PREVIEW_SIZE),
					height: Val::Px(SHIP_PREVIEW_SIZE),
					align_self: AlignSelf::Center,
					..default()
				},
			));

			// Stats display at bottom
			let stats_text = format!(
				"Speed: {:.0}\nFire: {:.2}s\nSize: {:.0}\n{}",
				stats.speed, stats.fire_cooldown, stats.size, stats.description
			);
			button.spawn((
				Text::new(stats_text),
				TextFont {
					font: font.clone(),
					font_size: 11.0,
					..default()
				},
				TextColor(Color::srgb(0.7, 0.7, 0.8)),
				TextLayout::new_with_justify(JustifyText::Center),
				Node {
					align_self: AlignSelf::Stretch,
					..default()
				},
			));
		});
}

fn spawn_weapon_button(
	parent: &mut ChildBuilder,
	weapon_type: WeaponType,
	asset_server: &Res<AssetServer>,
	font: &Handle<Font>,
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
		WeaponType::LightningChain => ("Chain lightning whips, recursive chaos", "1-10", "sprites/weapons/lightning_chain.png"),
	};

	parent
		.spawn((
			Node {
				width: Val::Px(BUTTON_WIDTH),
				height: Val::Px(BUTTON_HEIGHT),
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::SpaceBetween,
				align_items: AlignItems::Center,
				padding: UiRect::all(Val::Px(12.0)),
				border: UiRect::all(Val::Px(2.0)),
				..default()
			},
			BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
			BorderColor(Color::srgb(0.3, 0.3, 0.4)),
			Button,
			WeaponButton { weapon_type },
		))
		.with_children(|button| {
			// Weapon name at top
			button.spawn((
				Text::new(weapon_name.to_uppercase()),
				TextFont {
					font: font.clone(),
					font_size: 16.0,
					..default()
				},
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Node {
					align_self: AlignSelf::Center,
					..default()
				},
			));

			// Weapon icon preview - centered
			button.spawn((
				ImageNode {
					image: asset_server.load(icon_path),
					..default()
				},
				Node {
					width: Val::Px(100.0),
					height: Val::Px(100.0),
					align_self: AlignSelf::Center,
					..default()
				},
			));

			// Stats display at bottom
			let stats_text = format!(
				"DMG: {:.0} | SPD: {:.0}\nLevels: {}\n{}",
				config.base_damage, config.projectile_speed, levels, description
			);
			button.spawn((
				Text::new(stats_text),
				TextFont {
					font: font.clone(),
					font_size: 11.0,
					..default()
				},
				TextColor(Color::srgb(0.7, 0.7, 0.8)),
				TextLayout::new_with_justify(JustifyText::Center),
				Node {
					align_self: AlignSelf::Stretch,
					..default()
				},
			));
		});
}

pub fn handle_weapon_selection(
	interaction_query: Query<(&Interaction, &WeaponButton), Changed<Interaction>>,
	mut all_weapons: Query<(&WeaponButton, &mut BackgroundColor, &mut BorderColor)>,
	mut selected_weapon: ResMut<SelectedWeapon>,
) {
	for (interaction, weapon_button) in &interaction_query {
		if *interaction == Interaction::Pressed {
			selected_weapon.weapon_type = weapon_button.weapon_type;
			info!("Selected weapon: {:?}", weapon_button.weapon_type);

			// Update all weapon button styles
			for (button, mut bg, mut border) in all_weapons.iter_mut() {
				if button.weapon_type == weapon_button.weapon_type {
					*bg = BackgroundColor(Color::srgba(0.3, 0.4, 0.6, 0.9));
					*border = BorderColor(Color::srgb(0.5, 0.7, 1.0));
				} else {
					*bg = BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9));
					*border = BorderColor(Color::srgb(0.3, 0.3, 0.4));
				}
			}
		}
	}
}

pub fn handle_ship_selection(
	interaction_query: Query<(&Interaction, &ShipButton), Changed<Interaction>>,
	mut all_ships: Query<(&ShipButton, &mut BackgroundColor, &mut BorderColor)>,
	mut selected_ship: ResMut<SelectedShip>,
	mut start_button_query: Query<&mut Node, With<StartGameButton>>,
) {
	for (interaction, ship_button) in &interaction_query {
		if *interaction == Interaction::Pressed {
			selected_ship.ship_type = Some(ship_button.ship_type);
			info!("Selected ship: {:?}", ship_button.ship_type);

			// Show start button
			if let Ok(mut node) = start_button_query.get_single_mut() {
				node.display = Display::Flex;
			}

			// Update all ship button styles
			for (button, mut bg, mut border) in all_ships.iter_mut() {
				if button.ship_type == ship_button.ship_type {
					*bg = BackgroundColor(Color::srgba(0.3, 0.4, 0.6, 0.9));
					*border = BorderColor(Color::srgb(0.5, 0.7, 1.0));
				} else {
					*bg = BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9));
					*border = BorderColor(Color::srgb(0.3, 0.3, 0.4));
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
