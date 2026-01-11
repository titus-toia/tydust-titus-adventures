#!/usr/bin/env python3
"""
Isolate star cluster from black space background
Detects dense stellar concentration and removes dark edges
"""

import sys
from PIL import Image
import numpy as np

def isolate_star_cluster(input_path, output_path):
	print(f"Loading: {input_path}")
	img = Image.open(input_path).convert("RGBA")
	print(f"Original size: {img.size[0]}×{img.size[1]}")

	data = np.array(img)
	r, g, b, a = data[:,:,0], data[:,:,1], data[:,:,2], data[:,:,3]

	# Calculate brightness
	brightness = (r.astype(int) + g.astype(int) + b.astype(int)) / 3

	# Strategy: Find the star cluster by detecting bright pixels (stars)
	# Stars are bright white/blue pixels
	is_star = brightness > 100  # Bright pixels

	# Find the bounding area of star concentration
	# Use morphological operations to find the cluster region
	from scipy import ndimage

	# Dilate the star map to connect nearby stars into regions
	star_regions = ndimage.binary_dilation(is_star, iterations=10)

	# Find the largest connected component (main cluster)
	labeled, num_features = ndimage.label(star_regions)
	if num_features > 0:
		# Find largest region
		sizes = ndimage.sum(star_regions, labeled, range(num_features + 1))
		largest_label = np.argmax(sizes)
		cluster_mask = labeled == largest_label

		print(f"Found star cluster with {num_features} regions")

		# Find center of cluster
		y_coords, x_coords = np.where(cluster_mask)
		center_x = int(np.median(x_coords))
		center_y = int(np.median(y_coords))
		print(f"Cluster center: ({center_x}, {center_y})")

		# Calculate distances from center
		height, width = brightness.shape
		y_grid, x_grid = np.ogrid[:height, :width]
		distances = np.sqrt((x_grid - center_x)**2 + (y_grid - center_y)**2)

		# Find the radius that contains most stars
		star_distances = distances[is_star]
		cluster_radius = np.percentile(star_distances, 95)
		print(f"Cluster radius (95th percentile): {cluster_radius:.1f}px")

		# Remove dark space beyond the cluster with some padding
		is_background = (distances > cluster_radius + 50) & (brightness < 30)
	else:
		# Fallback: just remove very dark pixels
		print("Could not detect cluster, using brightness threshold")
		is_background = brightness < 20

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
		print("Usage: python isolate_star_cluster.py <input.png> [output.png]")
		sys.exit(1)

	input_path = sys.argv[1]
	output_path = sys.argv[2] if len(sys.argv) > 2 else "cluster_isolated.png"

	isolate_star_cluster(input_path, output_path)
