use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::collections::HashMap;

use crate::components::{FadingSound, PendingSound};
use crate::systems::level::SoundVolume;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SfxWhenFull {
	Reject,
	StealOldest,
}

/// Fire-and-forget SFX request that goes through the global SFX gate.
/// Use this instead of calling `audio.play(...)` directly in gameplay systems.
#[derive(Event, Clone, Copy, Debug)]
pub struct PlaySfxEvent {
	pub sound_path: &'static str,
	/// Volume multiplier applied before master volume (0.. maybe >1 for punchy sounds).
	pub volume: f32,
	/// Higher plays first when budget constrained (0-255).
	pub priority: u8,
	/// Cooldown to prevent spamming the same sound.
	pub cooldown_secs: f32,
	/// Max concurrent instances of this sound allowed at a time. 0 = unlimited.
	pub max_concurrent: u8,
	pub when_full: SfxWhenFull,
	/// Start fading out after this many seconds (optional).
	pub fade_after: Option<f32>,
	/// Duration of fade-out when stopping.
	pub fade_duration: f32,
}

impl PlaySfxEvent {
	pub fn simple(sound_path: &'static str, volume: f32, priority: u8, cooldown_secs: f32) -> Self {
		Self {
			sound_path,
			volume,
			priority,
			cooldown_secs,
			max_concurrent: 0,
			when_full: SfxWhenFull::Reject,
			fade_after: None,
			fade_duration: 0.0,
		}
	}
}

#[derive(Resource)]
pub struct SfxGateConfig {
	/// Max number of SFX to start per frame (prevents audio spam).
	pub max_starts_per_frame: usize,
	/// If any request >= this priority appears, drop all requests below `critical_floor_priority`.
	pub critical_threshold_priority: u8,
	pub critical_floor_priority: u8,
}

impl Default for SfxGateConfig {
	fn default() -> Self {
		Self {
			max_starts_per_frame: 12,
			critical_threshold_priority: 200,
			critical_floor_priority: 80,
		}
	}
}

#[derive(Resource, Default)]
pub struct SfxGateState {
	/// Last play time for each sound (in `Time::elapsed_secs_f64()`).
	last_played: HashMap<&'static str, f64>,
	/// Currently-playing instances by sound (best-effort; pruned when instances stop).
	active: HashMap<&'static str, Vec<Handle<AudioInstance>>>,
}

fn prune_active_for_sound(
	state: &mut SfxGateState,
	audio_instances: &Assets<AudioInstance>,
	sound_path: &'static str,
) {
	let Some(list) = state.active.get_mut(sound_path) else { return };

	list.retain(|h| {
		audio_instances
			.get(h)
			.map(|instance| instance.state() != PlaybackState::Stopped)
			.unwrap_or(false)
	});

	if list.is_empty() {
		state.active.remove(sound_path);
	}
}

fn try_play_sfx(
	commands: &mut Commands,
	audio: &Audio,
	asset_server: &AssetServer,
	audio_instances: &mut Assets<AudioInstance>,
	sound_volume: &SoundVolume,
	state: &mut SfxGateState,
	now: f64,
	req: PlaySfxEvent,
) -> bool {
	// Polyphony check.
	if req.max_concurrent > 0 {
		prune_active_for_sound(state, audio_instances, req.sound_path);
		let current = state.active.get(req.sound_path).map(|v| v.len()).unwrap_or(0);
		let limit = req.max_concurrent as usize;
		if current >= limit {
			match req.when_full {
				SfxWhenFull::Reject => return false,
				SfxWhenFull::StealOldest => {
					if let Some(list) = state.active.get_mut(req.sound_path) {
						if let Some(oldest) = list.first().cloned() {
							// Stop oldest quickly, then drop it from the list.
							if let Some(instance) = audio_instances.get_mut(&oldest) {
								instance.stop(AudioTween::linear(std::time::Duration::from_millis(30)));
							}
							list.remove(0);
						}
					}
				}
			}
		}
	}

	// Cooldown check.
	if req.cooldown_secs > 0.0 {
		let last = state.last_played.get(req.sound_path).copied().unwrap_or(-1.0e9);
		if now - last < req.cooldown_secs as f64 {
			return false;
		}
	}

	let handle = audio
		.play(asset_server.load(req.sound_path))
		.with_volume(sound_volume.apply(req.volume))
		.handle();

	state.last_played.insert(req.sound_path, now);
	state
		.active
		.entry(req.sound_path)
		.or_default()
		.push(handle.clone());

	if let Some(fade_after) = req.fade_after {
		commands.spawn(FadingSound {
			fade_timer: Timer::from_seconds(fade_after, TimerMode::Once),
			instance: handle,
		});
	} else {
		// Touch audio_instances so the handle actually exists in assets quickly; also prevents unused param warnings.
		let _ = audio_instances.get(&handle);
	}

	true
}

