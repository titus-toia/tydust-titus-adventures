#!/usr/bin/env python3
"""
Isolate planet from black space background
Preserves planet, atmospheric glow, and stars while removing deep space
"""

import sys
from PIL import Image
import numpy as np

def isolate_planet(input_path, output_path):
	print(f"Loading: {input_path}")
	img = Image.open(input_path).convert("RGBA")
	print(f"Original size: {img.size[0]}×{img.size[1]}")

	data = np.array(img)
	r, g, b, a = data[:,:,0], data[:,:,1], data[:,:,2], data[:,:,3]

	# Calculate brightness
	brightness = (r.astype(int) + g.astype(int) + b.astype(int)) / 3

	# Strategy: Detect the planet using its blue atmospheric glow
	# The planet has elevated blue values compared to pure black space

	# Find pixels with strong blue glow (the atmospheric ring, not stars)
	# Stars are white (all channels high), glow is blue (blue > red/green)
	is_blue_dominant = (b > r + 20) & (b > g + 20) & (b > 50)
	has_strong_glow = is_blue_dominant | ((b > 80) & (brightness > 60))

	# Find the center of the glowing region
	y_coords, x_coords = np.where(has_strong_glow)
	if len(x_coords) > 100:  # Need enough pixels to be reliable
		center_x = int(np.median(x_coords))
		center_y = int(np.median(y_coords))
		print(f"Planet center detected at: ({center_x}, {center_y})")

		# Calculate distance from center
		height, width = brightness.shape
		y_grid, x_grid = np.ogrid[:height, :width]
		distances = np.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2)

		# Find 95th percentile radius of glow (ignores outlier stars)
		glow_distances = distances[has_strong_glow]
		planet_radius = np.percentile(glow_distances, 95)
		print(f"Planet radius (95th percentile): {planet_radius:.1f}px")

		# Keep planet + glow, remove dark corners beyond radius
		is_background = (distances > planet_radius + 50) & (brightness < 25)
	else:
		# Fallback: just remove very dark pixels
		is_background = brightness < 15

	# Set alpha to 0 for background
	data[is_background, 3] = 0

	# Count what we removed
	removed = np.sum(is_background)
	total = brightness.size
	print(f"Removing {removed}/{total} pixels ({100*removed/total:.1f}%)")

	# Create new image
	img_cleaned = Image.fromarray(data, 'RGBA')

	# Crop to non-transparent content
	bbox = img_cleaned.getbbox()
	if bbox:
		img_cropped = img_cleaned.crop(bbox)
		print(f"Cropped to: {img_cropped.size[0]}×{img_cropped.size[1]}")

		# Check final transparency
		final_data = np.array(img_cropped)
		alpha = final_data[:,:,3]
		transparent = np.sum(alpha < 255)
		total = alpha.size
		print(f"Final transparent pixels: {transparent}/{total} ({100*transparent/total:.1f}%)")

		# Save
		img_cropped.save(output_path)
		print(f"✓ Saved to: {output_path}")
	else:
		print("✗ No content found after processing")
		sys.exit(1)

if __name__ == "__main__":
	if len(sys.argv) < 2:
		print("Usage: python isolate_planet.py <input.png> [output.png]")
		sys.exit(1)

	input_path = sys.argv[1]
	output_path = sys.argv[2] if len(sys.argv) > 2 else "planet_isolated.png"

	isolate_planet(input_path, output_path)
