# Implementation Roadmap V4

## Phase 1: Tileset Infrastructure (Foundation)

### Task 1.1: Tileset Data Structures
**Delegate to:** Code Agent
**Files:** `src/tileset/mod.rs` (new module)

```rust
// Core structures needed
pub struct Tileset {
    pub name: String,
    pub tile_size: u32,
    pub texture: Handle<Image>,
    pub tiles: HashMap<String, UVRect>,
}

pub struct UVRect {
    pub min: Vec2,  // UV coordinates 0-1
    pub max: Vec2,
}

pub struct TilesetDefinition {
    // Loaded from YAML
    pub name: String,
    pub tile_size: u32,
    pub spritesheet: String,
    pub tiles: HashMap<String, TilePosition>,
}

pub struct TilePosition {
    pub x: u32,  // Grid position, not pixels
    pub y: u32,
}
```

**Deliverable:** Tileset loading from YAML, UV calculation, texture atlas access

---

### Task 1.2: Create First Tileset Spritesheet
**Delegate to:** Asset Agent (uses Gemini)
**Output:** `assets/tilesets/industrial_station_sheet.png`

**Prompt for Gemini:**
```
Create a 384x192 pixel tileset spritesheet for a sci-fi space station.
Grid: 6 columns x 3 rows of 64x64 pixel tiles.

Row 1 (walls): top-left corner, top edge, top-right corner,
               bottom-left corner, bottom edge, bottom-right corner
Row 2 (walls): left edge, center fill, right edge,
               inner corner TL, inner corner TR, damaged variant
Row 3 (details): horizontal pipe, vertical pipe, vent grate,
                 warning light, panel, conduit

Style: Industrial metal, consistent top-left lighting,
       dark blue-gray palette with orange accent lights.
       Edges must tile seamlessly.
```

**Deliverable:** Spritesheet PNG + `tilesets/industrial_station.yaml`

---

### Task 1.3: Tile Cutting System
**Delegate to:** Code Agent
**Files:** `src/tileset/cutter.rs`

System that:
1. Loads spritesheet texture
2. Reads tileset YAML definition
3. Calculates UV coordinates for each named tile
4. Provides `get_tile_uvs(tileset, tile_name) -> UVRect`

---

## Phase 2: Structure Rendering

### Task 2.1: Structure Segment Component
**Delegate to:** Code Agent
**Files:** `src/components/structures.rs`

```rust
#[derive(Component)]
pub struct TileStructure {
    pub tileset: String,
    pub pattern: StructurePattern,
    pub start_y: f32,
    pub end_y: f32,
}

pub enum StructurePattern {
    LeftWall { width: u32 },
    RightWall { width: u32 },
    Corridor { gap: u32, wall_width: u32 },
    Platform { x: f32, width: u32 },
}
```

---

### Task 2.2: Tile Mesh Generation
**Delegate to:** Code Agent
**Files:** `src/systems/tile_renderer.rs`

For each structure segment in view:
1. Calculate which tiles needed based on pattern
2. Apply auto-tiling rules (corners, edges, centers)
3. Generate mesh with correct UVs from tileset
4. Render as single batched draw call

**Key:** Tiles scroll at 100% gameplay speed (not parallax!)

---

### Task 2.3: Auto-Tiling Logic
**Delegate to:** Code Agent
**Files:** `src/tileset/autotile.rs`

Given a structure pattern, determine which tile variant for each position:
- Corners for wall ends
- Edges for wall sides
- Centers for filled areas
- Handle transitions (wall to gap)

---

## Phase 3: Sector Definition Format

### Task 3.1: Sector YAML Schema
**Delegate to:** Design Agent
**Files:** `src/level/sector.rs` + schema docs

Define the complete sector format:
```yaml
sector:
  name: string
  length: number
  tileset: string
  background: string

  structures:
    - segment: "start-end"
      pattern: corridor|left_wall|right_wall|platforms
      params: {...}

  decorations:
    - on: structure_ref
      type: lights|vents|pipes
      interval: number

  enemies: [...]
  pickups: [...]
```

---

