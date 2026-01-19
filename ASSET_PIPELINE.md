# Asset Generation & QA Pipeline

## Visual Style Guide

### The Aesthetic: "Vibe-Coded Hell Simulator"

Our game is a grimdark industrial space setting with a dark, oppressive atmosphere. All assets should reflect this aesthetic:

**Primary Style: Pre-Rendered 3D Sprites**
- 90s-era pre-rendered look (Donkey Kong Country, Killer Instinct aesthetic)
- Smooth gradients and photorealistic materials
- Crisp highlights and dramatic shadows
- Glossy/metallic surfaces with reflections and rim lighting
- Hard shadows for depth and menace
- Weathered, battle-scarred industrial materials

**Acceptable Variations:**
- Occasional pixel-art styles are fine for variety and charm (UI elements, doodads, stylistic contrast)
- The vibe matters more than perfect consistency

### Prompt Keywords for Pre-Rendered Aesthetic

When generating assets, include these keywords to achieve the desired look:
- `pre-rendered 3D sprite`
- `glossy metallic materials`
- `dramatic rim lighting`
- `hard shadows`
- `photorealistic textures`
- `industrial materials`
- `crisp highlights`

### Material Guidance by Asset Type

| Type | Style | Example Keywords |
|------|-------|------------------|
| Structures | Weathered metal, rust, industrial grime | "industrial grime", "weathered steel", "rusted panels" |
| Ships | Sleek hulls with reflections, battle damage | "battle-scarred", "reflective hull", "damaged plating" |
| Effects | Sharp, luminous particles | "crisp particles", "luminous glow" |
| Environment | Dark, oppressive, atmospheric | "grim industrial", "dark oppressive atmosphere" |

### Directional Convention for Ships

**CRITICAL: All ship sprites must follow a consistent facing direction for proper rotation:**

- **Enemy Ships** → Face DOWN (South) ↓
  - Assets: `assets/sprites/raiders/*`
  - When rotated 0°, ship points downward (toward bottom of screen)
  - This allows clean algorithmic rotation for all 8/16 directions

- **Player Ships** → Face UP (North) ↑
  - Assets: `assets/sprites/ships/*`
  - When rotated 0°, ship points upward (toward top of screen)
  - Maintains player-friendly forward direction

**Why this matters:**
- Game engine rotates sprites by angle around Z-axis
- If ships face wrong direction at 0°, rotation calculations are offset
- Inconsistent directions break visual coherence across enemy types
- This is a **convention, not an engine requirement** - but enforces consistency

**When generating new ships:**
- Include in prompt: `"facing downward"` or `"nose pointing down"` for enemies
- Include in prompt: `"facing upward"` or `"nose pointing up"` for player ships
- Verify in final asset that top of ship points in correct direction

**If regenerating existing ships:**
```bash
# Rotate 90° CW:  (facing east → facing south)
python3 -c "from PIL import Image; img = Image.open('ship.png'); img.rotate(-90, expand=True).save('ship.png')"

# Rotate 180°:    (facing up → facing down, etc)
python3 -c "from PIL import Image; img = Image.open('ship.png'); img.rotate(180, expand=True).save('ship.png')"

# Rotate 90° CCW: (facing west → facing south)
python3 -c "from PIL import Image; img = Image.open('ship.png'); img.rotate(90, expand=True).save('ship.png')"
```

---

## Gemini Prompting Best Practices

### The Problem
Gemini generates "complete space scenes" by default, adding:
- Black space backgrounds with scattered stars
- Atmospheric depth and lighting
- Context that makes assets hard to isolate

For game engines, we need **isolated assets** that can be composited on OUR backgrounds.

### ✅ GOOD Prompts (Isolated Assets)

