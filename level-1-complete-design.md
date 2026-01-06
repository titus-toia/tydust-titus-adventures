# Level 1: "First Contact" - Complete Design Plan

## 1. Level Duration & Pacing (5-8 minute target)

### Current State
- Total distance: 24,802 GU
- 5 phases with varying scroll speeds (65-195 GU/s)
- Estimated time: ~3-4 minutes (too short)

### Target Structure (6-7 minutes)
```
Phase 1: The Approach (0:00-1:30)  - 1500 GU at 60 GU/s
  Visual: Distant station silhouettes, sparse debris
  Enemies: None or single scout (tutorial)
  Music: Ambient, mysterious

Phase 2: Outer Perimeter (1:30-3:00) - 2000 GU at 80 GU/s
  Visual: Communication arrays, solar farms, warning beacons
  Enemies: Light patrols (scouts)
  Music: Tension builds

Phase 3: Industrial Zone (3:00-5:00) - 3000 GU at 100 GU/s
  Visual: MEGA STRUCTURES - refineries, ore processors, cargo bays
  Enemies: Heavy resistance (fighters, turrets)
  Music: Combat intensity

Phase 4: Core Breach (5:00-6:00) - 1500 GU at 120 GU/s
  Visual: Reactor cores, defense grids, damaged sections
  Enemies: Elite units, environmental hazards
  Music: Peak intensity

Phase 5: Boss Arena (6:00-7:00) - 1500 GU at 60 GU/s (slowdown)
  Visual: Command center or hangar bay
  Enemies: Boss + minions
  Music: Boss theme
```

## 2. Story Arc: "Abandoned Station Awakens"

### Narrative Beats

**Act 1: Discovery (Phase 1)**
- Radio: "Approaching coordinates. Station should be here..."
- Visual: Massive asteroid station emerges from darkness
- Radio: "Reading zero life signs. Looks abandoned."
- Tutorial: Basic controls

**Act 2: First Contact (Phase 2)**
- Visual: Pass by outer defenses, everything dark/inactive
- Radio: "Wait... sensor contact. Single craft."
- First scout enemy appears (confused AI?)
- Radio: "Unidentified craft! All units to—" [cuts off]
- More scouts appear, station lights flicker on

**Act 3: Station Awakens (Phase 3)**
- Visual: Industrial sections power up, orange warning lights
- Radio: "The whole station is coming online! Defensive protocol engaged!"
- Heavy combat through refinery canyon
- Structures transition: dark → glowing with activity
- Radio: "They're not defending the station... the station IS defending!"

**Act 4: Into the Core (Phase 4)**
- Visual: Flying through reactor sections, energy arcs, explosions
- Radio: "Core breach detected! All defensive batteries destroyed!"
- Environmental hazards: energy beams from structures
- Radio: "Something big is launching from the hangar..."

**Act 5: Guardian (Phase 5)**
- Boss emerges: Either station AI core or massive guardian ship
- Arena: Large hangar or command deck
- Victory: Station powers down, safe passage through

## 3. Visual Asset Strategy

### Tileset Philosophy
**Tyrian-style modular structures:**
- Vertical wall tiles (128x256) that tile seamlessly
- Cap pieces (top/bottom)
- Connector pieces (bridges between walls)
- Detail overlays (turrets, pipes, windows)

### Asset Categories

#### A. Background Megastructures (z=-8 to z=-6)
**Purpose:** Create canyon/corridor feeling

1. **Outer Perimeter Set**
   - `perimeter_wall_left_mid.png` (128x256, tileable)
   - `perimeter_wall_right_mid.png` (128x256, tileable)
   - `perimeter_solar_array.png` (300x400, one-piece)
   - `perimeter_comm_tower.png` (200x600, one-piece)

2. **Industrial Core Set** (THE MAIN ATTRACTION)
   - `refinery_wall_left_mid.png` (128x256, tileable) - Pipes, vents, ore chutes
   - `refinery_wall_right_mid.png` (128x256, tileable) - Processing units
   - `refinery_furnace.png` (400x800, one-piece) - Glowing orange smelter
   - `ore_processor.png` (350x600, one-piece) - Mechanical crusher
   - `cargo_bay.png` (400x500, one-piece) - Storage containers

3. **Reactor Core Set**
   - `reactor_wall_mid.png` (128x256, tileable) - Energy conduits
   - `reactor_core.png` (500x1000, one-piece) - Glowing blue/purple core
   - `energy_distributor.png` (300x400, one-piece) - Capacitor banks