### Task 3.2: Sector Loader
**Delegate to:** Code Agent
**Files:** `src/level/sector_loader.rs`

Replace current level loading with sector-based system:
1. Load sector YAML
2. Parse structure timeline
3. Initialize tile structures (spawn when entering view)
4. Set up background layers
5. Queue enemy waves and pickups

---

## Phase 4: Background System Separation

### Task 4.1: Refactor Parallax System
**Delegate to:** Code Agent
**Files:** `src/systems/parallax.rs` (refactor)

Clean separation:
- Parallax layers: visual only, variable scroll speeds
- Structure layer: gameplay, 100% scroll, uses tiles
- Remove any parallax-structure coupling

---

### Task 4.2: Background Theme Definitions
**Delegate to:** Design Agent
**Files:** `assets/backgrounds/*.yaml`

Create themed background presets:
- `red_nebula.yaml` - danger zones
- `blue_nebula.yaml` - calm areas
- `asteroid_field.yaml` - rocky sections
- `deep_space.yaml` - open void

Each defines which parallax assets at which layers.

---

## Phase 5: Decoration System

### Task 5.1: Structure Attachment Points
**Delegate to:** Code Agent
**Files:** `src/systems/decorations.rs`

Decorations attach to tile structures:
- Lights at intervals on walls
- Vents with particle emitters
- Pipes connecting structures
- Hazard stripes at boundaries

Decorations move WITH their parent structure (same scroll speed).

---

## Phase 6: Level Composition Tools

### Task 6.1: Sector Composer Agent Prompt
**Delegate to:** Prompt Engineer
**Output:** Reusable prompt template

Create a prompt that lets Claude compose sectors:
```
Given:
- Available tilesets: [list]
- Available patterns: [list]
- Available backgrounds: [list]
- Level requirements: [theme, difficulty, length]

Output:
- Complete sector YAML
- Enemy wave timing
- Pickup placement
- Decoration placement
```

---

## Execution Order

```
Week 1: Phase 1 (Tileset Infrastructure)
  ├── 1.1 Data structures
  ├── 1.2 First tileset from Gemini
  └── 1.3 Tile cutting

Week 2: Phase 2 (Structure Rendering)
  ├── 2.1 Structure components
  ├── 2.2 Tile mesh generation
  └── 2.3 Auto-tiling

Week 3: Phase 3-4 (Sector System)
  ├── 3.1 Sector YAML schema
  ├── 3.2 Sector loader
  └── 4.1-4.2 Background separation

Week 4: Phase 5-6 (Polish)
  ├── 5.1 Decorations
  └── 6.1 Composition tools
```

---

## File Structure After Implementation

```
src/
├── tileset/
│   ├── mod.rs          # Tileset struct, loading
│   ├── cutter.rs       # UV calculation
│   └── autotile.rs     # Auto-tile logic
├── level/
│   ├── mod.rs          # Existing (keep V1-V3 compat)
│   ├── sector.rs       # New sector format
│   └── sector_loader.rs
├── systems/
│   ├── tile_renderer.rs  # New: renders tile structures
│   ├── parallax.rs       # Refactored: visual only
│   ├── decorations.rs    # New: structure attachments
│   └── ...existing...
└── components/
    ├── structures.rs     # New: tile structure components
    └── ...existing...

assets/
├── tilesets/
│   ├── industrial_station.yaml
│   ├── industrial_station_sheet.png
│   ├── asteroid_cave.yaml
│   └── asteroid_cave_sheet.png
├── backgrounds/
│   ├── red_nebula.yaml
│   ├── deep_space.yaml
│   └── ...
├── sectors/
│   ├── level1_sector1.yaml
│   ├── level1_sector2.yaml
│   └── ...
└── parallax/
    └── ...existing assets...
```

---

## Success Criteria

1. **Visual:** Walls look like Tyrian - solid, tiled, architectural
2. **Separation:** Parallax doesn't affect gameplay; structures do
3. **Delegatable:** Each task is self-contained, clear inputs/outputs
4. **Composable:** New sectors created from YAML without code changes
5. **Performant:** Tile batching, minimal draw calls