**Structures & Objects:**
```
"Top-down industrial cargo container, pre-rendered 3D sprite,
glossy metallic materials, weathered steel texture, dramatic rim lighting,
isolated on white background, no surrounding space, game sprite, clean edges, 400x300"

"Isometric industrial mining platform, pre-rendered 3D style, single structure,
weathered metal and rust, hard shadows, crisp highlights, transparent background,
no stars or space context, game asset, 512x384"
```

**Enemy Ships & Combat:**
```
"Enemy raider interceptor ship, pre-rendered 3D sprite, top-down view,
battle-scarred hull, reflective metallic panels, dramatic rim lighting,
hard shadows, photorealistic textures, isolated asset on white background,
game sprite, clean edges, 400x400"
```

**Cosmic Elements:**
```
"Isolated blue nebula cloud formation, wispy gas tendrils only,
no background stars, transparent edges, dark oppressive atmosphere, game parallax layer, 768x768"

"Single planet with thin atmospheric ring, isolated on white background,
top-down perspective, no surrounding space, crisp highlights on surface, 600x600"
```

**Key Phrases to Include:**
- `top-down view` / `overhead perspective` → 2D game angle
- `isolated asset` / `single object` → Prevents scene generation
- `transparent background` / `on white background` → Easy to process
- `no surrounding space` / `no stars` / `no background` → Explicit
- `game sprite` / `game asset` → Signals intended use
- `clean edges` / `sharp silhouette` → Better for engines

### ❌ BAD Prompts (Scene Generation)

**Avoid these:**
```
"Space station in deep space" → Adds black space + stars
"Distant planet silhouette" → Adds complete environment
"Industrial structure far away" → Adds atmospheric context
```

**Phrases to Avoid:**
- `in space` / `in deep space` → Triggers scene mode
- `far away` / `distant` → Adds depth/background
- `silhouette` → May add dramatic backdrops
- `atmospheric` (cosmic sense) → Adds environment

### Processing Workflow

1. **Generate with good prompt** → Gemini MCP
2. **Remove background** → `remove-background` skill (for white/gray backgrounds)
3. **Isolate cosmic assets** → Use specialized scripts:
   - Planets: `scripts/asset-processing/isolate_planet.py`
   - Nebulae: `scripts/asset-processing/isolate_nebula.py`
   - Star clusters: `scripts/asset-processing/isolate_star_cluster.py`
4. **Run QA pipeline** → `scripts/asset-processing/asset_qa_pipeline.py`
5. **Verify in-game** → Test in actual parallax layers

### Specialized Tools

**Planet Isolation** (`isolate_planet.py`):
For planet assets with atmospheric glows, automatically detects the blue glow and removes dark space in corners while preserving the planet + atmosphere.

```bash
python3 scripts/asset-processing/isolate_planet.py assets/far/planet.png assets/far/planet_clean.png
```

**Nebula Isolation** (`isolate_nebula.py`):
For nebula assets with scattered dark space "fluff", detects colored gas clouds and removes excess black space beyond the nebula core. Uses 85th percentile radius cutoff to be aggressive about removing outer wisps while preserving the main nebula body.

```bash
python3 scripts/asset-processing/isolate_nebula.py assets/backdrop/nebula_purple_deep.png assets/backdrop/nebula_clean.png
```

**Star Cluster Isolation** (`isolate_star_cluster.py`):
For star cluster assets, uses morphological operations to detect concentrated stellar regions and removes dark space beyond the cluster. Preserves the star concentration while removing scattered outlier stars and black space.

```bash
python3 scripts/asset-processing/isolate_star_cluster.py assets/backdrop/star_cluster_1.png assets/backdrop/cluster_clean.png
```

## Enemy Sprite Metadata (Sizing + Collision)

Enemy sprites often ship with large transparent padding. To keep gameplay size/collision consistent,
we generate a manifest that records the true alpha bounds and recommended collision shapes.

**Generate the manifest:**
```bash
python3 scripts/asset-processing/generate_enemy_manifest.py
```

**Destructive regenerate (ignore manual overrides):**
```bash
python3 scripts/asset-processing/generate_enemy_manifest.py --destructive
```

