use bevy::prelude::*;
use crate::components::{Player, PlayerDefenses, DefenseHexagon, DefenseLayer, ArmorDamageState, ArmorState, ChargeMeter, WeaponType, Weapon};
use crate::resources::SelectedWeapon;

#[derive(Component)]
pub struct PlayerHudContainer;

#[derive(Component)]
pub struct DefenseDisplayText;

#[derive(Component)]
pub struct Shield2Text;

#[derive(Component)]
pub struct Shield1Text;

#[derive(Component)]
pub struct ArmorText;

/// Marker for the charge meter rail sprite
#[derive(Component)]
pub struct ChargeMeterRail;

/// Marker for individual capacitor sprites in the charge meter
#[derive(Component)]
pub struct ChargeMeterCapacitor {
	/// Which capacitor index (0-9, left to right)
	pub index: u8,
}

/// State of a single capacitor
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapacitorState {
	Empty,
	Stock,
	Building,
	Critical,
}

/// Marker for "Enhanced mode offline" message elements
#[derive(Component)]
pub struct EnhancedModeOffline;

/// Marker for "Enhanced mode online" text (shown when level >= 8)
#[derive(Component)]
pub struct EnhancedModeOnline;

/// Marker for vintage lightbulbs flanking the title
#[derive(Component)]
pub struct EnhancedModeLightbulb {
	/// Charge threshold that lights this bulb (1.0, 2.0, 3.0, or 4.0)
	pub threshold: f32,
}

// === Charge meter layout constants ===
const PANEL_SCALE_Y: f32 = 0.21;
const TITLE_Y_OFFSET: f32 = 26.0;  // 5px up
const CAPACITOR_Y_OFFSET: f32 = -5.0;  // 5px up
const RAIL_Y_OFFSET: f32 = -33.0;  // 2.5px up
const CAPS_SPAN_RATIO: f32 = 0.58;
const CAPACITOR_SCALE: f32 = 0.17;  // 15% smaller
const RAIL_SCALE_MULT: f32 = 1.15;
const LIGHTBULB_SCALE: f32 = 0.067;  // 10% larger
const OFFLINE_ICON_SCALE: f32 = 0.123;  // 40% larger
const OFFLINE_ICON_X: f32 = -80.0;  // 5px right

