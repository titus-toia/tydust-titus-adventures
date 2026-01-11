# Grid Image Extraction Methodology

Documentation for extracting individual tiles from AI-generated grid images while preserving quality and sharing borders evenly.

## Problem Statement

When AI generates a grid of tiles (e.g., 4×4), there are dark borders/separators between tiles. Naive cutting (dividing dimensions by grid size) results in:
- Some tiles getting too much border (suffocated)
- Some tiles eating into neighbors' space
- Uneven border distribution

## Solution Overview

1. **Analyze grid structure** to find exact separator positions
2. **Cut at separator midpoints** to share borders evenly
3. **Crop to content bounds** to remove outer black while preserving interior details

---

## Step 1: Analyze Grid Structure

Find where the dark separator lines are by scanning for columns/rows that are mostly black.

```python
from PIL import Image

img = Image.open('grid_image.png')
pixels = img.load()
width, height = img.size

# Find dark vertical separators
dark_columns = []
for x in range(width):
    # Sample multiple points in this column
    dark_count = 0
    for y in [height//8, height//4, height//2, 3*height//4, 7*height//8]:
        pixel = pixels[x, y]
        brightness = sum(pixel[:3]) / 3
        if brightness < 15:  # Adjust threshold as needed
            dark_count += 1

    # If most samples are dark, this is a separator
    if dark_count >= 4:
        dark_columns.append(x)

# Group into continuous border regions
borders = []
if dark_columns:
    start = dark_columns[0]
    for i in range(1, len(dark_columns)):
        if dark_columns[i] != dark_columns[i-1] + 1:
            # Gap found, end of border
            borders.append((start, dark_columns[i-1]))
            start = dark_columns[i]
    borders.append((start, dark_columns[-1]))

# Example output for 1024×1024 grid:
# Border 0: x=0 to x=53 (54px) - outer left border
# Border 1: x=269 to x=287 (19px) - separator between col 1 and 2
# Border 2: x=504 to x=521 (18px) - separator between col 2 and 3
# Border 3: x=736 to x=755 (20px) - separator between col 3 and 4
# Border 4: x=972 to x=1023 (52px) - outer right border
```

**Repeat for rows** to get horizontal separators.

---

## Step 2: Calculate Cut Points at Separator Midpoints

Calculate the midpoint of each separator to share borders evenly:

```python
# For our example with detected borders:
# Borders: 0-53, 269-287, 504-521, 736-755, 972-1023

col_cuts = [
    0,                          # Left edge
    (0 + 53) // 2,             # Mid of left border = 27
    (269 + 287) // 2,          # Mid of separator 1 = 278
    (504 + 521) // 2,          # Mid of separator 2 = 512
    (736 + 755) // 2,          # Mid of separator 3 = 745
    (972 + 1023) // 2,         # Mid of right border = 997
    1024                        # Right edge
]

# Result: [0, 27, 278, 512, 745, 997, 1024]

# Same for rows
row_cuts = [0, 33, 282, 504, 729, 985, 1024]
```

**Key insight**: Cutting at midpoints means:
- Adjacent tiles each get half of the separator between them
- Outer tiles get half of the outer border
- No wasted or duplicated space

---

## Step 3: Extract Tiles Using Fixed Cut Points

```python
# Define tiles with grid coordinates
tiles = [
    # (col_start, row_start, col_end, row_end), filename
    ((1, 1, 2, 2), 'reactor_core_cyan.png'),      # Top-left square
    ((2, 1, 3, 2), 'storage_silos.png'),          # Top-middle square
    ((4, 1, 5, 3), 'tower_antenna_tall.png'),     # Tall tile (2 rows)
    # ... etc
]

for (col_start, row_start, col_end, row_end), name in tiles:
    box = (
        col_cuts[col_start],
        row_cuts[row_start],
        col_cuts[col_end],
        row_cuts[row_end]
    )
    tile = img.crop(box)
    tile.save(f'output/{name}')
```

---

## Step 4: Remove Outer Black Borders Only

To remove outer black borders while preserving interior black details:

```python
def find_content_bounds(img):
    '''Find bounding box of non-black content'''
    pixels = img.load()
    w, h = img.size

    min_x, max_x = w, 0
    min_y, max_y = h, 0

    # Scan all pixels
    for y in range(h):
        for x in range(w):
            r, g, b = img.getpixel((x, y))[:3]
            brightness = (r + g + b) / 3

            # Non-black pixel = content
            if brightness > 10:  # Threshold: adjust as needed
                min_x = min(min_x, x)
                max_x = max(max_x, x)
                min_y = min(min_y, y)
                max_y = max(max_y, y)

    # Add 1 to max to include the pixel
    return (min_x, min_y, max_x + 1, max_y + 1)

# Apply to each tile
for tile_path in tile_paths:
    tile = Image.open(tile_path)
    content_box = find_content_bounds(tile)
    cropped = tile.crop(content_box)
    cropped.save(tile_path)
```