/// Central SFX gate:
/// - merges `PlaySfxEvent` + `PendingSound` whose timers finished this frame
/// - sorts by priority
/// - enforces per-sound cooldown + global per-frame budget
/// - applies master volume consistently
pub fn process_sfx_gate(
	mut commands: Commands,
	time: Res<Time>,
	audio: Res<Audio>,
	asset_server: Res<AssetServer>,
	mut audio_instances: ResMut<Assets<AudioInstance>>,
	sound_volume: Res<SoundVolume>,
	config: Res<SfxGateConfig>,
	mut state: ResMut<SfxGateState>,
	mut events: EventReader<PlaySfxEvent>,
	mut pending: Query<(Entity, &mut PendingSound)>,
) {
	let now = time.elapsed_secs_f64();

	let mut requests: Vec<PlaySfxEvent> = events.read().copied().collect();

	// Tick pending timers; collect any that are ready.
	for (entity, mut sound) in pending.iter_mut() {
		sound.delay.tick(time.delta());
		if sound.delay.finished() {
			requests.push(PlaySfxEvent {
				sound_path: sound.sound_path,
				volume: sound.volume,
				priority: sound.priority,
				cooldown_secs: sound.cooldown_secs,
				max_concurrent: sound.max_concurrent,
				when_full: if sound.steal_oldest { SfxWhenFull::StealOldest } else { SfxWhenFull::Reject },
				fade_after: sound.fade_after,
				fade_duration: sound.fade_duration,
			});
			commands.entity(entity).despawn();
		}
	}

	if requests.is_empty() {
		return;
	}

	// Highest priority first.
	requests.sort_by(|a, b| b.priority.cmp(&a.priority));

	let has_critical = requests
		.first()
		.map(|r| r.priority >= config.critical_threshold_priority)
		.unwrap_or(false);

	let floor = if has_critical {
		config.critical_floor_priority
	} else {
		0
	};

	let mut started = 0usize;
	for req in requests {
		if started >= config.max_starts_per_frame {
			break;
		}
		if req.priority < floor {
			continue;
		}

		// If fade_duration is requested, we still just stop() with a fixed tween in the fading system for now.
		// (We can plumb fade_duration through later if you want per-sound control.)
		let _ = req.fade_duration;

		if try_play_sfx(
			&mut commands,
			&audio,
			&asset_server,
			&mut audio_instances,
			&sound_volume,
			&mut state,
			now,
			req,
		) {
			started += 1;
		}
	}
}

pub fn process_fading_sounds(
	mut commands: Commands,
	time: Res<Time>,
	mut fading: Query<(Entity, &mut FadingSound)>,
	mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
	for (entity, mut sound) in fading.iter_mut() {
		sound.fade_timer.tick(time.delta());

		if sound.fade_timer.finished() {
			if let Some(instance) = audio_instances.get_mut(&sound.instance) {
				instance.stop(AudioTween::linear(std::time::Duration::from_millis(150)));
			}
			commands.entity(entity).despawn();
		}
	}
}