/// Spawn the player HUD container with defense display
pub fn spawn_player_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
	// World-space position for hexagons (bottom-left visible area)
	let center = Vec2::new(-810.0, -340.0); // Adjusted 75px to the right

	// Spawn mesh panel background (bottom layer)
	commands.spawn((
		Sprite {
			image: asset_server.load("ui/hud_mesh_panel.png"),
			color: Color::srgba(1.0, 1.0, 1.0, 0.6), // 60% opacity
			..default()
		},
		Transform::from_xyz(center.x + 50.0, center.y + 50.0, 10.0) // Offset 50px right, 50px up
			.with_scale(Vec3::splat(0.4)), // Mesh scale (already vertical in file)
	));

	// Spawn digital display panel (mounted on mesh, encompasses hexagons + text)
	commands.spawn((
		Sprite {
			image: asset_server.load("ui/display_panel_vertical_green.png"),
			color: Color::srgba(1.0, 1.0, 1.0, 0.85),
			..default()
		},
		Transform::from_xyz(center.x + 9.0, center.y, 10.05) // Position at -815
			.with_scale(Vec3::splat(0.25)), // Same size as previous panel
	));

	// Load Orbitron font for HUD
	let orbitron_font = asset_server.load("fonts/Orbitron-Variable.ttf");

	// Spawn "DEFENCE" title at top of display
	commands.spawn((
		Text2d::new("DEFENCE"),
		TextFont {
			font: orbitron_font.clone(),
			font_size: 12.0,
			..default()
		},
		TextColor(Color::srgb(0.4, 1.0, 0.5)), // Light green to match display
		Transform::from_xyz(center.x + 9.0, center.y + 80.0, 10.2),
	));

	// Spawn Shield2 text (cyan) - top of display
	commands.spawn((
		Text2d::new("75"),
		TextFont {
			font: orbitron_font.clone(),
			font_size: 10.0, // Made smaller
			..default()
		},
		TextColor(Color::srgb(0.0, 1.0, 1.0)), // Cyan
		Transform::from_xyz(center.x + 9.0 - 28.0, center.y + 50.0, 10.2), // Spread further apart
		Shield2Text,
	));

	// Spawn Shield1 text (blue) - top center
	commands.spawn((
		Text2d::new("200"),
		TextFont {
			font: orbitron_font.clone(),
			font_size: 10.0, // Made smaller
			..default()
		},
		TextColor(Color::srgb(0.1, 0.4, 1.0)), // Deep blue
		Transform::from_xyz(center.x + 9.0, center.y + 50.0, 10.2),
		Shield1Text,
	));

	// Spawn Armor text (bronze) - top right
	commands.spawn((
		Text2d::new("100"),
		TextFont {
			font: orbitron_font,
			font_size: 10.0, // Made smaller
			..default()
		},
		TextColor(Color::srgb(0.7, 0.6, 0.4)), // Bronze
		Transform::from_xyz(center.x + 9.0 + 28.0, center.y + 50.0, 10.2), // Spread further apart
		ArmorText,
	));

	// Spawn Shield2 hexagon (outermost, cyan)
	commands.spawn((
		Sprite::from_image(asset_server.load("ui/shield2_cyan.png")),
		Transform::from_xyz(center.x + 9.0, center.y, 10.1)
			.with_scale(Vec3::splat(0.2295)), // Reduced by 23.5% total
		DefenseHexagon {
			layer: DefenseLayer::Shield2,
			base_size: 100.0,
			pulse_phase: 0.0,
			pulse_speed: 1.0,
			particle_spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
		},
	));

	// Spawn Shield1 hexagon (middle, deep blue)
	commands.spawn((
		Sprite::from_image(asset_server.load("ui/shield1_blue.png")),
		Transform::from_xyz(center.x + 9.0, center.y, 10.2)
			.with_scale(Vec3::splat(0.1683)), // Reduced by 23.5% total
		DefenseHexagon {
			layer: DefenseLayer::Shield1,
			base_size: 70.0,
			pulse_phase: 0.0,
			pulse_speed: 1.0,
			particle_spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
		},
	));

	// Spawn Armor hexagon (innermost, bronze)
	commands.spawn((
		Sprite::from_image(asset_server.load("ui/armor_bronze.png")),
		Transform::from_xyz(center.x + 9.0, center.y, 10.3)
			.with_scale(Vec3::splat(0.09945)), // Reduced by 23.5% total
		DefenseHexagon {
			layer: DefenseLayer::Armor,
			base_size: 40.0,
			pulse_phase: 0.0,
			pulse_speed: 1.0,
			particle_spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
		},
		ArmorDamageState {
			current_state: ArmorState::Intact,
		},
	));

	// === CHARGE METER (capacitor bank) ===
	// Mesh: 1024x1024 px at scale 0.4 = 409.6 world units, centered at (center.x + 50, center.y + 50)
	let mesh_center = Vec2::new(center.x + 50.0, center.y + 50.0);
	let mesh_width = 1024.0 * 0.4; // 409.6 world units
	let padding = 60.0; // 30px cut from each side

	// Panel width should fit within mesh with padding
	let panel_width = mesh_width - (padding * 2.0); // ~290 world units

	// Charge meter positioned relative to mesh
	let charge_meter_y = mesh_center.y + mesh_width / 2.0 - 75.0 - 35.0 + 85.0 - 45.0;
	let charge_meter_center = Vec2::new(mesh_center.x, charge_meter_y);

	// Panel: 1024px sprite, scale to fit
	let panel_scale_x = panel_width / 1024.0;
	let panel_scale_y = PANEL_SCALE_Y;
	commands.spawn((
		Sprite {
			image: asset_server.load("ui/digital_display_panel.png"),
			color: Color::srgba(1.0, 1.0, 1.0, 0.9),
			..default()
		},
		Transform::from_xyz(charge_meter_center.x, charge_meter_center.y, 10.04)
			.with_scale(Vec3::new(panel_scale_x, panel_scale_y, 1.0)),
		ChargeMeterRail,
	));

	// Vertical layout within panel
	let caps_span = panel_width * CAPS_SPAN_RATIO;
	let capacitor_spacing = caps_span / 9.0;
	let start_x = charge_meter_center.x - caps_span / 2.0;
	let capacitor_y = charge_meter_center.y + CAPACITOR_Y_OFFSET;
	let rail_y = charge_meter_center.y + RAIL_Y_OFFSET;

	// Rail underneath capacitors
	let rail_scale = caps_span / 512.0 * RAIL_SCALE_MULT;
	commands.spawn((
		Sprite::from_image(asset_server.load("ui/capacitor_rail.png")),
		Transform::from_xyz(charge_meter_center.x, rail_y, 10.05)
			.with_scale(Vec3::splat(rail_scale)),
	));

	// Spawn 10 capacitors (each represents 0.4 charge)
	for i in 0..10 {
		let x = start_x + (i as f32) * capacitor_spacing;
		commands.spawn((
			Sprite::from_image(asset_server.load("ui/capacitor_stock.png")),
			Transform::from_xyz(x, capacitor_y, 10.1)
				.with_scale(Vec3::splat(CAPACITOR_SCALE)),
			ChargeMeterCapacitor { index: i },
		));
	}

	// === "Enhanced mode online" title (shown when level >= 8) ===
	let title_y = charge_meter_center.y + TITLE_Y_OFFSET;
	let orbitron_font = asset_server.load("fonts/Orbitron-Variable.ttf");
	commands.spawn((
		Text2d::new("Enhanced mode online"),
		TextFont {
			font: orbitron_font.clone(),
			font_size: 9.0,
			..default()
		},
		TextColor(Color::srgba(0.4, 0.9, 1.0, 0.9)), // Cyan/electric blue
		Transform::from_xyz(charge_meter_center.x, title_y, 10.1),
		Visibility::Hidden,
		EnhancedModeOnline,
	));

	// Vintage lightbulbs flanking the "Enhanced mode online" text
	// Each lightbulb lights up when charge reaches its threshold
	let lightbulb_configs: [(f32, f32); 4] = [
		(-84.0, 1.0),  // Left outer → lights at 1.0 charge
		(-69.0, 2.0),  // Left inner → lights at 2.0 charge
		(69.0, 3.0),   // Right inner → lights at 3.0 charge
		(84.0, 4.0),   // Right outer → lights at 4.0 charge
	];
	for (x_offset, threshold) in lightbulb_configs {
		commands.spawn((
			Sprite::from_image(asset_server.load("ui/lightbulb_off_vintage.png")),
			Transform::from_xyz(
				charge_meter_center.x + x_offset,
				title_y,
				10.1
			).with_scale(Vec3::splat(LIGHTBULB_SCALE)),
			Visibility::Hidden,
			EnhancedModeOnline, // Reuse marker for visibility toggling
			EnhancedModeLightbulb { threshold },
		));
	}

	// === "Enhanced mode offline" message (shown when level < 8) ===
	commands.spawn((
		Sprite::from_image(asset_server.load("ui/enhanced_mode_offline.png")),
		Transform::from_xyz(charge_meter_center.x + OFFLINE_ICON_X, charge_meter_center.y, 10.1)
			.with_scale(Vec3::splat(OFFLINE_ICON_SCALE)),
		Visibility::Hidden,
		EnhancedModeOffline,
	));

	// "Enhanced mode offline." text
	commands.spawn((
		Text2d::new("Enhanced mode offline."),
		TextFont {
			font: orbitron_font,
			font_size: 10.0,
			..default()
		},
		TextColor(Color::srgba(0.6, 0.5, 0.4, 0.8)), // Dim amber/bronze
		Transform::from_xyz(charge_meter_center.x + 10.0, charge_meter_center.y, 10.1),
		Visibility::Hidden,
		EnhancedModeOffline,
	));
}

