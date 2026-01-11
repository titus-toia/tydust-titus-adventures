use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::systems::level::{CurrentLevel, MusicState, InfoOverlayEnabled};
use crate::components::{Player, Weapon, PlayerDefenses};

#[derive(Component)]
pub struct InfoOverlayContainer;

#[derive(Component)]
pub struct InfoOverlayText;

#[derive(Component)]
pub struct InfoControlsText;

pub fn spawn_info_overlay(mut commands: Commands) {
	// Top-left info overlay
	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			left: Val::Px(10.0),
			top: Val::Px(10.0),
			padding: UiRect::all(Val::Px(10.0)),
			flex_direction: FlexDirection::Column,
			..default()
		},
		BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
		InfoOverlayContainer,
		Visibility::Hidden,  // Start hidden
	)).with_children(|parent| {
		parent.spawn((
			Text::new(""),
			TextFont {
				font_size: 14.0,
				..default()
			},
			TextColor(Color::srgb(1.0, 1.0, 1.0)),
			InfoOverlayText,
		));
	});

	// Top-right controls guide
	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			right: Val::Px(10.0),
			top: Val::Px(60.0),
			padding: UiRect::all(Val::Px(10.0)),
			..default()
		},
		BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
		InfoOverlayContainer,
		Visibility::Hidden,
	)).with_children(|parent| {
		parent.spawn((
			Text::new("Controls:\nWASD - Move\nSpace - Fire\n+/- - Volume"),
			TextFont {
				font_size: 12.0,
				..default()
			},
			TextColor(Color::srgb(0.8, 0.8, 0.8)),
			InfoControlsText,
		));
	});
}

pub fn update_info_overlay(
	level: Option<Res<CurrentLevel>>,
	music_state: Res<MusicState>,
	diagnostics: Res<DiagnosticsStore>,
	player_query: Query<(&PlayerDefenses, &Weapon), With<Player>>,
	selected_level: Res<crate::systems::level::SelectedLevel>,
	mut text_query: Query<&mut Text, With<InfoOverlayText>>,
) {
	if let Some(level) = level {
		let phase_name = level.get_current_phase()
			.map(|p| p.name.clone())
			.unwrap_or_else(|| "Unknown".to_string());

		let distance = format!("{:.0}", level.distance);

		let music_path = music_state.current_track.as_deref().unwrap_or("None");

		let scroll_speed = level.get_scroll_speed();

		let fps = diagnostics
			.get(&FrameTimeDiagnosticsPlugin::FPS)
			.and_then(|fps| fps.smoothed())
			.unwrap_or(0.0);

		// Get player info
		let (armor_info, weapon_name, weapon_level) = if let Ok((defenses, weapon)) = player_query.get_single() {
			let armor_str = format!(
				"S2:{:.0}/{:.0} S1:{:.0}/{:.0} Arm:{:.0}/{:.0}",
				defenses.shield2, defenses.shield2_max,
				defenses.shield1, defenses.shield1_max,
				defenses.armor, defenses.armor_max
			);
			(
				armor_str,
				format!("{:?}", weapon.weapon_type),
				weapon.level
			)
		} else {
			("No Defenses".to_string(), "None".to_string(), 0)
		};

		// Format elapsed time as MM:SS
		let minutes = (level.time_elapsed / 60.0) as u32;
		let seconds = (level.time_elapsed % 60.0) as u32;
		let time_str = format!("{:02}:{:02}", minutes, seconds);

		let info_text = format!(
			"Level: {}\nPhase: {}\nDistance: {} GU\nTime: {}\nMusic: {}\nScroll: {:.1} GU/s\nFPS: {:.0}\n\nDefenses:\n{}\nWeapon: {}\nLevel: {}",
			selected_level.level_number,
			phase_name,
			distance,
			time_str,
			music_path,
			scroll_speed,
			fps,
			armor_info,
			weapon_name,
			weapon_level
		);

		for mut text in text_query.iter_mut() {
			**text = info_text.clone();
		}
	}
}

pub fn toggle_info_overlay_visibility(
	info_enabled: Res<InfoOverlayEnabled>,
	mut container_query: Query<&mut Visibility, With<InfoOverlayContainer>>,
) {
	for mut visibility in container_query.iter_mut() {
		*visibility = if info_enabled.0 {
			Visibility::Visible
		} else {
			Visibility::Hidden
		};
	}
}
