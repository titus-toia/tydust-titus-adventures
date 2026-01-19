# Asset Generation Guidelines

## ⚠️ CRITICAL: Asset Preservation Rules

**NEVER overwrite or delete existing assets in `assets/` without explicit user permission!**

### Before Replacing Any Asset:
1. **Check if file exists** in `assets/` directory
2. **Ask user permission** before overwriting
3. **Backup to timestamped file** if replacing:
   ```bash
   # Example: backup before replacement
   cp assets/ui/shield.png assets/ui/shield_backup_20260113.png
   ```
4. **Work in `generated_imgs/` first** - only move to `assets/` after user approval
5. **Never assume old assets are garbage** - the user may want to keep them

### Asset Workflow (MANDATORY):
1. Generate → `generated_imgs/`
2. **STOP - Show user the raw output and WAIT FOR APPROVAL**
   - Do NOT process, rename, or move anything until user gives the go signal
   - User may want to iterate, regenerate, or reject entirely
3. Only after user approval: Process (background removal, staging by category)
4. **ASK USER** before moving to `assets/` (especially if replacing existing)
5. Only then → move to `assets/[category]/`

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

Gemini MCP automatically saves to: `generated_imgs/generated-{timestamp}-{id}.png`

### 2. Stage in generated_imgs/ by Category
Move generated files to the appropriate **staging directory**:

```
generated_imgs/
├── ships/              (player ships, variants)
├── enemies/            (enemy ships, raiders)
├── structures/         (industrial buildings, stations)
├── doodads/            (cargo, debris, environmental objects)
├── particles/          (effects particles)
├── effects/            (shields, explosions, warps)
├── parallax/           (mid-layer scrolling elements)
├── backdrop/           (nebulae, cosmic phenomena)
├── far/                (distant objects, planets)
├── ui/                 (buttons, icons, HUD)
├── pickups/            (collectibles, power-ups)
└── misc/               (experimental/temporary)
```

This staging area lets you organize, review, and process generated assets before finalizing them in `assets/`.

### 3. Background Removal & Processing

**Remove backgrounds** with rembg (AI-based for clean edges):
```bash
python3 << 'EOF'
from rembg import remove
from PIL import Image
img = Image.open("generated_imgs/ships/my_ship.png")
output = remove(img)
output.save("generated_imgs/ships/my_ship.png")
EOF
```

### 4. Move to Final Asset Directory
Once processed, move from staging to final location:
```bash
# Example:
mv generated_imgs/ships/my_ship.png assets/sprites/ships/my_ship.png
```

### 5. Update Prompt Tracking JSON

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

### 6. Run QA Pipeline

After moving assets to final location, run the automated QA pipeline:

```bash
python3 scripts/asset-processing/asset_qa_pipeline.py
```

This automatically fixes:
- White/light backgrounds (converts to transparent)
- Hard edges on parallax/backdrop (adds feathering)
- Reports oversized files

See `ASSET_PIPELINE.md` for full details.

### 7. Special Processing (if needed)

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

### 3. STOP - Wait for User Approval

**CRITICAL: Do NOT proceed until user gives the go signal.**

After generating, show the user:
- The raw generated image(s)
- Brief notes on what came out (orientation, style, any issues)

Then **WAIT**. The user may want to:
- Approve and continue to processing
- Request iteration/edits on the current image
- Regenerate with different prompts
- Reject entirely and try something else

**Do NOT:**
- Rename or move files
- Run background removal
- Update generation logs
- Stage into category folders

Until user explicitly approves (e.g., "looks good", "process it", "go ahead").

### 4. Stage in generated_imgs/ by Category

Gemini MCP automatically saves to: `generated_imgs/generated-{timestamp}-{id}.png`

**Action:** Rename and move to staging directory:

```bash
# Generate descriptive filename
# Format: {description_words}.png

# Examples:
mv generated_imgs/generated-*.png generated_imgs/ships/enemy_fighter_ship.png
mv generated_imgs/generated-*.png generated_imgs/structures/mining_platform.png
mv generated_imgs/generated-*.png generated_imgs/doodads/cargo_container.png
```

Stage by **category directory** in `generated_imgs/`, not the final asset path. This staging area is for organization and review before processing.

### 5. Remove Backgrounds (in staging area)

Process staged asset with rembg:

```python
from rembg import remove
from PIL import Image

path = "generated_imgs/ships/enemy_fighter_ship.png"
img = Image.open(path)
output = remove(img)
output.save(path)
```

### 6. Update Generation Log

**File:** `assets/generation_log.json`

Track the prompt used for this asset:

```json
{
  "assets/sprites/raiders/enemy_fighter_ship.png": "enemy fighter ship with red paint, top-down view, isolated asset on white background, game sprite"
}
```

Note: Log uses final `assets/` path, even though asset is still in staging.

