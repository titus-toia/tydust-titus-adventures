use bevy::prelude::*;

use crate::components::{OneshotEffect, SpriteFrameAnimation};

pub fn animate_sprite_frames(
	time: Res<Time>,
	mut q: Query<(&mut Sprite, &mut SpriteFrameAnimation)>,
) {
	for (mut sprite, mut anim) in &mut q {
		if anim.frames.is_empty() {
			continue;
		}

		anim.timer.tick(time.delta());
		let steps = anim.timer.times_finished_this_tick();
		if steps == 0 {
			continue;
		}

		for _ in 0..steps {
			// Advance frame
			if anim.current + 1 >= anim.frames.len() {
				if anim.looping {
					anim.current = 0;
				} else {
					anim.current = anim.frames.len() - 1;
				}
			} else {
				anim.current += 1;
			}

			sprite.image = anim.frames[anim.current].clone();
		}
	}
}

/// Despawn one-shot effect entities when their animation finishes.
pub fn cleanup_oneshot_effects(
	mut commands: Commands,
	q: Query<(Entity, &SpriteFrameAnimation), With<OneshotEffect>>,
) {
	for (entity, anim) in &q {
		if anim.is_finished() {
			commands.entity(entity).despawn_recursive();
		}
	}
}

