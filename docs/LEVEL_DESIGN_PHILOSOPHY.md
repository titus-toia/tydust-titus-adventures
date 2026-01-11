# Level Design Pipeline: Story-First Visual Worldbuilding

## The Philosophy

**Every asset must answer: "What happened here?"**

Not "floating space rock" but "ore processing equipment blown apart in the attack."
Not "random debris" but "escape pod from the mining colony, still broadcasting distress."

This constraint makes everything cohesive and memorable.

---

## The Pipeline (6 Phases)

### Phase 1: The Pilot's Flight Log

Write a **detailed, distance-indexed journal** at 500 GU increments:

```
[DISTANCE: 2500 GU]
ABOVE: Distant nebula shifts from purple to orange-red
LEFT: Massive carrier hull enters frame, 3x screen width, battle-scarred
RIGHT: Nothing - open space, contrast with left density
BELOW: Spherical eco-dome passes underneath, lights flickering
CENTER: Debris field thickens - hull plates, antenna fragments
MOOD: Discovery turning to unease
LAYER FOCUS: MegaStructures (carrier), Gameplay (debris)
```

**Why this format works:**
- Forces spatial thinking (above/below/left/right)
- Captures parallax layering naturally
- Prevents "everything in the center" syndrome
- Creates rhythm (dense left → sparse right → dense below)

### Phase 2: The Asset Manifest

Extract every visual element. Categorize:

```yaml
assets:
  hero_structures:    # Large, unique, define the level
  modular_pieces:     # Reusable, combinable
  atmospheric:        # Background mood setters
  debris:             # Small, scattered detail
```

Each asset needs: name, description, size, layer, quantity, generation strategy.

### Phase 3: Parallax Coherence Map

Map what each layer shows across the ENTIRE level:

```
Layer 0 - DeepSpace (0.15x):     Background nebulae, distant stars
Layer 1 - DeepStructures (0.3x): Station silhouettes, far structures
Layer 2 - MegaStructures (0.5x): Walls, carrier hulls, large landmarks
Layer 3 - Gameplay (1.0x):       Active debris, enemies, pickups
Layer 4 - Foreground (1.3x):     Particle effects, close debris
```

### Phase 3.5: Style Bible & Composition Rules

**Style Bible:**
```yaml
color_palette:
  primary: "#2a2a3a"      # Dark blue-gray metal
  secondary: "#4a4a5a"    # Lighter hull panels
  accent_warm: "#ff6b35"  # Orange warning lights, fire
  accent_cool: "#35a7ff"  # Blue emergency lights, energy
  danger: "#ff3535"       # Red alerts, damage
  glow: "#ffaa00"         # Reactor glow, molten metal

materials:
  - "weathered industrial metal with rivets"
  - "carbon scoring and burn marks"
  - "rust streaks near joints"
  - "glowing cracks from internal heat"
  - "exposed wiring and pipes"

design_language:
  shapes: "angular, functional, brutalist"
  greeble_density: "high - pipes, vents, panels everywhere"
  wear_level: "heavy - this station is old and damaged"
```

**Composition Rules:**
```yaml
tileable:           # Must seamlessly repeat
  - perimeter_wall_segment (vertical)
  - pipe_section (both directions)

connectable:        # Designed to attach
  - wall_left_top → wall_left_mid → wall_left_bottom
  - pipe_elbow + pipe_straight + pipe_junction

composed_from_parts:  # Large structures from pieces
  - carrier_wreck: [hull_front, hull_mid, hull_rear]
  - refinery: [base, tower, pipes, smoke_stack]

standalone:         # Complete single assets
  - escape_pod, cargo_container, asteroid
```

**Generation Order:**
1. Style reference image first (mood board)
2. Hero structure #1 sets the tone
3. Use it as reference for all subsequent generations
4. Tileable pieces with explicit edge requirements
5. Standalone debris last

### Phase 4: Generation Prompts