**Implementation:**
```python
import json
from pathlib import Path

log_path = Path("assets/generation_log.json")
log = json.load(open(log_path)) if log_path.exists() else {}
log[final_asset_path] = full_prompt_used
json.dump(log, open(log_path, 'w'), indent=2)
```

### 7. Move to Final Asset Directory

Move from staging to final location:

```bash
# From generated_imgs staging to final assets directory
mv generated_imgs/{category}/{filename}.png assets/{final_category}/{filename}.png

# Examples:
mv generated_imgs/ships/player_fighter.png assets/sprites/ships/player_fighter.png
mv generated_imgs/structures/mining_platform.png assets/structures/mining_platform.png
mv generated_imgs/enemies/raider_boss.png assets/sprites/raiders/raider_boss.png
```

### 8. Run QA Pipeline

After moving to final asset directory, run the automated QA pipeline:

```bash
python3 scripts/asset-processing/asset_qa_pipeline.py
```

This will:
- Clean up any remaining white pixels (threshold: RGB > 220)
- Add edge feathering for parallax/backdrop layers (80px gradient)
- Report file size issues (>2MB warning)
- Verify transparency

**Category-specific handling:**

**For backdrop assets (nebulae, cosmic):**
- Skip AI background removal (threshold removal creates nice porous effects)
- QA pipeline handles cleanup automatically

**For all other assets (ships, structures, doodads, etc):**
- Background already removed via rembg in staging
- QA pipeline ensures any remaining white is cleaned

### 9. Confirm to User

Report back with:
- ✓ Category detected
- ✓ Asset generated
- ✓ Staged to: `generated_imgs/{category}/{filename}`
- ✓ Moved to: `assets/{final_path}`
- ✓ Background removed (rembg)
- ✓ Generation log updated
- ✓ QA pipeline completed

## Example Complete Workflow

**User says:** "Generate an industrial cargo container"

**Your response (Phase 1 - Generate & Wait):**

1. **Detect category:** `doodads` (contains "cargo container")

2. **Enhance prompt:**
   - Original: "industrial cargo container"
   - Enhanced: "industrial cargo container, pre-rendered 3D sprite, top-down view, glossy metallic materials, weathered steel, isolated on white background, game sprite, 400x300"

3. **Generate:** Call `mcp__nano-banana__generate_image` with enhanced prompt

4. **STOP - Show user and wait:**
   ```
   Generated industrial cargo container.
   - Style: Pre-rendered 3D, weathered metal look
   - Orientation: Top-down ✓
   - Notes: Clean edges, good isolation

   Let me know if this looks good, or if you want to iterate/regenerate.
   ```

**User says:** "looks good, process it"

**Your response (Phase 2 - Process):**

5. **Stage in generated_imgs/:**
   - From: `generated_imgs/generated-2026-01-10T15-30-45-123Z.png`
   - To: `generated_imgs/doodads/industrial_cargo_container.png`

6. **Remove background:**
   - Process in staging with rembg to clean edges

7. **Update log:** Add entry to `assets/generation_log.json` (with final asset path)

8. **Move to final location:**
   - From: `generated_imgs/doodads/industrial_cargo_container.png`
   - To: `assets/doodads/industrial_cargo_container.png`

9. **Run QA:** Execute asset QA pipeline on final location

10. **Report:**
   ```
   ✓ Processed industrial cargo container
   ✓ Staged to: generated_imgs/doodads/industrial_cargo_container.png
   ✓ Moved to: assets/doodads/industrial_cargo_container.png
   ✓ Background removed (rembg)
   ✓ Generation log updated
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

**Process each through the full workflow:**
```
User: "Generate 3 enemy ships"

Response:
1. Generate enemy ship #1
   ✓ Staged to: generated_imgs/enemies/enemy_ship_1.png
   ✓ Moved to: assets/sprites/raiders/enemy_ship_1.png
2. Generate enemy ship #2
   ✓ Staged to: generated_imgs/enemies/enemy_ship_2.png
   ✓ Moved to: assets/sprites/raiders/enemy_ship_2.png
3. Generate enemy ship #3
   ✓ Staged to: generated_imgs/enemies/enemy_ship_3.png
   ✓ Moved to: assets/sprites/raiders/enemy_ship_3.png
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

- **ALWAYS** wait for user approval after generating before processing anything
- **ALWAYS** detect category automatically - don't ask user unless ambiguous
- **ALWAYS** enhance prompts with isolation keywords
- **ALWAYS** move file from generated_imgs/ to correct assets/ directory (after approval)
- **ALWAYS** update generation_log.json
- **ALWAYS** run appropriate background removal based on category
- **ALWAYS** run QA pipeline after background removal
- **NEVER** process, rename, or move files until user gives the go signal
- **NEVER** skip the approval step - user may want to iterate or reject

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