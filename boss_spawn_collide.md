# Boss & Complex Ship Multi-Collider Design

## Problem

Boss ships and complex enemies need multiple hitboxes where hitting any region counts as a hit. Current collision system only supports single circle per enemy.

## Two Approaches

### Option 1: Child Entities with Colliders (Recommended)

**Design:**
- Boss is parent entity
- Each hitbox region is a child entity with its own `Collider`
- Children reference parent via `ColliderPart` component
- Collision system hits children, damage routes to parent

**Component Structure:**

```rust
#[derive(Component)]
pub struct ColliderPart {
    pub parent_entity: Entity,  // The boss this collider belongs to
    pub damage_multiplier: f32, // Optional: weak points do 2x damage
}
```

**Boss Spawn Example:**

```rust
// Spawn boss
let boss_entity = commands.spawn((
    Enemy { enemy_type: EnemyType::Boss, .. },
    Health::new(5000.0),
    Transform::from_xyz(400.0, 1000.0, 0.0),
    // Boss visual sprite
)).id();

// Spawn collider parts as children
commands.spawn((
    Collider::new(80.0),  // Core hitbox
    ColliderPart { parent_entity: boss_entity, damage_multiplier: 1.0 },
    Transform::from_xyz(0.0, 0.0, 1.0),  // Centered on boss
    Parent(boss_entity),
));

commands.spawn((
    Collider::new(40.0),  // Left wing
    ColliderPart { parent_entity: boss_entity, damage_multiplier: 0.5 },
    Transform::from_xyz(-100.0, 0.0, 1.0),  // Left offset
    Parent(boss_entity),
));

commands.spawn((
    Collider::new(40.0),  // Right wing
    ColliderPart { parent_entity: boss_entity, damage_multiplier: 0.5 },
    Transform::from_xyz(100.0, 0.0, 1.0),  // Right offset
    Parent(boss_entity),
));
```

**Collision System Changes:**

```rust
pub fn check_projectile_enemy_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile)>,

    // Regular single-collider enemies
    enemies: Query<(Entity, &Transform, &Collider, &Enemy), (Without<ColliderPart>, Without<Player>)>,

    // Multi-collider parts
    collider_parts: Query<(&Transform, &Collider, &ColliderPart, &GlobalTransform)>,

    mut hit_events: EventWriter<EnemyHitEvent>,
) {
    for (proj_entity, proj_transform, projectile) in projectiles.iter() {
        let proj_pos = proj_transform.translation.truncate();
        let proj_radius = projectile.damage.sqrt() * 2.0;

        // Check regular enemies (existing code)
        for (enemy_entity, enemy_transform, collider, _enemy) in enemies.iter() {
            // ... existing circle check ...
        }

        // Check multi-collider parts
        for (part_transform, part_collider, part_info, global_transform) in collider_parts.iter() {
            let part_pos = global_transform.translation().truncate();  // Use global position!
            let distance = proj_pos.distance(part_pos);

            if distance < proj_radius + part_collider.radius {
                hit_events.send(EnemyHitEvent {
                    enemy: part_info.parent_entity,  // Route to parent!
                    damage: projectile.damage * part_info.damage_multiplier,
                    hit_sound: None,
                });
                commands.entity(proj_entity).despawn();
                break;
            }
        }
    }
}
```

**Pros:**
- ✓ Very flexible (each part can have different shapes, properties)
- ✓ Can despawn parts independently (break off a wing!)
- ✓ Natural for bosses with turrets or destructible parts
- ✓ Bevy's transform hierarchy handles offsets automatically
- ✓ Easy to debug (visualize each collider separately)
- ✓ Supports damage multipliers per part (weak points!)

**Cons:**
- ✗ More entities to manage
- ✗ Slightly more complex queries

---

### Option 2: Vec of Shapes in Single Collider

**Design:**
- Single `Collider` component contains multiple shapes
- Each shape has local offset from entity center

**Component Structure:**

```rust
pub struct ColliderShape {
    pub shape_type: ShapeType,
    pub offset: Vec2,  // Local offset from entity center
}

pub enum ShapeType {
    Circle { radius: f32 },
    Capsule { radius: f32, length: f32, angle: f32 },
}

pub struct Collider {
    pub shapes: Vec<ColliderShape>,
}

impl Collider {
    pub fn single_circle(radius: f32) -> Self {
        Self {
            shapes: vec![ColliderShape {
                shape_type: ShapeType::Circle { radius },
                offset: Vec2::ZERO,
            }]
        }
    }

    pub fn multi_circle(circles: Vec<(f32, Vec2)>) -> Self {
        Self {
            shapes: circles.into_iter().map(|(radius, offset)| {
                ColliderShape {
                    shape_type: ShapeType::Circle { radius },
                    offset,
                }
            }).collect()
        }
    }
}
```

