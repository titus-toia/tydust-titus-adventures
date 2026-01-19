use bevy::prelude::*;
use bevy_hanabi::prelude::*;

/// Resource holding all particle effect handles
#[derive(Resource)]
pub struct ParticleEffects {
    /// All named effects for debug display and spawning
    pub effects: Vec<NamedEffect>,
    /// Texture for aluminum flakes (large debris chunks)
    pub flake_texture: Handle<Image>,
}

pub struct NamedEffect {
    pub name: &'static str,
    pub handle: Handle<EffectAsset>,
    pub cleanup_time: f32,
    /// Whether this effect needs a texture (for EffectMaterial)
    pub needs_texture: bool,
}

/// Marker component for Hanabi effects (for cleanup)
#[derive(Component)]
pub struct HanabiEffect {
    pub lifetime: Timer,
}

/// Debug marker for effects spawned in debug grid
#[derive(Component)]
pub struct DebugEffect;

pub fn setup_particle_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: Res<AssetServer>,
) {
    let mut named_effects = Vec::new();

    // Load flake texture
    let flake_texture: Handle<Image> = asset_server.load("particles/spark_white.png");

    // === EXPLOSION SPARKS ===

    // Small explosion: 200 sparks, tight spread (125px max)
    named_effects.push(NamedEffect {
        name: "small_sparks",
        handle: create_spark_burst(&mut effects, 200, 60.0, 100.0, 0.5, 1.25, 6.0, 12.0),
        cleanup_time: 1.5,
        needs_texture: false,
    });

    // Medium explosion: 150 sparks
    named_effects.push(NamedEffect {
        name: "medium_sparks",
        handle: create_spark_burst(&mut effects, 150, 180.0, 380.0, 0.4, 0.8, 4.0, 8.0),
        cleanup_time: 1.0,
        needs_texture: false,
    });

    // Large explosion: 300 sparks
    named_effects.push(NamedEffect {
        name: "large_sparks",
        handle: create_spark_burst(&mut effects, 300, 200.0, 450.0, 0.5, 1.0, 5.0, 10.0),
        cleanup_time: 1.2,
        needs_texture: false,
    });

    // === ALUMINUM FLAKES (textured for large visible chunks) ===

    // Small aluminum flakes - visible burning debris
    named_effects.push(NamedEffect {
        name: "small_aluminum",
        handle: create_aluminum_flakes(&mut effects, 8, 15.0, 50.0, 2.0, 3.0, 40.0, 80.0, 15.0),
        cleanup_time: 3.5,
        needs_texture: true,
    });

    // Medium aluminum flakes - big burning chunks
    named_effects.push(NamedEffect {
        name: "medium_aluminum",
        handle: create_aluminum_flakes(&mut effects, 12, 20.0, 70.0, 2.5, 4.0, 60.0, 120.0, 18.0),
        cleanup_time: 4.5,
        needs_texture: true,
    });

    // Large aluminum flakes - massive glowing debris
    named_effects.push(NamedEffect {
        name: "large_aluminum",
        handle: create_aluminum_flakes(&mut effects, 18, 25.0, 90.0, 3.0, 5.0, 80.0, 160.0, 20.0),
        cleanup_time: 5.5,
        needs_texture: true,
    });

    commands.insert_resource(ParticleEffects {
        effects: named_effects,
        flake_texture,
    });
}

/// Create a standard spark burst effect (orange/yellow fast sparks)
fn create_spark_burst(
    effects: &mut Assets<EffectAsset>,
    particle_count: u32,
    speed_min: f32,
    speed_max: f32,
    lifetime_min: f32,
    lifetime_max: f32,
    size_min: f32,
    size_max: f32,
) -> Handle<EffectAsset> {
    let writer = ExprWriter::new();

    let age = writer.lit(0.0).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(lifetime_min).uniform(writer.lit(lifetime_max)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(5.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    let speed = writer.lit(speed_min).uniform(writer.lit(speed_max)).expr();
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed,
    };

    let size = writer.lit(size_min).uniform(writer.lit(size_max)).expr();
    let init_size = SetAttributeModifier::new(Attribute::SIZE, size);

    // Orange/yellow spark gradient
    let color_gradient = Gradient::new()
        .with_key(0.0, Vec4::new(1.0, 0.95, 0.7, 1.0))  // Bright yellow-white
        .with_key(0.3, Vec4::new(1.0, 0.7, 0.2, 1.0))   // Orange
        .with_key(0.7, Vec4::new(0.9, 0.3, 0.1, 0.8))   // Red-orange
        .with_key(1.0, Vec4::new(0.5, 0.1, 0.0, 0.0));  // Fade out

    let size_gradient = Gradient::new()
        .with_key(0.0, Vec3::splat(1.0))
        .with_key(0.5, Vec3::splat(0.6))
        .with_key(1.0, Vec3::splat(0.0));

    let spawner = SpawnerSettings::once((particle_count as f32).into());

    let effect = EffectAsset::new(particle_count, spawner, writer.finish())
        .with_name("spark_burst")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_size)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        });

    effects.add(effect)
}

