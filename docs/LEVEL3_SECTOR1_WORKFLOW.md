# Level 3 Sector 1: Space City - Delegated Workflow

## Overview
Creating "Space City Descent" - the opening sector of Level 3 where player flies through a dense urban megastructure.

---

## Agent 1: Tileset Generator Agent

**Input:**
- Theme: "Space city megastructure"
- Style: "Blade Runner industrial, dark blue-gray metal, cyan/orange lights"
- Grid: 8×3 tiles at 64×64 pixels

**Task:**
1. Generate spritesheet via Gemini
2. Create tileset definition YAML
3. Document tile purposes

**Output:**
- `assets/tilesets/space_city_sheet.png` ✓ (Done - 1.7MB)
- `assets/tilesets/space_city.yaml` (Next)

**Status:** 🟡 50% complete (spritesheet generated, need YAML definition)

---

## Agent 2: Structure Pattern Agent

**Input:**
- Available tiles from space_city tileset
- Requirements: "Building walls on sides, landing platforms crossing"

**Task:**
Define reusable structure patterns:
1. `building_wall_left` - Vertical buildings on left margin
2. `building_wall_right` - Vertical buildings on right margin
3. `landing_platform` - Horizontal platforms crossing playable area
4. `city_tower` - Tall structures in background

**Output:**
- `assets/patterns/city_building_wall.yaml`
- `assets/patterns/city_landing_platform.yaml`
- `assets/patterns/city_tower.yaml`

**Status:** 🔴 Waiting for Agent 1

---

## Agent 3: Background Composer Agent

**Input:**
- Theme: "Dense city skyline at night"
- Mood: "Urban, neon-lit, busy"

**Task:**
Design parallax background layers:
1. DeepSpace: Dark city skyline silhouettes (static)
2. FarField: Distant skyscrapers with tiny lights (10% scroll)
3. MidDistance: Closer buildings with neon signs (40% scroll)
4. Atmospheric: Light beams, fog, flying cars (60% scroll)

**Output:**
- `assets/backgrounds/city_skyline_night.yaml`

**Status:** 🔴 Waiting for Agent 1

---

## Agent 4: Sector Composer Agent

**Input:**
- Available: space_city tileset
- Available: Patterns from Agent 2
- Available: Background from Agent 3
- Requirements: "First sector, 5000 units, introduce city theme gradually"

**Task:**
Create complete sector composition:
1. Structure timeline (when/where buildings appear)
2. Decoration placement (neon signs, lights, billboards)
3. Enemy waves (city defense drones)
4. Pickup placement

**Output:**
- `assets/sectors/level3_sector1_city_descent.yaml`

**Status:** 🔴 Waiting for Agents 2-3

---

## Agent 5: Margin Decorator Agent (NEW!)

**Input:**
- Sector definition from Agent 4
- Theme: "Urban atmosphere"

**Task:**
Populate the **side margins** (outside playable area):
1. Left margin (-400 to -250): Buildings, billboards, traffic
2. Right margin (250 to 400): Buildings, landing pads, lights
3. All at Z-depth -5.0 (behind gameplay, in front of far parallax)
4. Scroll at 100% (same as structures)

**Output:**
- Extended sector YAML with `margin_decorations` section

**Status:** 🔴 Waiting for Agent 4

---

## Current Workflow State

```
[✓] Agent 1: Tileset Generator    → 50% (spritesheet done)
[⏳] Agent 2: Pattern Designer     → 0% (blocked)
[⏳] Agent 3: Background Composer  → 0% (blocked)
[⏳] Agent 4: Sector Composer      → 0% (blocked)
[⏳] Agent 5: Margin Decorator     → 0% (blocked)
```

---

## Answering Your Question: Side Margins

**How to add structures outside playable area?**

Add a new section to sector YAML:

```yaml
# In sector file
margin_decorations:
  left_margin:
    x_range: [-400, -250]  # Outside playable area
    structures:
      - type: building_wall
        from: 0
        to: 5000
        decorations:
          - type: neon_signs
            interval: 200
          - type: windows_lit
            density: high
      - type: traffic_lane
        from: 0
        to: 5000
        vehicles: flying_cars

  right_margin:
    x_range: [250, 400]
    structures:
      - type: building_wall
        from: 0
        to: 5000
      - type: landing_pad
        at: 2000
      - type: billboard
        at: [1000, 3000, 4500]
```

**Key properties of margin decorations:**
- **X position:** Fixed outside playable area
- **Scroll speed:** 100% (same as gameplay structures)
- **Z-depth:** -5.0 to -3.0 (behind gameplay, in front of distant parallax)
- **Purpose:** Visual richness, reinforce setting, frame the action
- **No collision:** Player can't reach them

This gives us Tyrian-style side architecture that makes the world feel dense!

---

## Next Steps

**I need delegation decision from you:**

Should I:
1. **Complete Agent 1** (finish tileset YAML definition)?
2. **Spawn all 5 agents in parallel** with the current state and let them work?
3. **Mock the pipeline** (write example YAMLs showing what each agent would produce)?

Which approach demonstrates the system best?
