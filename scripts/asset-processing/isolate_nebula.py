#!/usr/bin/env python3
"""
Isolate nebula from black space background
Detects colored gas clouds and removes dark space fluff
"""

import sys
from PIL import Image
import numpy as np

def isolate_nebula(input_path, output_path):
	print(f"Loading: {input_path}")
	img = Image.open(input_path).convert("RGBA")
	print(f"Original size: {img.size[0]}×{img.size[1]}")

	data = np.array(img)
	r, g, b, a = data[:,:,0], data[:,:,1], data[:,:,2], data[:,:,3]

	# Calculate brightness and color intensity
	brightness = (r.astype(int) + g.astype(int) + b.astype(int)) / 3

	# Nebulae have color variation and moderate brightness
	# Not pure black (space) or pure white (stars)
	color_variance = np.abs(r.astype(int) - g.astype(int)) + np.abs(g.astype(int) - b.astype(int)) + np.abs(b.astype(int) - r.astype(int))

	# Strategy: Find nebula pixels by detecting colored regions
	# Nebula: has color (variance > 30) OR moderate-to-bright (50-220)
	# Stars: very bright (>220) - we'll keep these too if near nebula
	# Space: very dark (<50) with no color
	is_nebula = (color_variance > 30) | ((brightness > 50) & (brightness < 220))

	# Find the bounding area of nebula concentration
	try:
		from scipy import ndimage

		# Dilate the nebula map to connect nearby regions
		nebula_regions = ndimage.binary_dilation(is_nebula, iterations=15)

		# Find the largest connected component (main nebula)
		labeled, num_features = ndimage.label(nebula_regions)
		if num_features > 0:
			# Find largest region
			sizes = ndimage.sum(nebula_regions, labeled, range(num_features + 1))
			largest_label = np.argmax(sizes)
			nebula_mask = labeled == largest_label

			print(f"Found nebula with {num_features} regions")

			# Find center of nebula
			y_coords, x_coords = np.where(nebula_mask)
			center_x = int(np.median(x_coords))
			center_y = int(np.median(y_coords))
			print(f"Nebula center: ({center_x}, {center_y})")

			# Calculate distances from center
			height, width = brightness.shape
			y_grid, x_grid = np.ogrid[:height, :width]
			distances = np.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2)

			# Find the radius that contains core nebula (more aggressive to remove fluff)
			nebula_distances = distances[is_nebula]
			nebula_radius = np.percentile(nebula_distances, 85)
			print(f"Nebula radius (85th percentile): {nebula_radius:.1f}px")

			# Remove dark space beyond the nebula core
			# More aggressive: no padding, higher brightness threshold
			is_background = (distances > nebula_radius) & (brightness < 60)
		else:
			# Fallback: just remove very dark pixels
			print("Could not detect nebula regions, using brightness threshold")
			is_background = brightness < 40
	except ImportError:
		# Fallback if scipy not available
		print("scipy not available, using simple brightness threshold")
		is_background = brightness < 40

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
		print("Usage: python isolate_nebula.py <input.png> [output.png]")
		sys.exit(1)

	input_path = sys.argv[1]
	output_path = sys.argv[2] if len(sys.argv) > 2 else "nebula_isolated.png"

	isolate_nebula(input_path, output_path)