/// Calculate defense layer alpha based on current/max ratio (non-linear fade)
fn calculate_defense_alpha(current: f32, max: f32, base_alpha: f32) -> f32 {
	let ratio = (current / max).clamp(0.0, 1.0);
	// Non-linear mapping: more dramatic at low health
	// 100% = base_alpha, 50% = base_alpha * 0.6, 0% = base_alpha * 0.15
	base_alpha * (0.15 + 0.85 * ratio.powf(0.7))
}

/// Animate defense hexagons with pulse, color tinting, and armor sprite swapping
pub fn animate_defense_hexagons(
	mut hexagon_query: Query<(
		&mut Transform,
		&mut Sprite,
		&mut DefenseHexagon,
		Option<&mut ArmorDamageState>,
	)>,
	player_query: Query<&PlayerDefenses, With<Player>>,
	asset_server: Res<AssetServer>,
	time: Res<Time>,
) {
	let Ok(defenses) = player_query.get_single() else { return };

	for (mut transform, mut sprite, mut hexagon, armor_state) in hexagon_query.iter_mut() {
		let (current, max) = match hexagon.layer {
			DefenseLayer::Shield2 => (defenses.shield2, defenses.shield2_max),
			DefenseLayer::Shield1 => (defenses.shield1, defenses.shield1_max),
			DefenseLayer::Armor => (defenses.armor, defenses.armor_max),
		};

		let ratio = (current / max).clamp(0.0, 1.0);

		// === ARMOR SPRITE SWAPPING ===
		if hexagon.layer == DefenseLayer::Armor {
			if let Some(mut armor_state) = armor_state {
				let new_state = match ratio {
					r if r > 0.7 => ArmorState::Intact,
					r if r > 0.4 => ArmorState::Cracked,
					r if r > 0.15 => ArmorState::HeavyCracks,
					_ => ArmorState::Shattered,
				};

				if armor_state.current_state != new_state {
					armor_state.current_state = new_state;
					sprite.image = asset_server.load(match new_state {
						ArmorState::Intact => "ui/armor_bronze.png",
						ArmorState::Cracked => "ui/armor_bronze_cracked1.png",
						ArmorState::HeavyCracks => "ui/armor_bronze_cracked2.png",
						ArmorState::Shattered => "ui/armor_bronze_shattered.png",
					});
				}
			}

			// Armor doesn't pulse or tint - skip color/animation for armor
			let alpha = calculate_defense_alpha(current, max, 1.0);
			sprite.color = Color::srgb(1.0, 1.0, 1.0).with_alpha(alpha);
			continue;
		}

		// === SHIELD PULSE ANIMATION ===
		// Update pulse speed based on health ratio
		hexagon.pulse_speed = match ratio {
			r if r > 0.5 => 1.0,   // Slow pulse
			r if r > 0.2 => 2.5,   // Medium pulse
			_ => 5.0,              // Rapid pulse (critical)
		};

		// Advance pulse phase
		hexagon.pulse_phase += hexagon.pulse_speed * time.delta_secs();
		if hexagon.pulse_phase > std::f32::consts::TAU {
			hexagon.pulse_phase -= std::f32::consts::TAU;
		}

		// Calculate scale modulation
		let amplitude = match ratio {
			r if r > 0.5 => 0.02,  // Subtle
			r if r > 0.2 => 0.05,  // Noticeable
			_ => 0.10,              // Dramatic
		};

		let scale_multiplier = 1.0 + (hexagon.pulse_phase.sin() * amplitude);

		// Apply scale based on layer (maintain relative sizing)
		let base_scale = match hexagon.layer {
			DefenseLayer::Shield2 => 0.2295,
			DefenseLayer::Shield1 => 0.1683,
			DefenseLayer::Armor => 0.09945,
		};
		transform.scale = Vec3::splat(base_scale * scale_multiplier);

		// === SHIELD COLOR TINTING ===
		let base_color = match hexagon.layer {
			DefenseLayer::Shield2 => Color::srgb(0.0, 1.0, 1.0),  // Cyan
			DefenseLayer::Shield1 => Color::srgb(0.1, 0.4, 1.0),  // Deep blue
			DefenseLayer::Armor => Color::srgb(0.7, 0.6, 0.4),    // Bronze (unused here)
		};

		// Blend toward red/orange at low health
		let warning_color = Color::srgb(1.0, 0.3, 0.0);
		let tint_factor = (1.0 - ratio).powf(2.0); // Quadratic for dramatic shift

		let final_color = Color::srgb(
			base_color.to_srgba().red * (1.0 - tint_factor) + warning_color.to_srgba().red * tint_factor,
			base_color.to_srgba().green * (1.0 - tint_factor) + warning_color.to_srgba().green * tint_factor,
			base_color.to_srgba().blue * (1.0 - tint_factor) + warning_color.to_srgba().blue * tint_factor,
		);

		// Apply alpha (non-linear fade)
		let alpha = calculate_defense_alpha(current, max, 1.0);
		sprite.color = final_color.with_alpha(alpha);
	}
}