4. **Boss Arena**
   - `hangar_wall_damaged.png` (128x256) - Battle damage
   - `hangar_ceiling.png` (800x200) - Industrial ceiling
   - `command_deck.png` (600x400) - If station AI boss

#### B. Foreground Details (z=-4 to z=-3)
**Purpose:** Add depth, breakup repetition

- `detail_spotlight.png` (64x96) - Searching beam
- `detail_warning_light.png` (32x32) - Blinking orange
- `detail_vent_steam.png` (64x128) - Animated-looking steam
- `detail_damaged_panel.png` (96x96) - Battle damage
- `detail_turret_inactive.png` (80x80) - Dormant turret
- `detail_turret_active.png` (80x80) - Armed turret (glowing)

#### C. Transition Pieces
**Purpose:** Connect sections, narrative markers

- `transition_airlock.png` (200x300) - Marks section change
- `transition_bridge.png` (400x100) - Connects left/right walls
- `transition_blast_door.png` (300x400) - Opening/damaged

### Visual State Changes
Same structures with different states:
- `refinery_inactive.png` → `refinery_active.png` (lights turn on)
- `turret_dormant.png` → `turret_armed.png` (powers up)

## 4. Level YAML Structure

### Phase Organization
```yaml
phases:
- name: The Approach
  distance: 0-1500
  scroll_speed: 60
  structures: perimeter_distant

- name: Outer Perimeter
  distance: 1500-3500
  scroll_speed: 80
  structures: perimeter_close

- name: Industrial Zone
  distance: 3500-6500
  scroll_speed: 100
  structures: refinery_canyon

- name: Core Breach
  distance: 6500-8000
  scroll_speed: 120
  structures: reactor_danger

- name: Boss Arena
  distance: 8000-9500
  scroll_speed: 60
  structures: hangar_boss
```

### Doodad Density Strategy
- Phase 1: 1-2 structures per 500 GU (sparse, distant)
- Phase 2: 2-3 structures per 500 GU (building density)
- Phase 3: 4-6 structures per 500 GU (CANYON EFFECT - walls on both sides)
- Phase 4: 3-4 structures per 500 GU (chaos, some destroyed)
- Phase 5: Arena pieces (ceiling, walls, boss platform)

## 5. Asset Generation Plan

### Batch 1: Outer Perimeter (Phase 2)
Generate with Gemini:
1. Perimeter wall tiles (dark, inactive, warning stripes)
2. Solar array (dormant panels)
3. Communication tower (antenna, lights off)

### Batch 2: Industrial Core (Phase 3) - PRIORITY
Generate with Gemini:
1. Refinery wall tiles (pipes, industrial detail)
2. Furnace structure (glowing orange interior)
3. Ore processor (mechanical crusher)
4. Cargo bay (stacked containers)

### Batch 3: Reactor Core (Phase 4)
Generate with Gemini:
1. Reactor wall tiles (energy conduits)
2. Reactor core structure (glowing blue/purple)
3. Energy distributor

### Batch 4: Details & Polish
Generate with Gemini:
1. Detail sprites (turrets, lights, vents)
2. Transition pieces (airlocks, bridges)
3. Boss arena pieces

## 6. Implementation Steps

1. **Update phase distances** in level1.yaml for 6-7 minute duration
2. **Generate Batch 1** (perimeter assets)
3. **Place Batch 1** in YAML with proper z-depths and spacing
4. **Test pacing** - does it feel right?
5. **Generate Batch 2** (industrial - the hero assets)
6. **Create "canyon"** - dense wall placement on left+right
7. **Add details** for visual variety
8. **Generate Batch 3** (reactor core)
9. **Polish with transitions** between sections
10. **Boss arena** setup
11. **Radio chatter** updates to match narrative
12. **Final playtesting** and tweaking

## 7. Success Criteria

- [ ] Level duration: 6-7 minutes
- [ ] Clear visual progression through station sections
- [ ] "Canyon" effect in industrial zone - flying between structures
- [ ] Narrative makes sense from radio chatter + visuals
- [ ] Enemy density matches visual intensity
- [ ] No visual repetition - structures vary enough
- [ ] Boss arena feels distinct and climactic
- [ ] Player can navigate without feeling lost

## Next Action
Start with **Batch 1: Outer Perimeter assets** - these set the tone for the abandoned station before it wakes up.