**Outputs:**
- `assets/enemies/enemy_manifest.yaml`

**Defaults:**
- Visual bounds threshold: `16`
- Collision bounds threshold: `64`
- Gameplay size uses **content height** in game units
- Collision shapes supported: `circle`, `ellipse`, `capsule` (capsule aligns to longest axis)

**Manual overrides preserved:**
- `gameplay_height_gu`, `collision_shape`, `collision_scale`, and `sockets` are preserved if already present.

**Socket schema (optional):**
```yaml
sockets:
  - id: left
    offset_px: [-18, -6]
    angle_deg: 0      # optional
    tags: ["left"]    # optional
```

Offsets are **sprite-local** in pixels, relative to the texture center (+x right, +y up).

**Firing override (optional):**
```yaml
fire_cooldown: 1.2
```

When set, this overrides the default `EnemyType` fire cooldown (per-enemy).

If a sprite changes, re-run the script so collision + sizing stay correct.

## Asset QA Pipeline

## Purpose

Automatically detect and fix common issues with AI-generated game assets:
- White/light backgrounds that should be transparent
- Hard edges on parallax/backdrop images (should fade to transparent)
- Oversized files (raw AI output without optimization)

## Usage

**Run after generating any new assets:**

```bash
# Auto-fix mode (default) - fixes issues automatically
python3 asset_qa_pipeline.py

# Check-only mode - report issues without fixing
python3 asset_qa_pipeline.py --no-fix
```

## What It Checks

### 1. White Backgrounds
**Issue**: AI-generated sprites often have white/light gray backgrounds instead of transparency
**Fix**: Converts pixels with RGB > 220 to fully transparent (alpha=0)
**Applies to**: All asset types

### 2. Hard Edges
**Issue**: Parallax/backdrop images have sharp rectangular edges that look wrong in-game
**Fix**: Adds 80px alpha feathering to all edges (smooth fade to transparent)
**Applies to**: `parallax/` and backdrop images only

### 3. Large File Sizes
**Issue**: Raw AI output can be 2MB+ per image
**Fix**: Reports warning (manual optimization needed)
**Applies to**: All asset types

## Directory Structure

The pipeline scans:
- `assets/backdrop/` - Deepest layer (nebulae, cosmic phenomena, static)
- `assets/far/` - Far layer (distant objects, slow parallax)
- `assets/backgrounds/` - Mid-distance backgrounds
- `assets/doodads/` - Small environmental objects (gameplay layer)
- `assets/structures/` - Large structures (walls, towers, mega-structures)
- `assets/sprites/ships/` - Player/enemy ships
- `assets/particles/` - Particle effects

**Note:** Cosmic phenomena (nebulae, dust clouds, star fields) in `backdrop/` are NOT feathered - black space with stars is intentional content.

## Example Output

```
=== Scanning doodads: assets/doodads ===
⚠ cargo_container_1.png
  - White background: 38.2% (521511 pixels)
  → Removed 521511 white pixels

✓ asteroid_1.png - OK
✓ beacon_1.png - OK

=== Scanning parallax: assets/parallax ===
⚠ nebula_orange_industrial.png
  - Hard edges: avg edge alpha 248
  → Feathered edges (80px)
  - Large file: 1.66 MB

Summary: 15 files, 3 issues, 2 fixes applied
```

## Integration

**After generating assets with Gemini/MCP:**

```bash
# Generate asset
claude> generate a cargo container sprite

# Run QA pipeline
python3 asset_qa_pipeline.py

# Test in game
cargo run --release
```

## Technical Details

**White background removal:**
- Threshold: RGB > 220
- Converts to RGBA (0,0,0,0)
- Preserves colored pixels

**Edge feathering:**
- 80px gradient from edge to interior
- Linear alpha falloff
- Preserves center fully opaque

**File size:**
- Warning threshold: 2MB
- Typical good size: 50-500KB