/// Update digital display text with current defense values
pub fn update_digital_display_text(
	player_query: Query<&PlayerDefenses, With<Player>>,
	mut shield2_query: Query<&mut Text2d, (With<Shield2Text>, Without<Shield1Text>, Without<ArmorText>)>,
	mut shield1_query: Query<&mut Text2d, (With<Shield1Text>, Without<Shield2Text>, Without<ArmorText>)>,
	mut armor_query: Query<&mut Text2d, (With<ArmorText>, Without<Shield2Text>, Without<Shield1Text>)>,
) {
	let Ok(defenses) = player_query.get_single() else { return };

	if let Ok(mut text) = shield2_query.get_single_mut() {
		**text = format!("{:.0}", defenses.shield2);
	}

	if let Ok(mut text) = shield1_query.get_single_mut() {
		**text = format!("{:.0}", defenses.shield1);
	}

	if let Ok(mut text) = armor_query.get_single_mut() {
		**text = format!("{:.0}", defenses.armor);
	}
}

/// Update charge meter capacitors and lightbulbs based on current charge state
pub fn update_charge_meter_ui(
	charge_meter: Res<ChargeMeter>,
	selected_weapon: Res<SelectedWeapon>,
	weapon_query: Query<&Weapon, With<Player>>,
	mut capacitor_query: Query<(&mut Sprite, &ChargeMeterCapacitor), Without<EnhancedModeLightbulb>>,
	mut lightbulb_query: Query<(&mut Sprite, &EnhancedModeLightbulb), Without<ChargeMeterCapacitor>>,
	mut rail_query: Query<&mut Visibility, (With<ChargeMeterRail>, Without<EnhancedModeOffline>, Without<EnhancedModeOnline>)>,
	mut offline_query: Query<&mut Visibility, (With<EnhancedModeOffline>, Without<ChargeMeterRail>, Without<EnhancedModeOnline>)>,
	mut online_query: Query<&mut Visibility, (With<EnhancedModeOnline>, Without<ChargeMeterRail>, Without<EnhancedModeOffline>)>,
	asset_server: Res<AssetServer>,
) {
	// Get weapon level
	let weapon_level = weapon_query.get_single().map(|w| w.level).unwrap_or(1);

	// Show panel for any lightning weapon, but capacitors only for level 8+
	let is_lightning = selected_weapon.weapon_type == WeaponType::LightningChain;
	let show_panel = is_lightning; // Panel visible for all lightning levels
	let show_capacitors = is_lightning && weapon_level >= 8;
	let show_offline = is_lightning && weapon_level < 8;
	let show_online = is_lightning && weapon_level >= 8;

	// Hide/show panel + rail (visible for any lightning weapon)
	for mut visibility in rail_query.iter_mut() {
		*visibility = if show_panel { Visibility::Visible } else { Visibility::Hidden };
	}

	// Hide/show offline message (when level < 8)
	for mut visibility in offline_query.iter_mut() {
		*visibility = if show_offline { Visibility::Visible } else { Visibility::Hidden };
	}

	// Hide/show online title (when level >= 8)
	for mut visibility in online_query.iter_mut() {
		*visibility = if show_online { Visibility::Visible } else { Visibility::Hidden };
	}

	// Process capacitors (only visible at level 8+)
	for (mut sprite, capacitor) in capacitor_query.iter_mut() {
		if !show_capacitors {
			sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
			continue;
		}

		// Each capacitor represents 0.4 charge
		// Capacitor 0 = charge 0.0-0.4, Capacitor 9 = charge 3.6-4.0
		let cap_threshold = (capacitor.index as f32 + 1.0) * 0.4;

		// Available stock (current minus what's being built)
		let available_stock = charge_meter.current;

		// Building charge "eats into" stock from right
		// If building 0.8 and have 4.0 stock, caps 9-8 show "building", rest show "stock"
		let building = charge_meter.charge_building;

		let state = if building > 0.0 {
			// When building, stock shows up to (available_stock - building) as stock
			// Then the next (building) amount shows as "building"
			let stock_threshold = available_stock - building;

			if cap_threshold <= stock_threshold {
				// This capacitor is fully in stock range
				CapacitorState::Stock
			} else if cap_threshold <= available_stock {
				// This capacitor is in the "building" range
				CapacitorState::Building
			} else {
				// Beyond available charge
				CapacitorState::Empty
			}
		} else {
			// Not building, just show stock
			if cap_threshold <= available_stock {
				// Check for critical (very low charge)
				if available_stock <= 0.4 && cap_threshold <= available_stock {
					CapacitorState::Critical
				} else {
					CapacitorState::Stock
				}
			} else {
				CapacitorState::Empty
			}
		};

		// Update sprite based on state
		let sprite_path = match state {
			CapacitorState::Empty => "ui/capacitor_empty.png",
			CapacitorState::Stock => "ui/capacitor_stock.png",
			CapacitorState::Building => "ui/capacitor_building.png",
			CapacitorState::Critical => "ui/capacitor_critical.png",
		};

		sprite.image = asset_server.load(sprite_path);
		sprite.color = Color::WHITE;
	}

	// Update lightbulbs based on charge thresholds
	// Each lightbulb lights up yellow-orange when charge >= its threshold
	let current_charge = charge_meter.current;
	for (mut sprite, lightbulb) in lightbulb_query.iter_mut() {
		let is_lit = current_charge >= lightbulb.threshold;

		if is_lit {
			// Lit: HDR yellow-orange glow via color (bloom will make it glow)
			sprite.color = Color::srgb(3.0, 2.0, 0.5);  // Warm incandescent HDR glow
		} else {
			// Unlit: dim gray
			sprite.color = Color::srgb(0.4, 0.4, 0.4);
		}
	}
}

