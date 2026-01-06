# Z-Layer Architecture

Visual depth layering for Tydust. Lower z = further back.

## Layer Stack

| z-depth | Layer Name | Speed | Contents |
|---------|------------|-------|----------|
| -10.0 | Backdrop | 0.1x | Dark tile fill |
| -9.5 | DeepCosmos | 0.15x | Nebulae, distant planets, galaxy arms |
| -9.0 | Stars | 0.2-0.4x | Individual star sprites (varied speeds) |
| -8.0 | DeepStructures | 0.3x | Space hulks, distant stations, dead capital ships |
| -6.0 | MegaStructures | 0.5x | Asteroid station main bodies, large setpieces |
| -4.0 | StructureDetails | 0.7x | Attached modules, docking arms, gantries |
| -3.0 | NearDebris | 1.0x | Floating derelict hulls, close wreckage |
| -0.5 | Gameplay | varies | Interactive obstacles (asteroids, satellites) |
| 0.5-1.0 | Actors | 1.0x | Player, enemies, projectiles |
| 2.5 | Foreground | 2.5x | Fast-passing debris, dust clouds, sparks |

## Layer Descriptions

### DeepCosmos (-9.5)
Atmospheric backdrop elements. Nebulae, distant planets, galaxy features. Should feel impossibly far away. Very slow scroll creates depth.

### DeepStructures (-8.0)
Distant large objects that establish setting. Dead battleships on the horizon, distant space stations, asteroid fields. Silhouettes work well here.

### MegaStructures (-6.0)
The main setpiece layer. Asteroid station modules, carved rock facilities, industrial rigs. These can span multiple screen heights (2000-5000 GU tall). Player flies "alongside" these.

### StructureDetails (-4.0)
Attached to or near mega structures. Docking arms, communication arrays, fuel pipes, scaffolding. Adds detail and scale to the main structures.

### NearDebris (-3.0)
Floating closer to the player. Small derelict hulls, escape pods, cargo that drifted away from structures. Creates layered depth between structures and gameplay.

## Level Themes

### Level 1: Asteroid Station
- DeepCosmos: Purple/blue nebulae
- DeepStructures: Distant mining rigs, dead freighters
- MegaStructures: Carved asteroid facilities, docking bays, ore processors
- StructureDetails: Mining equipment, cargo cranes, landing pads
- NearDebris: Drifting containers, broken hull plates

### Level 2: (TBD)
### Level 3: (TBD)

## Multi-Screen Structures

For structures taller than one screen (1302 GU), use explicit size in YAML:

```yaml
- sprite: asteroid_station_core.png
  position: [-200.0, 2000.0]
  size: [400, 2500]        # width, height in GU
  z_depth: -6.0            # explicit z override
  layer: mega_structure
  spawn_distance: 5000.0
```

## Speed Multipliers

Speed determines parallax effect. Lower = slower = feels further away.

- 0.1-0.2x: Cosmic scale (nebulae, stars)
- 0.3-0.5x: Large distant objects (hulks, mega structures)
- 0.7-1.0x: Mid-ground (details, near debris)
- 1.5-3.0x: Foreground rushing past
