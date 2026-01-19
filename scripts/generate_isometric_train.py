#!/usr/bin/env python3
"""
Generate isometric structure train positions
Analyzes an isometric asset and generates YAML for head-to-tail placement
"""

import sys
from PIL import Image
import numpy as np
from pathlib import Path
import yaml

def analyze_isometric_asset(image_path):
    """Analyze isometric asset to determine dimensions and orientation"""
    img = Image.open(image_path).convert('RGBA')
    width, height = img.size

    # Convert to numpy for analysis
    data = np.array(img)
    alpha = data[:, :, 3]

    # Find bounding box of non-transparent content
    rows = np.any(alpha > 10, axis=1)
    cols = np.any(alpha > 10, axis=0)

    if not np.any(rows) or not np.any(cols):
        print("✗ No visible content found in image")
        return None

    y_min, y_max = np.where(rows)[0][[0, -1]]
    x_min, x_max = np.where(cols)[0][[0, -1]]

    content_width = x_max - x_min + 1
    content_height = y_max - y_min + 1

    print(f"Image dimensions: {width}×{height}px")
    print(f"Content bounding box: {content_width}×{content_height}px")
    print(f"Content position: ({x_min}, {y_min}) to ({x_max}, {y_max})")

    # For isometric perspective:
    # Diagonal modules typically have ~45° angle
    # X-step ≈ content_width * 0.5 to 0.7 (depending on perspective)
    # Y-step (spawn distance) depends on scroll speed and desired spacing

    # Estimate isometric diagonal length
    diagonal = np.sqrt(content_width**2 + content_height**2)

    return {
        'width': width,
        'height': height,
        'content_width': content_width,
        'content_height': content_height,
        'content_center_x': (x_min + x_max) / 2,
        'content_center_y': (y_min + y_max) / 2,
        'diagonal': diagonal,
        'x_min': x_min,
        'x_max': x_max,
        'y_min': y_min,
        'y_max': y_max,
    }

def calculate_train_positions(
    num_modules,
    x_step,
    spawn_distance_start,
    spawn_distance_step,
    direction='bottom-left-to-top-right',
    z_order_start=5,
    center_offset_x=0
):
    """Calculate positions for a train of isometric modules

    Args:
        num_modules: Number of modules in the train
        x_step: Horizontal spacing between modules (px)
        spawn_distance_start: Starting spawn distance (GU)
        spawn_distance_step: Distance between modules (GU)
        direction: 'bottom-left-to-top-right' or 'top-right-to-bottom-left'
        z_order_start: Starting z-order
        center_offset_x: X offset to center the entire train
    """
    positions = []

    # Determine direction multipliers
    if direction == 'bottom-left-to-top-right':
        x_multiplier = 1  # Move right
        z_multiplier = -1  # Z-order decreases (farther modules have lower z)
    else:  # top-right-to-bottom-left
        x_multiplier = -1  # Move left
        z_multiplier = 1  # Z-order increases

    # Calculate center index for symmetrical placement
    center_idx = (num_modules - 1) / 2

    for i in range(num_modules):
        # Calculate position relative to center
        offset_from_center = i - center_idx

        position = {
            'x': center_offset_x + (offset_from_center * x_step * x_multiplier),
            'spawn_distance': spawn_distance_start + (i * spawn_distance_step),
            'z_order': z_order_start + (i * z_multiplier),
            'index': i,
        }
        positions.append(position)

    return positions

def generate_yaml_structure(
    sprite_path,
    layer,
    positions,
    size=None,
    mode='train'
):
    """Generate YAML structure for level file

    Args:
        sprite_path: Path to sprite (relative to assets/)
        layer: Layer name (e.g., 'mega_structures')
        positions: List of position dicts from calculate_train_positions
        size: Optional [width, height] override
        mode: 'train' (centered) or 'bridge' (absolute positions)
    """
    structures = []

    for pos in positions:
        structure = {
            'sprite': sprite_path,
            'layer': layer,
            'position': int(pos['x']),
            'spawn_distance': int(pos['spawn_distance']),
            'z_order': int(pos['z_order']),
        }

        if size:
            structure['size'] = size

        structures.append(structure)

    return structures

def print_yaml_output(structures, mode='train', x_step=None, spawn_step=None):
    """Print YAML structures with comments"""

    if mode == 'train':
        print(f"""
  # ============================================
  # Isometric Train (mode: {mode})
  #   - X step: {x_step:.0f}px, Spawn step: {spawn_step:.0f} GU
  #   - Modules: {len(structures)}
  #   - Z-order: {'decreasing' if structures[0]['z_order'] > structures[-1]['z_order'] else 'increasing'}
  # ============================================""")

    for structure in structures:
        print(f"  - sprite: {structure['sprite']}")
        print(f"    layer: {structure['layer']}")
        print(f"    position: {structure['position']}")
        print(f"    spawn_distance: {structure['spawn_distance']}")
        print(f"    z_order: {structure['z_order']}")
        if 'size' in structure:
            print(f"    size: {structure['size']}")
        print()