/// Render glowing center on capacitors when building charge is maxed
pub fn render_capacitor_glow(
	mut gizmos: Gizmos,
	charge_meter: Res<ChargeMeter>,
	selected_weapon: Res<SelectedWeapon>,
	weapon_query: Query<&Weapon, With<Player>>,
	capacitor_query: Query<(&Transform, &ChargeMeterCapacitor)>,
) {
	// Only for lightning weapon level 8+
	if selected_weapon.weapon_type != WeaponType::LightningChain {
		return;
	}
	let weapon_level = weapon_query.get_single().map(|w| w.level).unwrap_or(1);
	if weapon_level < 8 {
		return;
	}

	// Only glow when building is maxed AND we have enough stock
	let building_maxed = charge_meter.charge_building >= 2.0 && charge_meter.current >= 2.0;
	if !building_maxed {
		return;
	}

	// Determine which capacitors are in "building" state
	let available_stock = charge_meter.current;
	let building = charge_meter.charge_building;
	let stock_threshold = available_stock - building;

	for (transform, capacitor) in capacitor_query.iter() {
		let cap_threshold = (capacitor.index as f32 + 1.0) * 0.4;

		// Check if this capacitor is in building state
		let is_building = building > 0.0
			&& cap_threshold > stock_threshold
			&& cap_threshold <= available_stock;

		if is_building {
			let pos = transform.translation.truncate() + Vec2::new(0.0, -1.5);

			// Layered HDR rectangles for stronger glow effect
			gizmos.rect_2d(pos, Vec2::new(5.0, 20.0), Color::srgba(1.0, 1.0, 1.0, 0.3));
			gizmos.rect_2d(pos, Vec2::new(4.0, 19.0), Color::srgb(2.0, 2.0, 2.0));
			gizmos.rect_2d(pos, Vec2::new(3.0, 18.0), Color::srgb(3.5, 3.5, 3.5));
		}
	}
}

