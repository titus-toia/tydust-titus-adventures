# Level Architecture V4: Tileset-Based Design

## The Core Insight

Classic shmups use **TILESETS** to build structures. The tiles:
- Are seamlessly tileable (edges match)
- Come in variants (corner, edge, center, damaged)
- Create SOLID ARCHITECTURE that defines playable space
- Have consistent lighting direction

## The Five Mini-Systems

### System 1: Tileset Definition (`/tilesets/`)

A tileset is a spritesheet with defined tile regions and connection rules.

```yaml
# tilesets/industrial_station.yaml
name: Industrial Station
tile_size: 64  # pixels
spritesheet: industrial_station_sheet.png

tiles:
  # Wall tiles with auto-tile positions
  wall_tl: { x: 0, y: 0 }      # top-left corner
  wall_t:  { x: 1, y: 0 }      # top edge
  wall_tr: { x: 2, y: 0 }      # top-right corner
  wall_l:  { x: 0, y: 1 }      # left edge
  wall_c:  { x: 1, y: 1 }      # center (filled)
  wall_r:  { x: 2, y: 1 }      # right edge
  wall_bl: { x: 0, y: 2 }      # bottom-left corner
  wall_b:  { x: 1, y: 2 }      # bottom edge
  wall_br: { x: 2, y: 2 }      # bottom-right corner

  # Platform tiles
  plat_l:  { x: 3, y: 0 }
  plat_c:  { x: 4, y: 0 }
  plat_r:  { x: 5, y: 0 }

  # Decoration tiles
  pipe_h:  { x: 3, y: 1 }      # horizontal pipe
  pipe_v:  { x: 4, y: 1 }      # vertical pipe
  vent:    { x: 5, y: 1 }
  light:   { x: 3, y: 2 }
  grate:   { x: 4, y: 2 }

# Auto-tile rules (which tiles connect to which)
autotile:
  wall:
    corners: [wall_tl, wall_tr, wall_bl, wall_br]
    edges: [wall_t, wall_l, wall_r, wall_b]
    center: wall_c
```

**Agent Responsibility:** Tileset Generator
- Prompts Gemini for tileset spritesheet
- Defines tile cutting grid
- Specifies auto-tile connection rules

---

### System 2: Structure Patterns (`/patterns/`)

Reusable structure templates built from tiles.

```yaml
# patterns/corridor.yaml
name: Corridor
description: Two parallel walls with playable space between

parameters:
  width: narrow | standard | wide  # maps to tile counts
  left_wall: bool
  right_wall: bool

# Width mappings (in tiles)
widths:
  narrow: { gap: 4, wall: 2 }
  standard: { gap: 6, wall: 2 }
  wide: { gap: 8, wall: 3 }

structure:
  # Relative positions from left edge
  left_wall:
    tiles: [wall_tl, wall_t, wall_tr]  # top cap
    repeat: [wall_l, wall_c, wall_r]   # vertical fill
    end: [wall_bl, wall_b, wall_br]    # bottom cap
    position: 0

  right_wall:
    # Mirror of left
    position: gap + wall  # dynamic based on width
```

```yaml
# patterns/platform_bridge.yaml
name: Platform Bridge
description: Horizontal platform crossing the playable area

structure:
  platform:
    tiles: [plat_l, plat_c..., plat_r]
    width: spans_gap
    y_position: variable
```

**Agent Responsibility:** Structure Pattern Agent
- Creates reusable patterns from tilesets
- Defines parameterization (width, variants)

---

### System 3: Parallax Backgrounds (`/backgrounds/`)

**Completely separate from gameplay structures.**

```yaml
# backgrounds/red_nebula_industrial.yaml
name: Red Nebula Industrial
mood: danger, industrial

layers:
  - layer: DeepSpace
    assets:
      - sprite: nebula_red_danger.png
        position: centered
        alpha: 0.6
    scroll: 0%

  - layer: FarField
    assets:
      - sprite: distant_planet.png
        position: [-300, 0]
      - sprite: station_silhouette_distant.png
        position: [200, 0]
    scroll: 10%

  - layer: MidDistance
    procedural:
      type: gas_wisps
      color: orange
      density: low
    scroll: 40%
```

**Agent Responsibility:** Parallax Background Agent
- Designs mood-appropriate background layers
- Specifies static + procedural elements
- NO gameplay collision - pure visual

---

### System 4: Sector Composition (`/sectors/`)

