#!/usr/bin/env python3
"""
Process tileset batch - crop 2x2 grid into individual tiles and scale to 128x128
"""
from PIL import Image
import sys
import os

def process_batch(input_path, output_dir, tile_names):
	"""
	Crop a 512x512 (or similar) 2x2 grid into 4 tiles and scale to 128x128

	Args:
		input_path: Path to input grid image
		output_dir: Directory to save processed tiles
		tile_names: List of 4 filenames [top-left, top-right, bottom-left, bottom-right]
	"""
	img = Image.open(input_path)
	width, height = img.size

	print(f"Input image: {width}x{height}")

	# Calculate tile size (assuming 2x2 grid)
	tile_width = width // 2
	tile_height = height // 2

	print(f"Tile size before scaling: {tile_width}x{tile_height}")

	# Define crop regions (x, y, x+w, y+h)
	crops = [
		(0, 0, tile_width, tile_height),  # Top-left
		(tile_width, 0, width, tile_height),  # Top-right
		(0, tile_height, tile_width, height),  # Bottom-left
		(tile_width, tile_height, width, height),  # Bottom-right
	]

	os.makedirs(output_dir, exist_ok=True)

	for i, (crop_box, filename) in enumerate(zip(crops, tile_names)):
		tile = img.crop(crop_box)

		# Scale to 128x128
		tile_scaled = tile.resize((128, 128), Image.LANCZOS)

		output_path = os.path.join(output_dir, filename)
		tile_scaled.save(output_path)

		print(f"✓ Saved: {filename} (128x128)")

	print(f"\n✓ Processed {len(tile_names)} tiles to {output_dir}")

if __name__ == "__main__":
	if len(sys.argv) < 3:
		print("Usage: python process_tileset_batch.py <input_image> <output_dir> [tile1.png] [tile2.png] [tile3.png] [tile4.png]")
		print("Example: python process_tileset_batch.py batch1.png tiles/isometric_industrial/ corridor_h.png corridor_v.png cross.png corridor_h_var.png")
		sys.exit(1)

	input_image = sys.argv[1]
	output_dir = sys.argv[2]
	tile_names = sys.argv[3:7]  # Next 4 arguments

	if len(tile_names) != 4:
		print("Error: Must provide exactly 4 tile filenames")
		sys.exit(1)

	process_batch(input_image, output_dir, tile_names)
