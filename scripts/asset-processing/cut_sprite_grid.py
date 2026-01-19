#!/usr/bin/env python3
"""
Cut a sprite sheet grid into individual centered frames.

Detects content center of mass and repositions to frame center.
Useful for explosion/effect sprite sheets where content may not be perfectly aligned.

Usage:
    python cut_sprite_grid.py <input_image> <output_prefix> [--cols N] [--rows N] [--size N]

Examples:
    python cut_sprite_grid.py explosion.png explosion_frame --cols 3 --rows 3
    python cut_sprite_grid.py effects.png effect --cols 4 --rows 2 --size 256
"""

import argparse
from PIL import Image
import numpy as np
from pathlib import Path


def find_content_center(frame_arr, brightness_threshold=25):
    """Find center of mass of content based on brightness."""
    r, g, b = frame_arr[:,:,0], frame_arr[:,:,1], frame_arr[:,:,2]
    brightness = (r.astype(float) + g.astype(float) + b.astype(float)) / 3
    content_mask = brightness > brightness_threshold

    ys, xs = np.where(content_mask)
    if len(xs) > 0:
        return int(np.mean(xs)), int(np.mean(ys))
    return None, None


def cut_and_center_grid(input_path, output_prefix, cols=3, rows=3, target_size=None, brightness_threshold=25):
    """
    Cut sprite sheet into grid and center each frame.

    Args:
        input_path: Path to source sprite sheet
        output_prefix: Prefix for output files (e.g., "explosion_frame" -> "explosion_frame_1.png")
        cols: Number of columns in grid
        rows: Number of rows in grid
        target_size: Output frame size (default: auto from grid cell size)
        brightness_threshold: Threshold for detecting content (default: 25, good for black bg)

    Returns:
        List of output file paths
    """
    img = Image.open(input_path).convert("RGBA")
    arr = np.array(img)
    h, w = arr.shape[:2]

    cell_w, cell_h = w // cols, h // rows

    if target_size is None:
        target_size = min(cell_w, cell_h)

    print(f"Source: {w}x{h}")
    print(f"Grid: {cols}x{rows}")
    print(f"Cell size: {cell_w}x{cell_h}")
    print(f"Output size: {target_size}x{target_size}")
    print()

    output_dir = Path(output_prefix).parent
    output_dir.mkdir(parents=True, exist_ok=True)

    output_files = []

    for row in range(rows):
        for col in range(cols):
            frame_num = row * cols + col + 1

            # Extract cell
            left = col * cell_w
            top = row * cell_h
            right = left + cell_w
            bottom = top + cell_h

            frame_arr = arr[top:bottom, left:right]

            # Find content center
            content_x, content_y = find_content_center(frame_arr, brightness_threshold)

            if content_x is not None:
                # Calculate offset to center content
                frame_center_x = cell_w // 2
                frame_center_y = cell_h // 2

                offset_x = frame_center_x - content_x
                offset_y = frame_center_y - content_y

                # Adjust for target size difference
                size_offset_x = (target_size - cell_w) // 2
                size_offset_y = (target_size - cell_h) // 2

                paste_x = offset_x + size_offset_x
                paste_y = offset_y + size_offset_y

                print(f"Frame {frame_num}: content center ({content_x}, {content_y}), offset ({offset_x}, {offset_y})")

                # Create centered frame with black background
                frame = Image.fromarray(frame_arr)
                new_frame = Image.new("RGB", (target_size, target_size), (0, 0, 0))

                # For RGBA source, composite properly
                if frame.mode == 'RGBA':
                    temp = Image.new("RGBA", (target_size, target_size), (0, 0, 0, 255))
                    temp.paste(frame, (paste_x, paste_y))
                    new_frame = temp.convert("RGB")
                else:
                    new_frame.paste(frame, (paste_x, paste_y))

                output_path = f"{output_prefix}_{frame_num}.png"
                new_frame.save(output_path)
                output_files.append(output_path)
            else:
                print(f"Frame {frame_num}: no content found, skipping")

    print(f"\nDone! {len(output_files)} frames saved.")
    return output_files


def main():
    parser = argparse.ArgumentParser(description="Cut sprite sheet grid into centered frames")
    parser.add_argument("input", help="Input sprite sheet image")
    parser.add_argument("output_prefix", help="Output filename prefix (e.g., 'explosion_frame')")
    parser.add_argument("--cols", type=int, default=3, help="Number of columns (default: 3)")
    parser.add_argument("--rows", type=int, default=3, help="Number of rows (default: 3)")
    parser.add_argument("--size", type=int, default=None, help="Output frame size (default: auto)")
    parser.add_argument("--threshold", type=int, default=25, help="Brightness threshold for content detection (default: 25)")

    args = parser.parse_args()

    cut_and_center_grid(
        args.input,
        args.output_prefix,
        cols=args.cols,
        rows=args.rows,
        target_size=args.size,
        brightness_threshold=args.threshold
    )


if __name__ == "__main__":
    main()