/// Create burning aluminum flakes - bright white/silver flickering debris that floats down
/// Uses a texture so particles render as quads (not size-limited points)
fn create_aluminum_flakes(
    effects: &mut Assets<EffectAsset>,
    particle_count: u32,
    speed_min: f32,
    speed_max: f32,
    lifetime_min: f32,
    lifetime_max: f32,
    size_min: f32,
    size_max: f32,
    gravity: f32,
) -> Handle<EffectAsset> {
    let writer = ExprWriter::new();

    // Texture slot index 0 (corresponds to first image in EffectMaterial::images)
    let texture_slot = writer.lit(0u32).expr();

    let age = writer.lit(0.0).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(lifetime_min).uniform(writer.lit(lifetime_max)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Start slightly spread out
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(15.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Slower initial burst, more upward bias
    let speed = writer.lit(speed_min).uniform(writer.lit(speed_max)).expr();
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed,
    };

    let size = writer.lit(size_min).uniform(writer.lit(size_max)).expr();
    let init_size = SetAttributeModifier::new(Attribute::SIZE, size);

    // Bright white/silver with orange hot spots, flickering effect via gradient
    // The "flicker" is simulated by having brightness variations in the gradient
    let color_gradient = Gradient::new()
        .with_key(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0))    // Pure white (hot)
        .with_key(0.1, Vec4::new(1.0, 0.9, 0.7, 1.0))    // Warm white
        .with_key(0.2, Vec4::new(1.0, 1.0, 0.95, 1.0))   // Back to white (flicker)
        .with_key(0.35, Vec4::new(1.0, 0.85, 0.6, 1.0))  // Orange tint
        .with_key(0.5, Vec4::new(1.0, 0.95, 0.9, 0.95))  // White again
        .with_key(0.65, Vec4::new(1.0, 0.7, 0.4, 0.9))   // More orange
        .with_key(0.8, Vec4::new(0.9, 0.6, 0.3, 0.7))    // Cooling down
        .with_key(1.0, Vec4::new(0.5, 0.4, 0.3, 0.0));   // Fade to dark

    // Size pulses slightly then shrinks
    let size_gradient = Gradient::new()
        .with_key(0.0, Vec3::splat(0.8))
        .with_key(0.15, Vec3::splat(1.0))   // Pulse up
        .with_key(0.3, Vec3::splat(0.9))    // Pulse down
        .with_key(0.5, Vec3::splat(1.0))    // Pulse up again
        .with_key(0.7, Vec3::splat(0.7))
        .with_key(1.0, Vec3::splat(0.0));   // Shrink to nothing

    // Gravity and drag expressions (must be before writer.finish())
    let accel = writer.lit(Vec3::new(0.0, -gravity, 0.0)).expr();
    let drag = writer.lit(1.5).expr();

    let spawner = SpawnerSettings::once((particle_count as f32).into());
    let module = writer.finish();

    let effect = EffectAsset::new(particle_count, spawner, module)
        .with_name("aluminum_flakes")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_size)
        // Gravity pulls flakes down, drag slows them (flutter effect)
        .update(AccelModifier::new(accel))
        .update(LinearDragModifier::new(drag))
        // Texture makes particles render as quads, not points
        .render(ParticleTextureModifier::new(texture_slot))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Modulate,  // Modulate with texture
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        });

    effects.add(effect)
}

// === SPAWNING HELPERS ===

#[derive(Clone, Copy, Debug)]
pub enum ExplosionSize {
    Small,
    Medium,
    Large,
}

