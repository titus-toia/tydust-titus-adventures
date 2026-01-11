# Asset Generation Guidelines

## Directory Organization

When generating assets, save them to the appropriate category directory:

### Core Asset Categories

**Structures** → `assets/structures/`
- Industrial structures, space stations, walls, towers
- Mega-structures, platforms, facilities
- Example: mining_platform_1.png, cargo_bay_3.png

**Enemies/Raiders** → `assets/sprites/raiders/`
- Enemy ships, hostile units, bosses
- Example: hellcat.png, marauder.png, scimitar.png

**Player Ships** → `assets/sprites/ships/`
- Player-controlled ships, variants
- Example: wraith.png, tempest.png, bastion.png

**Doodads** → `assets/doodads/`
- Small environmental objects (cargo containers, debris, antennas)
- Non-interactive decorative elements
- Example: cargo_container_1.png, asteroid_2.png, beacon_1.png

**Ambient Doodads** → `assets/doodads/ambient/`
- Atmospheric environmental elements
- Background decorations that don't interact with gameplay
- Example: sparking_debris_1.png, venting_pipe_1.png

**Projectiles** → `assets/sprites/projectiles/`
- Player weapon projectiles (bullets, missiles, beams)
- Example: basic_blaster.png, plasma_cannon.png, laser_beam.png

**Enemy Projectiles** → `assets/sprites/enemy_projectiles/`
- Enemy weapon fire
- Example: plasma_ball.png, spread_shot.png, basic_shot.png

**Particles** → `assets/particles/`
- Particle effects (exhaust, sparks, explosions, impacts)
- Small, repeating visual effects
- Example: exhaust_cyan.png, flame_orange.png, spark_white.png

**Parallax** → `assets/parallax/`
- Mid-layer background elements that scroll (gas wisps, distant debris)
- Objects with moderate depth
- Example: nebula_red_1.png, gas_wisp_purple_1.png, passing_rock_1.png

**Backgrounds** → `assets/backgrounds/`
- Mid-distance background layers
- General environment elements

**Far** → `assets/far/`
- Distant layer (far planets, distant structures)
- Very slow parallax elements
- Example: planet_1.png, distant_moon_1.png

**Backdrop** → `assets/backdrop/`
- Deepest static layer (nebulae, star fields, cosmic phenomena)
- No parallax scrolling, pure atmosphere
- Example: nebula_purple_deep.png, star_cluster_1.png

**UI Elements** → `assets/ui/`
- Menu elements, buttons, icons, HUD components
- Example: button_start.png, health_bar.png, weapon_icon_1.png

**Pickups** → `assets/pickups/`
- Collectibles, power-ups, bonus items
- Example: health_pack.png, shield_boost.png, score_gem.png

**Effects** → `assets/effects/`
- Visual effects that aren't particles (shields, explosions, warps)
- Example: shield_impact.png, warp_effect.png

**Miscellaneous** → `assets/misc/`
- Assets that don't fit other categories
- Temporary/experimental assets

## Asset Generation Workflow

### 1. Generate Asset
```bash
# Use Gemini MCP to generate image with appropriate prompt
# Follow prompting best practices from ASSET_PIPELINE.md
```

### 2. Save to Appropriate Directory
Save generated images to the correct category directory listed above.

**Important:** Always generate assets in the project directory (`/home/titus/tydust/assets/...`), **NOT** in skill directories.

### 3. Update Prompt Tracking JSON

**Maintain:** `assets/generation_log.json`

Track all generated assets with their prompts for quality control and reproducibility:

```json
{
  "assets/structures/mining_platform_1.png": "Isometric industrial mining platform, single structure, transparent background, no stars, game asset, 512x384",
  "assets/sprites/raiders/viper.png": "Top-down enemy fighter ship, red paint, isolated asset on white background, game sprite, 256x256",
  "assets/particles/exhaust_cyan.png": "Glowing cyan particle exhaust flame, small sprite, transparent background, 64x64"
}
```

**After each generation session, append new entries to this JSON file.**

This log helps identify:
- Which prompts produced good results
- Which assets were successfully generated
- What to regenerate if quality is poor