Template for consistency:
```
[ASSET DESCRIPTION]

Style requirements:
- Industrial mining station aesthetic
- Color palette: dark blue-gray, orange accents, blue emergency lights
- Weathered metal with rivets, carbon scoring, rust streaks
- Angular, brutalist, functional design

Technical requirements:
- [SIZE] PNG with transparent background
- [TILING NOTES if applicable]
- Side-view for vertical shmup
```

### Phase 5: Implementation Order

1. Layer 0 backgrounds (sets mood)
2. Hero structures (defines landmarks)
3. Parallax coherence pass
4. Modular pieces
5. Debris generation
6. YAML integration
7. Playtest pass

### Phase 6: Quality Gates

Before considering a level "done":

- [ ] Can I describe what I'm flying through in one sentence?
- [ ] Do I know where I am relative to the "destination"?
- [ ] Is there visual variety every 30 seconds?
- [ ] Do parallax layers create depth, not confusion?
- [ ] Are there 3+ memorable "landmark" moments?
- [ ] Does the environment telegraph enemy intensity?
- [ ] Would a screenshot at any point look intentional?

---

## Layer Reference (DoodadLayer enum)

| Layer | Z-Depth | Scroll | Use For |
|-------|---------|--------|---------|
| DeepSpace | -9.0 | 0.15x | Nebulae, distant stars |
| DeepStructures | -8.0 | 0.30x | Station silhouettes |
| FarField | -6.0 | 0.40x | Distant debris |
| MegaStructures | -4.0 | 0.50x | Large walls, platforms |
| MidDistance | -2.0 | 0.65x | Medium structures |
| StructureDetails | -1.0 | 0.80x | Attached details |
| NearBackground | -0.5 | 0.90x | Close background |
| Gameplay | 0.0 | 1.00x | Active debris, enemies |
| Foreground | 1.0 | 1.30x | Particles, close FX |

---

## Execution Checklist

For each new level:

1. [ ] Define the journey (where → where, what happened?)
2. [ ] Write flight log (500 GU beats, spatial detail)
3. [ ] Extract asset manifest (categorized)
4. [ ] Map parallax coherence (layer by layer)
5. [ ] Generate style reference
6. [ ] Generate hero structures
7. [ ] Generate modular pieces
8. [ ] Generate atmospheric
9. [ ] Generate debris
10. [ ] Write level YAML
11. [ ] Playtest until intentional

---

## Example Flight Log Beat

```
[DISTANCE: 8500 GU]
ABOVE: Orange nebula dominates upper third, station core glow emerging
LEFT: Perimeter wall continues, defense turret (destroyed) mounted
RIGHT: Carrier wreck section visible - you're flying OVER it
BELOW: Carrier hull stretches beneath, battle damage, fires burning
CENTER: Scattered defense satellite debris, antenna fragments
MOOD: We're inside something now - no turning back
LAYER FOCUS: MegaStructures (walls, carrier), DeepSpace (nebula shift)
NOTE: This is the "corridor established" moment - walls on both sides
```

---

## The Golden Rule

**If you can't explain what the player is looking at and why it's there, don't put it there.**

Every escape pod was someone fleeing.
Every hull breach tells a story.
Every flickering light is a system failing.

The level is not a backdrop - it's evidence of what happened.

---

## Geography-First Level Generation (V3)

**The Problem with Doodad Collections:**
Scattering individual sprites creates "space soup" - random floating debris with no sense of place. Look at Tyrian: vertical towers form corridors, turrets mount on walls, the level has *architecture*.

**The Solution: Structures Define Playable Space**

Instead of "spawn 50 hull fragments scattered," think:
- "This section is a canyon with walls on both sides"
- "Turrets are mounted ON the walls"
- "The player navigates through the architecture"

### Section-Based Design

Each level is a sequence of **Sections**, each with a defined **shape**:

```yaml
sections:
- name: "Canyon Entry"
  start_distance: 7000.0
  end_distance: 12000.0
  shape: canyon          # Walls on both sides
  walls:
    - side: left
      structure_sprite: refinery_wall.png
      x_position: -300.0
      segment_height: 400.0
      mounted:            # Objects ON the wall
        - object_type: turret
          interval: 500.0
          offset: 60.0    # Toward playable area
  obstacles:              # Things crossing the lane
    - obstacle_type: platform
      sprite: pipe.png
      interval: 800.0
  floating:               # MINIMAL debris (not the focus!)
    sprites: [spark.png]
    density: 1.5
```

### Available Shapes

| Shape | Description | Walls |
|-------|-------------|-------|
| `open` | Full width, no constraints | None |
| `left_corridor` | Wall on left, open right | Left only |
| `right_corridor` | Wall on right, open left | Right only |
| `canyon` | Walls on both sides, narrow center | Both |
| `split` | Obstacle in center, two lanes | Center |
| `weave` | Alternating obstacles | Various |

### Mounted Objects (NOT Floating!)

The key insight: objects **attach to structures**, not float randomly.

```yaml
mounted:
- object_type:
    turret:
      sprite: turret.png
      damage: 10.0
  interval: 400.0      # Every 400 GU along the wall
  offset: 60.0         # 60px toward playable area
  jitter: 20.0         # Random variation
```

Types: `turret`, `light`, `vent`, `pipe`, `debris`

### Obstacles (Cross the Lane)

These force the player to navigate:

```yaml
obstacles:
- obstacle_type:
    platform:
      sprite: pipe_cluster.png
      width: 150.0
  interval: 600.0
  interval_jitter: 80.0
```

Types: `platform`, `asteroid`, `conduit`, `barrier`

### Floating Debris (MINIMAL)

This is NOT the main content. Keep `density` low (1-3 per 1000 GU).

```yaml
floating:
  sprites: [spark.png, fragment.png]
  density: 2.0          # NOT 50!
  size_range: [25.0, 45.0]
```

### Generation Order (Structure-First)

1. **Walls** - Continuous segments define the corridor
2. **Mounted Objects** - Attached to walls at intervals
3. **Obstacles** - Cross the playable space
4. **Floating** - Minimal scattered debris (last, least important)

### Example: Tyrian-Style Canyon

```yaml
- name: "Industrial Canyon"
  start_distance: 10000.0
  end_distance: 20000.0
  shape: canyon
  walls:
  - side: left
    structure_sprite: tower_left.png
    x_position: -280.0
    wobble: 20.0
    segment_height: 350.0
    mounted:
    - object_type:
        turret: { sprite: turret.png, damage: 15.0 }
      interval: 400.0
      offset: 50.0
    - object_type:
        vent: { sprite: gas_vent.png }
      interval: 300.0
      offset: 70.0
  - side: right
    structure_sprite: tower_right.png
    x_position: 280.0
    wobble: 20.0
    segment_height: 350.0
    mounted:
    - object_type:
        light: { sprite: beacon.png }
      interval: 350.0
      offset: 40.0
  obstacles:
  - obstacle_type:
      conduit: { sprite: pipe_horizontal.png }
    interval: 700.0
  floating:
    sprites: [spark.png]
    density: 1.0
```

This creates:
- Continuous walls on both sides
- Turrets/vents mounted on walls (not floating!)
- Horizontal pipes crossing between walls
- Minimal floating sparks for atmosphere

**Result: A PLACE, not a sprite collection.**

---

## File Locations

- **Level YAML**: `assets/level-defs/level{N}.yaml`
- **Data Structures**: `src/level/mod.rs` (Section, WallDefinition, MountedObject, etc.)
- **Generation**: `LevelDataV3.to_level_data()` in `src/level/mod.rs`
- **Loading**: `src/systems/level.rs` (auto-detects V1/V2/V3)

---

## Quick Reference: V1 vs V2 vs V3

| Version | Philosophy | Main Content |
|---------|-----------|--------------|
| V1 | Manual doodad list | Individual `DoodadSpawn` entries |
| V2 | Zone-based scatter | `zones` with `doodad_pools` |
| V3 | Geography-first | `sections` with `walls` + `mounted` |

**Use V3 for new levels.** It's the only one that creates places, not collections.