/// Draw subtle red tick marks at charge levels 1, 2, 3, 4 using gizmos
/// Ticks only light up when charge reaches their threshold
pub fn render_charge_meter_ticks(
	mut gizmos: Gizmos,
	selected_weapon: Res<SelectedWeapon>,
	weapon_query: Query<&Weapon, With<Player>>,
	charge_meter: Res<ChargeMeter>,
	time: Res<Time>,
) {
	// Only show for lightning weapon level 8+
	if selected_weapon.weapon_type != WeaponType::LightningChain {
		return;
	}

	let weapon_level = weapon_query.get_single().map(|w| w.level).unwrap_or(1);
	if weapon_level < 8 {
		return;
	}

	// Same positioning calculation as spawn_player_hud
	let center = Vec2::new(-810.0, -340.0);
	let mesh_center = Vec2::new(center.x + 50.0, center.y + 50.0);
	let mesh_width = 1024.0 * 0.4;
	let padding = 60.0;
	let panel_width = mesh_width - (padding * 2.0);
	let charge_meter_y = mesh_center.y + mesh_width / 2.0 - 75.0 - 35.0 + 85.0 - 45.0;
	let charge_meter_center = Vec2::new(mesh_center.x, charge_meter_y);

	// Match vertical layout from spawn_player_hud
	let rail_y = charge_meter_center.y + RAIL_Y_OFFSET;
	let caps_span = panel_width * CAPS_SPAN_RATIO;
	let capacitor_spacing = caps_span / 9.0;
	let start_x = charge_meter_center.x - caps_span / 2.0;

	let pulse = (time.elapsed_secs() * 2.0).sin() * 0.15 + 0.85;

	// Tick dimensions - on the rail
	let tick_height = 12.0;
	let tick_y_base = rail_y; // Aligned with rail

	// Current charge level
	let current_charge = charge_meter.current;

	// Draw ticks at 1, 2, 3, 4 charge levels
	// Each tick lights up only when charge >= its threshold
	// (cap_index, is_on_cap, charge_threshold)
	let tick_positions: [(f32, bool, f32); 4] = [
		(1.5, false, 1.0),  // 1.0 charge - between caps
		(4.0, true, 2.0),   // 2.0 charge - exactly at cap 4
		(6.5, false, 3.0),  // 3.0 charge - between caps
		(9.0, true, 4.0),   // 4.0 charge - exactly at cap 9
	];

	for (cap_index, is_on_cap, threshold) in tick_positions {
		let x = start_x + cap_index * capacitor_spacing;

		let bottom = Vec2::new(x, tick_y_base - tick_height / 2.0);
		let top = Vec2::new(x, tick_y_base + tick_height / 2.0);

		// Determine if this tick is "lit" based on charge level
		let is_lit = current_charge >= threshold;

		if is_lit {
			// Lit tick: bright red with glow
			let base_alpha = 0.6;
			let tick_color = Color::srgba(1.0, 0.2, 0.1, base_alpha * pulse);

			// Glow effect
			let glow_color = Color::srgba(1.0, 0.3, 0.1, base_alpha * pulse * 0.3);
			for offset in [-2.0_f32, -1.0, 0.0, 1.0, 2.0] {
				let glow_bottom = Vec2::new(x + offset, tick_y_base - tick_height / 2.0 - 2.0);
				let glow_top = Vec2::new(x + offset, tick_y_base + tick_height / 2.0 + 2.0);
				gizmos.line_2d(glow_bottom, glow_top, glow_color);
			}

			// Core line
			gizmos.line_2d(bottom, top, tick_color);

			// If on a cap, draw brighter
			if is_on_cap {
				let bright = Color::srgba(1.0, 0.4, 0.2, base_alpha * pulse * 1.2);
				gizmos.line_2d(bottom, top, bright);
			}
		} else {
			// Unlit tick: dim gray, no glow
			let dim_color = Color::srgba(0.3, 0.2, 0.2, 0.3);
			gizmos.line_2d(bottom, top, dim_color);
		}
	}
}