### 4. Run QA Pipeline

After generating assets, run the automated QA pipeline:

```bash
python3 scripts/asset-processing/asset_qa_pipeline.py
```

This automatically fixes:
- White/light backgrounds (converts to transparent)
- Hard edges on parallax/backdrop (adds feathering)
- Reports oversized files

See `ASSET_PIPELINE.md` for full details.

### 5. Special Processing (if needed)

For cosmic assets that Gemini adds black space to:

**Planets:**
```bash
python3 scripts/asset-processing/isolate_planet.py assets/far/planet.png assets/far/planet_clean.png
```

**Nebulae:**
```bash
python3 scripts/asset-processing/isolate_nebula.py assets/backdrop/nebula.png assets/backdrop/nebula_clean.png
```

**Star Clusters:**
```bash
python3 scripts/asset-processing/isolate_star_cluster.py assets/backdrop/stars.png assets/backdrop/stars_clean.png
```

## Quick Reference

| Asset Type | Directory | Notes |
|------------|-----------|-------|
| Structures | `structures/` | Industrial buildings, stations |
| Enemy ships | `sprites/raiders/` | Hostile units |
| Player ships | `sprites/ships/` | Player-controlled |
| Doodads | `doodads/` | Small objects, debris |
| Player projectiles | `sprites/projectiles/` | Weapon fire |
| Enemy projectiles | `sprites/enemy_projectiles/` | Enemy fire |
| Particles | `particles/` | Effects particles |
| Parallax | `parallax/` | Mid-layer scrolling |
| Far layer | `far/` | Distant objects |
| Backdrop | `backdrop/` | Static deepest layer |
| UI | `ui/` | Interface elements |
| Pickups | `pickups/` | Collectibles |
| Effects | `effects/` | Visual effects |
| Misc | `misc/` | Other/experimental |

---

# AUTOMATED ASSET GENERATION WORKFLOW

**CRITICAL: When the user requests asset generation, follow this workflow automatically.**

## When to Trigger This Workflow

Trigger when user says:
- "Generate [asset description]"
- "Create a [asset type]"
- "Make a [asset description]"
- "I need a [asset type]"

## Automated Workflow Steps

### 1. Detect Category from Description

Use keyword matching to automatically determine the correct asset directory:

```python
CATEGORY_KEYWORDS = {
  'structures': ['structure', 'station', 'platform', 'tower', 'building', 'facility', 'bay', 'wall', 'mining', 'refinery', 'reactor'],
  'sprites/raiders': ['enemy', 'raider', 'hostile', 'boss', 'elite', 'marauder', 'scimitar', 'hellcat'],
  'sprites/ships': ['player ship', 'corvette', 'interceptor', 'wraith', 'tempest', 'bastion', 'valkyrie'],
  'doodads': ['cargo', 'container', 'debris', 'antenna', 'fragment', 'wreckage', 'hull', 'escape pod', 'satellite'],
  'doodads/ambient': ['spark', 'vent', 'leak', 'damaged', 'sparking', 'venting', 'leaking'],
  'sprites/projectiles': ['projectile', 'bullet', 'missile', 'beam', 'laser', 'weapon fire', 'blaster', 'plasma cannon'],
  'sprites/enemy_projectiles': ['enemy projectile', 'enemy fire', 'enemy bullet'],
  'particles': ['particle', 'exhaust', 'flame', 'spark', 'smoke', 'explosion', 'impact', 'muzzle flash'],
  'parallax': ['gas wisp', 'nebula cloud', 'passing', 'drifting', 'floating debris'],
  'far': ['distant', 'far planet', 'far moon', 'far away'],
  'backdrop': ['nebula', 'star field', 'star cluster', 'cosmic', 'deep space background'],
  'ui': ['button', 'icon', 'menu', 'HUD', 'interface', 'UI element'],
  'pickups': ['pickup', 'power-up', 'collectible', 'health pack', 'shield boost'],
  'effects': ['shield effect', 'warp effect', 'explosion effect', 'impact effect'],
  'misc': ['test', 'experimental', 'prototype']
}
```