def main():
    if len(sys.argv) < 2:
        print("""
Usage: python generate_isometric_train.py <image_path> [options]

Options:
  --modules N           Number of modules in train (default: 7)
  --x-step N           Horizontal spacing in pixels (auto if not specified)
  --spawn-start N      Starting spawn distance in GU (default: 5000)
  --spawn-step N       Spawn distance step in GU (auto if not specified)
  --direction DIR      'bl-tr' or 'tr-bl' (default: bl-tr)
  --layer LAYER        Layer name (default: mega_structures)
  --z-start N          Starting z-order (default: 5)
  --center-x N         X offset to center train (default: 0)
  --scroll-speed N     Scroll speed for auto-calculation (default: 100)
  --sprite-scale N     Scale factor for sprite size (default: 1.0)
  --output FILE        Save to YAML file instead of printing

Example:
  python generate_isometric_train.py assets/structures/isometric/industrial_corridor_module.png \\
    --modules 7 --x-step 385 --spawn-step 699 --layer mega_structures
        """)
        sys.exit(1)

    image_path = sys.argv[1]

    # Parse options
    args = sys.argv[2:]
    options = {
        'modules': 7,
        'x_step': None,  # Auto-calculate
        'spawn_start': 5000,
        'spawn_step': None,  # Auto-calculate
        'direction': 'bl-tr',
        'layer': 'mega_structures',
        'z_start': 5,
        'center_x': 0,
        'scroll_speed': 100,
        'sprite_scale': 1.0,
        'output': None,
    }

    i = 0
    while i < len(args):
        if args[i] == '--modules':
            options['modules'] = int(args[i+1])
            i += 2
        elif args[i] == '--x-step':
            options['x_step'] = float(args[i+1])
            i += 2
        elif args[i] == '--spawn-start':
            options['spawn_start'] = float(args[i+1])
            i += 2
        elif args[i] == '--spawn-step':
            options['spawn_step'] = float(args[i+1])
            i += 2
        elif args[i] == '--direction':
            dir_map = {'bl-tr': 'bottom-left-to-top-right', 'tr-bl': 'top-right-to-bottom-left'}
            options['direction'] = dir_map.get(args[i+1], args[i+1])
            i += 2
        elif args[i] == '--layer':
            options['layer'] = args[i+1]
            i += 2
        elif args[i] == '--z-start':
            options['z_start'] = int(args[i+1])
            i += 2
        elif args[i] == '--center-x':
            options['center_x'] = float(args[i+1])
            i += 2
        elif args[i] == '--scroll-speed':
            options['scroll_speed'] = float(args[i+1])
            i += 2
        elif args[i] == '--sprite-scale':
            options['sprite_scale'] = float(args[i+1])
            i += 2
        elif args[i] == '--output':
            options['output'] = args[i+1]
            i += 2
        else:
            i += 1

    # Analyze asset
    print(f"Analyzing: {image_path}")
    print("=" * 60)

    info = analyze_isometric_asset(image_path)
    if not info:
        sys.exit(1)

    print("\n" + "=" * 60)
    print("ISOMETRIC TRAIN CALCULATION")
    print("=" * 60)

    # Auto-calculate x_step if not provided
    if options['x_step'] is None:
        # For isometric: use ~55% of content width as step
        # This creates slight overlap for seamless connection
        options['x_step'] = info['content_width'] * 0.55
        print(f"Auto X-step: {options['x_step']:.1f}px (55% of content width)")

    # Auto-calculate spawn_step if not provided
    if options['spawn_step'] is None:
        # Calculate based on scroll speed
        # spawn_step should match the visual spacing when scrolling
        # At 100 scroll speed: 1 GU = 1 world unit of movement
        # For 0.52 layer scale (mega_structures), adjust accordingly
        layer_speed_map = {
            'deep_structures': 0.15,
            'mega_structures': 0.4,
            'mid_distance': 0.6,
            'structure_details': 0.8,
            'near_background': 1.0,
        }
        speed_mult = layer_speed_map.get(options['layer'], 1.0)

        # Spawn step in GU to match X spacing visually
        # Approximate: spawn_step ≈ x_step / (scroll_speed * speed_mult * layer_scale)
        layer_scale = 0.52  # Default for mega_structures
        options['spawn_step'] = options['x_step'] / (speed_mult * layer_scale) * (options['scroll_speed'] / 100)
        print(f"Auto Spawn-step: {options['spawn_step']:.1f} GU (matched to visual spacing)")

    print(f"\nTrain Configuration:")
    print(f"  Modules: {options['modules']}")
    print(f"  X-step: {options['x_step']:.1f}px")
    print(f"  Spawn-step: {options['spawn_step']:.1f} GU")
    print(f"  Direction: {options['direction']}")
    print(f"  Layer: {options['layer']}")
    print(f"  Z-order start: {options['z_start']}")
    print(f"  Center offset: {options['center_x']:.1f}px")

    # Calculate positions
    positions = calculate_train_positions(
        num_modules=options['modules'],
        x_step=options['x_step'],
        spawn_distance_start=options['spawn_start'],
        spawn_distance_step=options['spawn_step'],
        direction=options['direction'],
        z_order_start=options['z_start'],
        center_offset_x=options['center_x'],
    )

    # Determine sprite path (relative to assets/)
    sprite_path = Path(image_path)
    if 'assets' in sprite_path.parts:
        # Extract path after 'assets/'
        idx = sprite_path.parts.index('assets')
        sprite_path = '/'.join(sprite_path.parts[idx+1:])
    else:
        sprite_path = sprite_path.name

    # Generate structures
    size = [int(info['content_width'] * options['sprite_scale']),
            int(info['content_height'] * options['sprite_scale'])]

    structures = generate_yaml_structure(
        sprite_path=sprite_path,
        layer=options['layer'],
        positions=positions,
        size=size,
        mode='train',
    )

    print("\n" + "=" * 60)
    print("YAML OUTPUT")
    print("=" * 60)

    # Print or save
    if options['output']:
        with open(options['output'], 'w') as f:
            yaml.dump({'structures': structures}, f, default_flow_style=False, sort_keys=False)
        print(f"✓ Saved to: {options['output']}")
    else:
        print_yaml_output(structures, mode='train',
                         x_step=options['x_step'],
                         spawn_step=options['spawn_step'])

    print("\n" + "=" * 60)
    print("✓ Done!")

if __name__ == "__main__":
    main()