/// Render shimmering lightning particles around "Enhanced mode online" title
pub fn render_enhanced_mode_sparks(
	mut gizmos: Gizmos,
	selected_weapon: Res<SelectedWeapon>,
	weapon_query: Query<&Weapon, With<Player>>,
	charge_meter: Res<ChargeMeter>,
	time: Res<Time>,
) {
	// Only show for lightning weapon level 8+
	if selected_weapon.weapon_type != WeaponType::LightningChain {
		return;
	}

	let weapon_level = weapon_query.get_single().map(|w| w.level).unwrap_or(1);
	if weapon_level < 8 {
		return;
	}

	// Same positioning as spawn_player_hud
	let center = Vec2::new(-810.0, -340.0);
	let mesh_center = Vec2::new(center.x + 50.0, center.y + 50.0);
	let mesh_width = 1024.0 * 0.4;
	let charge_meter_y = mesh_center.y + mesh_width / 2.0 - 75.0 - 35.0 + 85.0 - 45.0;
	let title_y_offset = 22.0;
	let title_y = charge_meter_y + title_y_offset;
	let title_center = Vec2::new(mesh_center.x, title_y);

	let t = time.elapsed_secs();

	// Generate zigzag lightning sparks
	for i in 0..5 {
		let phase = (i as f32) * 2.1 + t * (1.5 + i as f32 * 0.4);
		let spark_active = (phase.sin() * 2.5 + (phase * 1.7).cos() * 1.5) > 2.0;

		if !spark_active {
			continue;
		}

		// Spark origin - spread along the title width
		let x_offset = ((i as f32 - 2.0) * 30.0) + (phase * 0.5).sin() * 10.0;
		let y_offset = (phase * 1.2).cos() * 3.0;
		let origin = title_center + Vec2::new(x_offset, y_offset);

		// Main direction (mostly horizontal with some variation)
		let main_angle = (phase * 1.3).sin() * 0.4;
		let main_dir = Vec2::new(main_angle.cos(), main_angle.sin());
		let perp = Vec2::new(-main_dir.y, main_dir.x);

		// Flickering alpha
		let alpha = 0.5 + (phase * 3.5).sin().abs() * 0.4;
		let spark_color = Color::srgba(0.4, 0.85, 1.0, alpha);
		let bright_color = Color::srgba(0.85, 0.95, 1.0, alpha * 0.8);

		// Draw zigzag bolt (3-4 segments)
		let num_segments = 3 + ((phase * 2.0).sin() > 0.3) as usize;
		let segment_len = 3.0 + (phase * 1.4).sin().abs() * 2.0;
		let mut pos = origin;

		for seg in 0..num_segments {
			let seg_phase = phase + seg as f32 * 1.1;
			// Alternate zigzag direction
			let zigzag = if seg % 2 == 0 { 1.0 } else { -1.0 };
			let deviation = zigzag * (1.5 + (seg_phase * 2.0).sin().abs() * 2.0);

			let next_pos = pos + main_dir * segment_len + perp * deviation;

			// Draw segment
			gizmos.line_2d(pos, next_pos, spark_color);

			// Bright core on some segments
			if (seg_phase * 3.0).sin() > 0.5 {
				gizmos.line_2d(pos, next_pos, bright_color);
			}

			pos = next_pos;
		}

		// Occasional small branch from middle segment
		if (phase * 4.0).sin() > 0.6 && num_segments > 2 {
			let branch_start = origin + main_dir * segment_len;
			let branch_angle = main_angle + (phase * 2.5).sin() * 0.8;
			let branch_dir = Vec2::new(branch_angle.cos(), branch_angle.sin());
			let branch_end = branch_start + branch_dir * 4.0;
			let branch_color = Color::srgba(0.4, 0.85, 1.0, alpha * 0.6);
			gizmos.line_2d(branch_start, branch_end, branch_color);
		}
	}

	// Capacitor glow handled via HDR sprite colors in update_charge_meter_ui
}