**Detection Algorithm:**
1. Convert user description to lowercase
2. Check keywords in priority order (most specific first):
   - Check `sprites/enemy_projectiles` before `sprites/projectiles`
   - Check `doodads/ambient` before `doodads`
   - Check `sprites/raiders` before generic matches
3. Return first matching category
4. Default to `misc` if no match

**Example:**
- User: "Generate an enemy fighter ship"
- Contains: "enemy" → Match `sprites/raiders`
- Category: `sprites/raiders`

### 2. Generate with Gemini MCP

Call the Gemini MCP tool to create the image:

```
mcp__nano-banana__generate_image(prompt=user_description)
```

**Important:** Enhance prompts for better results by adding:
- `"isolated asset on white background"`
- `"top-down view"` (for ships/structures)
- `"transparent background"` (for particles/effects)
- `"game sprite"` or `"game asset"`
- `"no surrounding space"` (to prevent black space backgrounds)

### 3. Determine Output Path

Generate descriptive filename and path:

**Filename format:** `{description_words}_{timestamp}.png`

Example:
```python
# User prompt: "enemy fighter ship with red paint"
# Category detected: sprites/raiders
# Filename: enemy_fighter_ship_20260110_153045.png
# Full path: assets/sprites/raiders/enemy_fighter_ship_20260110_153045.png
```

### 4. Move Generated File

Gemini MCP saves to: `/home/titus/generated_imgs/generated-{timestamp}-{id}.png`

**Action:** Move the generated file to the correct asset directory:

```bash
mv /home/titus/generated_imgs/generated-*.png assets/{category}/{descriptive_name}.png
```

### 5. Update Generation Log

**File:** `assets/generation_log.json`

Add entry mapping asset path to the prompt used:

```json
{
  "assets/sprites/raiders/enemy_fighter_ship_20260110_153045.png": "enemy fighter ship with red paint, top-down view, isolated asset on white background, game sprite"
}
```

**Implementation:**
```python
import json
from pathlib import Path

log_path = Path("assets/generation_log.json")
log = json.load(open(log_path)) if log_path.exists() else {}
log[relative_asset_path] = full_prompt_used
json.dump(log, open(log_path, 'w'), indent=2)
```

### 6. Run Post-Processing

**CRITICAL: Always run background removal and QA in this order:**

**Step 6a: Remove Background (AI-based)**

Use the `remove-background` Skill for generated assets:

```
Skill: remove-background
Args: path/to/generated/asset.png
```