**Boss Spawn Example:**

```rust
commands.spawn((
    Enemy { enemy_type: EnemyType::Boss, .. },
    Health::new(5000.0),
    Collider::multi_circle(vec![
        (80.0, Vec2::new(0.0, 0.0)),    // Core
        (40.0, Vec2::new(-100.0, 0.0)), // Left wing
        (40.0, Vec2::new(100.0, 0.0)),  // Right wing
    ]),
    Transform::from_xyz(400.0, 1000.0, 0.0),
));
```

**Collision System Changes:**

```rust
pub fn check_projectile_enemy_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Projectile)>,
    enemies: Query<(Entity, &Transform, &Collider, &Enemy), Without<Player>>,
    mut hit_events: EventWriter<EnemyHitEvent>,
) {
    for (proj_entity, proj_transform, projectile) in projectiles.iter() {
        let proj_pos = proj_transform.translation.truncate();
        let proj_radius = projectile.damage.sqrt() * 2.0;

        for (enemy_entity, enemy_transform, collider, _enemy) in enemies.iter() {
            let enemy_pos = enemy_transform.translation.truncate();

            // Check each shape in the collider
            let mut hit = false;
            for shape in &collider.shapes {
                let shape_pos = enemy_pos + shape.offset;

                match shape.shape_type {
                    ShapeType::Circle { radius } => {
                        let distance = proj_pos.distance(shape_pos);
                        if distance < proj_radius + radius {
                            hit = true;
                            break;
                        }
                    },
                    // ... other shapes
                }
            }

            if hit {
                hit_events.send(EnemyHitEvent {
                    enemy: enemy_entity,
                    damage: projectile.damage,
                    hit_sound: None,
                });
                commands.entity(proj_entity).despawn();
                break;
            }
        }
    }
}
```

**Pros:**
- ✓ Fewer entities (everything on boss entity)
- ✓ Simpler entity management
- ✓ Good for static multi-box configs

**Cons:**
- ✗ Can't have different damage multipliers per part
- ✗ Can't despawn parts independently
- ✗ All shapes must share same collision properties

---

## Recommendation

**Use Option 1 (Child Entities) for this game**

**Reasons:**
1. **Bosses are rare** (1-2 per level), so entity overhead doesn't matter
2. **Future flexibility**: You might want destructible parts, weak points, turrets
3. **Bevy-idiomatic**: Uses Parent/Child hierarchy naturally
4. **Easier debugging**: Can visualize each collider separately in dev tools
5. **Supports gameplay features**: Damage multipliers, breakable parts, etc.

**When to use Option 2:**
- Simple ships with 2-3 static hitboxes
- Performance-critical scenarios (hundreds of multi-collider entities)
- All parts share same damage/behavior

---

## Current Collision System Assessment

**File:** `src/systems/collision.rs`

**Current Implementation:**
- Simple circle-circle distance checks
- Three collision scenarios:
  - Projectile vs Enemy
  - Player vs Enemy
  - Enemy Projectile vs Player

**Code Quality: 6/10**
- Clean ECS queries
- Event-driven architecture
- Good separation of concerns
- BUT: No spatial partitioning (O(n²) brute force)
- BUT: Hardcoded to circles only

**Extensibility: 3/10**
- `Collider` is just `{ radius: f32 }`
- Would need moderate refactoring to support other shapes

**Suitability for shmup: 8/10**
- Genre-appropriate (most shmups use circle collision)
- Performs well for typical enemy/bullet counts
- Predictable for players

---

## Future Enhancements

### Easy Win: Ellipse Collision
Add support for elongated sprites (like ScoutSting at 144×244):

```rust
pub struct Collider {
    pub radius_x: f32,
    pub radius_y: f32,
}
```

Ellipse-ellipse collision is still just a distance check with axis scaling.

### Performance: Spatial Partitioning
If you hit 100+ enemies/bullets simultaneously, add spatial grid or quadtree to avoid O(n²) checks.

### Precision: Capsule Collision
For very elongated ships or precise hitboxes:

```rust
pub enum ColliderShape {
    Circle { radius: f32 },
    Capsule { radius: f32, length: f32 },
}
```

Capsule collision is more complex but still reasonably fast.