/// Spawn a complete explosion with sparks + aluminum flakes
pub fn spawn_explosion_effect(
    commands: &mut Commands,
    effects: &ParticleEffects,
    position: Vec3,
    size: ExplosionSize,
) {
    let (spark_name, aluminum_name, cleanup_time) = match size {
        ExplosionSize::Small => ("small_sparks", "small_aluminum", 2.5),
        ExplosionSize::Medium => ("medium_sparks", "medium_aluminum", 3.0),
        ExplosionSize::Large => ("large_sparks", "large_aluminum", 3.5),
    };

    // Spawn sparks (no texture needed)
    if let Some(effect) = effects.effects.iter().find(|e| e.name == spark_name) {
        commands.spawn((
            ParticleEffect::new(effect.handle.clone()),
            Transform::from_translation(position),
            HanabiEffect {
                lifetime: Timer::from_seconds(cleanup_time, TimerMode::Once),
            },
        ));
    }

    // Spawn aluminum flakes (with texture for large quad rendering)
    if let Some(effect) = effects.effects.iter().find(|e| e.name == aluminum_name) {
        commands.spawn((
            ParticleEffect::new(effect.handle.clone()),
            Transform::from_translation(position),
            HanabiEffect {
                lifetime: Timer::from_seconds(cleanup_time, TimerMode::Once),
            },
            EffectMaterial {
                images: vec![effects.flake_texture.clone()],
            },
        ));
    }
}

/// Spawn a single named effect (for debug or custom use)
pub fn spawn_named_effect(
    commands: &mut Commands,
    effects: &ParticleEffects,
    name: &str,
    position: Vec3,
) -> bool {
    if let Some(effect) = effects.effects.iter().find(|e| e.name == name) {
        let mut entity = commands.spawn((
            ParticleEffect::new(effect.handle.clone()),
            Transform::from_translation(position),
            HanabiEffect {
                lifetime: Timer::from_seconds(effect.cleanup_time, TimerMode::Once),
            },
        ));
        if effect.needs_texture {
            entity.insert(EffectMaterial {
                images: vec![effects.flake_texture.clone()],
            });
        }
        true
    } else {
        false
    }
}

// === DEBUG DISPLAY ===

/// Spawn all effects in a grid for visual debugging
pub fn spawn_debug_effect_grid(
    mut commands: Commands,
    effects: Res<ParticleEffects>,
    keyboard: Res<ButtonInput<KeyCode>>,
    existing: Query<Entity, With<DebugEffect>>,
) {
    // Press T to spawn debug grid
    if !keyboard.just_pressed(KeyCode::KeyT) {
        return;
    }

    // Clear existing debug effects
    for entity in existing.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let effect_count = effects.effects.len();
    let cols = 3;
    let spacing_x = 200.0;
    let spacing_y = 250.0;
    let start_x = -((cols - 1) as f32 * spacing_x) / 2.0;
    let start_y = 200.0;

    info!("Spawning {} debug effects in grid", effect_count);

    for (i, effect) in effects.effects.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;

        let x = start_x + col as f32 * spacing_x;
        let y = start_y - row as f32 * spacing_y;
        let pos = Vec3::new(x, y, 5.0);

        info!("  [{}] {} at ({}, {}) needs_texture={}", i, effect.name, x, y, effect.needs_texture);

        // Spawn the effect
        let mut entity = commands.spawn((
            ParticleEffect::new(effect.handle.clone()),
            Transform::from_translation(pos),
            HanabiEffect {
                lifetime: Timer::from_seconds(effect.cleanup_time + 1.0, TimerMode::Once),
            },
            DebugEffect,
        ));
        if effect.needs_texture {
            entity.insert(EffectMaterial {
                images: vec![effects.flake_texture.clone()],
            });
        }

        // Spawn label
        commands.spawn((
            Text2d::new(effect.name),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(pos + Vec3::new(0.0, -80.0, 0.0)),
            DebugEffect,
        ));
    }
}

/// Cleanup finished effects
pub fn cleanup_hanabi_effects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HanabiEffect)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.lifetime.tick(time.delta());
        if effect.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// === BACKWARDS COMPATIBILITY ===

/// Legacy resource alias
pub type ExplosionEffects = ParticleEffects;

/// Legacy setup function alias
pub use setup_particle_effects as setup_explosion_effects;

/// Legacy cleanup function alias
pub use cleanup_hanabi_effects as cleanup_explosion_effects;
