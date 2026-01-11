#!/usr/bin/env python3
from PIL import Image
import os

# Load the image
img_path = '/home/titus/tydust/generated_imgs/generated-2026-01-08T21-12-45-772Z-edvr10.png'
img = Image.open(img_path)

# Convert to RGBA if needed
if img.mode != 'RGBA':
	img = img.convert('RGBA')

# Image is 1024x1024, split into 4 tiles of 512x512
width, height = img.size
tile_width = width // 2
tile_height = height // 2

# Define crop boxes and names
tiles = [
	((0, 0, tile_width, tile_height), "structure_block_1.png"),  # Top-left
	((tile_width, 0, width, tile_height), "structure_block_2.png"),  # Top-right
	((0, tile_height, tile_width, height), "structure_block_3.png"),  # Bottom-left
	((tile_width, tile_height, width, height), "structure_block_4.png"),  # Bottom-right
]

output_dir = '/home/titus/tydust/assets/structures/'
os.makedirs(output_dir, exist_ok=True)

for crop_box, filename in tiles:
	# Crop the tile
	tile = img.crop(crop_box)

	# Remove gray/checkerboard background
	data = tile.getdata()
	new_data = []
	for pixel in data:
		r, g, b, a = pixel
		# Remove near-gray pixels (checkerboard pattern from transparency)
		# Also remove very dark pixels that might be background
		if (abs(r - g) < 10 and abs(g - b) < 10 and abs(r - b) < 10 and r < 230) and \
		   (r < 30 and g < 30 and b < 30):
			# Make it fully transparent
			new_data.append((0, 0, 0, 0))
		else:
			new_data.append(pixel)

	tile.putdata(new_data)

	# Save
	output_path = os.path.join(output_dir, filename)
	tile.save(output_path)
	print(f"Saved: {output_path} ({tile.size[0]}x{tile.size[1]})")

print("Done!")