**Why this works:**
- Outer black borders have NO non-black pixels, so they're excluded from bounds
- Interior black details are surrounded by non-black content, so they're included in bounds
- The bounding box approach naturally preserves all interior details

**Brightness threshold:**
- `brightness > 10`: Very conservative, only excludes pure black (0-10)
- Adjust if needed: lower = more aggressive, higher = more conservative

---

## Complete Script Template

```python
from PIL import Image
import os

def analyze_grid(img):
    '''Find separator positions'''
    pixels = img.load()
    width, height = img.size

    # Find dark columns (vertical separators)
    dark_columns = []
    for x in range(width):
        dark_count = sum(
            1 for y in [height//8, height//4, height//2, 3*height//4, 7*height//8]
            if sum(pixels[x, y][:3]) / 3 < 15
        )
        if dark_count >= 4:
            dark_columns.append(x)

    # Group into borders
    borders = []
    if dark_columns:
        start = dark_columns[0]
        for i in range(1, len(dark_columns)):
            if dark_columns[i] != dark_columns[i-1] + 1:
                borders.append((start, dark_columns[i-1]))
                start = dark_columns[i]
        borders.append((start, dark_columns[-1]))

    # Calculate cut points (midpoints)
    cuts = [0] + [(s + e) // 2 for s, e in borders] + [width]
    return cuts

def find_content_bounds(img):
    '''Find bounding box of non-black content'''
    pixels = img.load()
    w, h = img.size

    min_x, max_x = w, 0
    min_y, max_y = h, 0

    for y in range(h):
        for x in range(w):
            r, g, b = img.getpixel((x, y))[:3]
            if (r + g + b) / 3 > 10:  # Non-black
                min_x, max_x = min(min_x, x), max(max_x, x)
                min_y, max_y = min(min_y, y), max(max_y, y)

    return (min_x, min_y, max_x + 1, max_y + 1)

# Main workflow
img = Image.open('grid_image.png')

# 1. Analyze grid
col_cuts = analyze_grid(img)
row_cuts = analyze_grid(img.rotate(90, expand=True))  # Rotate to analyze rows

# 2. Define tiles
tiles = [
    ((1, 1, 2, 2), 'tile1.png'),
    ((2, 1, 3, 2), 'tile2.png'),
    # ... etc
]

# 3. Extract and crop
os.makedirs('output', exist_ok=True)
for (col_start, row_start, col_end, row_end), name in tiles:
    box = (col_cuts[col_start], row_cuts[row_start], col_cuts[col_end], row_cuts[row_end])
    tile = img.crop(box)

    # Crop to content
    content_box = find_content_bounds(tile)
    tile_cropped = tile.crop(content_box)
    tile_cropped.save(f'output/{name}')
```

---

## Key Takeaways

1. **Don't use simple division**: `width / grid_cols` gives uneven results
2. **Analyze actual separator positions**: Grids may have irregular spacing
3. **Cut at separator midpoints**: Shares borders evenly between tiles
4. **Use bounding box for cropping**: Preserves interior details while removing outer borders
5. **Adjust thresholds as needed**: `brightness < 15` for separator detection, `brightness > 10` for content detection

---

## Example Results

For a 1024×1024 grid with 4×4 tiles:

**Before:**
- Grid cell size: 256×256 (includes borders)
- Mixed border widths: 18-54px

**After cut points:**
- Tile 1: 251×249 (shared borders on all sides)
- Tile 2: 234×249 (shared borders)
- Tower (2-tall): 252×471 (shared borders)

**After content cropping:**
- Tile 1: 217×208 (outer black removed)
- Tile 2: 218×207 (outer black removed)
- Tower: 218×432 (outer black removed)
- Interior black details: **preserved**

---

## Troubleshooting

**Problem**: Tiles still have uneven borders
- **Solution**: Check if separators are detected correctly. Print `borders` list and verify positions match visual inspection.

**Problem**: Content cropping removes too much
- **Solution**: Lower the brightness threshold (e.g., `brightness > 5` instead of `> 10`)

**Problem**: Content cropping doesn't remove enough
- **Solution**: Raise the brightness threshold (e.g., `brightness > 15` instead of `> 10`)

**Problem**: Interior black holes appear
- **Solution**: Use bounding box approach (as documented) instead of pixel-by-pixel transparency removal