A sector is a complete level segment combining all systems.

```yaml
# sectors/station_approach.yaml
name: Station Approach
length: 8000  # game units
tileset: industrial_station

# Background (visual only, no collision)
background: red_nebula_industrial

# Structure timeline (defines playable space)
structures:
  - segment: 0-2000
    type: open_space
    # No tiles - just parallax background visible

  - segment: 2000-5000
    type: corridor
    pattern: corridor
    params:
      width: standard
      left_wall: true
      right_wall: true
    # Optional per-segment variations
    variations:
      - at: 2500
        right_wall: false  # gap in right wall
      - at: 3500-4000
        width: narrow      # corridor narrows

  - segment: 5000-6500
    type: platforms
    pattern: platform_sequence
    params:
      count: 4
      spacing: 350
      alternating: true  # left-right-left-right

  - segment: 6500-8000
    type: open_space

# Decorations (attach to structures)
decorations:
  - on: corridor.left_wall
    place: lights
    interval: 200

  - on: corridor.right_wall
    place: vents
    interval: 400
    particles: steam

# Gameplay elements
enemies:
  - at: 500
    wave: scout_patrol
  - at: 2500
    wave: corridor_ambush
  # ...

pickups:
  - at: 1000
    type: weapon
  # ...
```

**Agent Responsibility:** Sector Composer
- Combines tileset + patterns + background
- Designs structure progression
- Places decorations, enemies, pickups

---

### System 5: Tile Renderer (Rust code)

The runtime system that:
1. Loads tileset spritesheets
2. Cuts tiles on startup
3. Renders structure segments using tiles
4. Handles auto-tiling (selecting correct corner/edge pieces)

```rust
// Conceptual structure
struct Tileset {
    texture: Handle<Image>,
    tile_size: u32,
    tiles: HashMap<String, TileRegion>,
}

struct TileRegion {
    x: u32,
    y: u32,
    // UV coordinates calculated from x, y, tile_size
}

struct StructureSegment {
    pattern: String,
    start_distance: f32,
    end_distance: f32,
    // Rendered as tile entities when segment enters view
}
```

---

## Spatial Model (Critical!)

The level is a **2D space**. The viewport is a window into it.

```
PARALLAX LAYERS (visual only, varied scroll speeds):
├── DeepSpace (-9.5)    - Static backdrop, 0% scroll
├── FarField (-9.0)     - Distant objects, 10% scroll
├── MidDistance (-7.0)  - Atmospheric, 40% scroll
└── Foreground (2.5)    - Speed lines, 150% scroll

GAMEPLAY LAYER (100% scroll, z=0):
├── Structures at various X,Y positions
│   - Building at x=-300 (left side)
│   - Platform at x=0, y=2000 (center)
│   - Building at x=300 (right side)
├── Enemies, Player, Projectiles, Pickups
└── All entities scroll together at 100%
```

**There are no "margins" or special side areas.** A corridor is just structures placed at x=-300 and x=300. Open space is the absence of structures. It's all just X,Y placement in a scrolling 2D world.

---

## Workflow for Creating a New Level Theme

### Step 1: Generate Tileset
```
Human: "Create an asteroid cave tileset"
Agent: [Prompts Gemini for spritesheet]
Agent: [Defines tile cutting and auto-tile rules]
Output: tilesets/asteroid_cave.yaml + .png
```

### Step 2: Define Patterns
```
Human: "We need narrow tunnels and open caverns"
Agent: [Creates pattern definitions using asteroid tiles]
Output: patterns/asteroid_tunnel.yaml, patterns/asteroid_cavern.yaml
```

### Step 3: Create Background
```
Human: "Deep space visible through cave gaps"
Agent: [Designs parallax layers]
Output: backgrounds/deep_space_cave.yaml
```

### Step 4: Compose Sectors
```
Human: "Design level 3 - asteroid mining facility"
Agent: [Combines tileset + patterns + background]
Agent: [Places structure segments, decorations, enemies]
Output: sectors/asteroid_mining_01.yaml
```

---

## Benefits of This Architecture

1. **Separation of Concerns** - Visual parallax vs gameplay structures
2. **Reusability** - Tilesets and patterns used across sectors
3. **Delegatable** - Each system is a clear agent responsibility
4. **Scalable** - Add new tilesets/patterns without code changes
5. **Authentic** - Matches how classic shmups actually worked
6. **Efficient** - Tiles are small, repeated efficiently by GPU