This uses AI (rembg) to intelligently remove backgrounds while preserving:
- Alpha transparency
- Edge quality
- Asset details
- Proper occlusion (objects block what's behind them)

**IMPORTANT - Category-specific background removal:**

**Use AI-based removal (`remove-background` Skill) for:**
- `sprites/*` - Ships, projectiles, enemies (needs clean edges)
- `structures/*` - Buildings, stations (must occlude properly)
- `doodads/*` - Cargo, debris, fragments (clean cutouts)
- `particles/*` - Effects (precise alpha)
- `parallax/*` - Mid-layer elements (proper layering)
- **`far/*`** - Distant hulks, planets, structures (MUST occlude stars behind them)
- `ui/*`, `pickups/*`, `effects/*` - All need clean alpha

**Use simple threshold removal (QA pipeline only) for:**
- **`backdrop/*`** - Nebulae, cosmic phenomena
  - Threshold-based removal creates porous/wispy effects (desirable!)
  - QA pipeline's RGB > 220 removal is perfect for these
  - No need for AI removal - let it be transparent/holey

**Skip all background removal for:**
- Assets explicitly described as having backgrounds (rare)
- Pre-processed assets (if user says "already has transparent background")

**Step 6b: Run QA Pipeline**

After background removal, run the QA pipeline:

```bash
python3 scripts/asset-processing/asset_qa_pipeline.py
```

This will:
- Clean up any remaining white pixels (threshold: RGB > 220)
- Add edge feathering for parallax/backdrop layers (80px gradient)
- Report file size issues (>2MB warning)
- Verify transparency

**Order matters:**
1. `remove-background` Skill first (AI removal)
2. QA pipeline second (cleanup + feathering)

### 7. Confirm to User

Report back with:
- ✓ Category detected
- ✓ Asset generated
- ✓ Saved to: `{full_path}`
- ✓ Generation log updated
- ✓ QA pipeline completed
- Next steps (if applicable)

## Example Complete Workflow

**User says:** "Generate an industrial cargo container"

**Your response:**

1. **Detect category:** `doodads` (contains "cargo container")

2. **Enhance prompt:**
   - Original: "industrial cargo container"
   - Enhanced: "industrial cargo container, top-down view, isolated asset on white background, game sprite, clean edges, 400x300"

3. **Generate:** Call `mcp__nano-banana__generate_image` with enhanced prompt

4. **Move file:**
   - From: `/home/titus/generated_imgs/generated-2026-01-10T15-30-45-123Z.png`
   - To: `assets/doodads/industrial_cargo_container_20260110_153045.png`

5. **Update log:** Add entry to `assets/generation_log.json`

6. **Remove background:** Use `remove-background` Skill on the asset

7. **Run QA:** Execute asset QA pipeline

8. **Report:**
   ```
   ✓ Generated industrial cargo container
   ✓ Category: doodads
   ✓ Saved to: assets/doodads/industrial_cargo_container_20260110_153045.png
   ✓ Generation log updated
   ✓ Background removed (AI-based)
   ✓ QA pipeline completed

   Asset ready for use in game!
   ```

## Special Cases

### Ambiguous Category

If category cannot be determined (defaults to `misc`):

**Ask user:**
```
⚠ Couldn't detect category from description.
Available categories: structures, sprites/raiders, sprites/ships, doodads, particles, etc.

Which category should this go in?
```

### Multiple Assets in One Request

If user requests multiple assets:

**Process each separately:**
```
User: "Generate 3 enemy ships"

Response:
1. Generate enemy ship #1 → sprites/raiders/enemy_ship_1_20260110_153045.png
2. Generate enemy ship #2 → sprites/raiders/enemy_ship_2_20260110_153050.png
3. Generate enemy ship #3 → sprites/raiders/enemy_ship_3_20260110_153055.png
```

### Cosmic Assets (Nebulae, Planets)

Gemini often adds black space to cosmic elements.

**For backdrop assets (nebulae, star clusters):**
```
✓ Generated nebula backdrop
✓ Saved to: assets/backdrop/nebula_purple_20260110_153045.png
✓ Skipped AI background removal (threshold removal will create nice wispy effects)
✓ QA pipeline will handle background cleanup

If excess black space remains, run specialized isolation:
  python3 scripts/asset-processing/isolate_nebula.py assets/backdrop/nebula_purple_*.png assets/backdrop/nebula_clean.png
```

**For far layer assets (planets, distant structures):**
```
✓ Generated distant planet
✓ Saved to: assets/far/planet_blue_20260110_153045.png
✓ AI background removal complete (planet will properly occlude stars)
✓ QA pipeline complete

If planet has atmospheric glow with excess black corners:
  python3 scripts/asset-processing/isolate_planet.py assets/far/planet_blue_*.png assets/far/planet_clean.png
```

## Important Notes

- **ALWAYS** detect category automatically - don't ask user unless ambiguous
- **ALWAYS** enhance prompts with isolation keywords
- **ALWAYS** move file from generated_imgs/ to correct assets/ directory
- **ALWAYS** update generation_log.json
- **ALWAYS** run appropriate background removal based on category
- **ALWAYS** run QA pipeline after background removal
- **NEVER** leave generated files in generated_imgs/ directory
- **NEVER** skip the workflow steps

**Post-processing order is critical:**

**For most assets (sprites, structures, doodads, far, etc.):**
1. Move asset to correct directory
2. Update generation_log.json
3. Run `remove-background` Skill (AI-based removal)
4. Run QA pipeline (cleanup + feathering)

**For backdrop assets (nebulae, cosmic phenomena):**
1. Move asset to correct directory
2. Update generation_log.json
3. Skip AI background removal (let QA pipeline handle it)
4. Run QA pipeline (threshold removal creates nice porous effects)

This workflow is CRITICAL for maintaining organized assets. Follow it every time.