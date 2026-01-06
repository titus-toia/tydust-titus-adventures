#!/usr/bin/env python3
"""
Enhanced Pseudo-3D Doodad Generator
Applies industrial-armature aesthetic with cast shadows, specular highlights,
directional lighting, and material-specific details.
"""

from PIL import Image, ImageDraw
import os
import glob
from pathlib import Path

# Color Palettes - Industrial Aesthetic
STEEL = [
	(35, 40, 50),    # Very dark
	(65, 75, 90),    # Dark
	(110, 125, 145), # Mid
	(180, 195, 220)  # Light
]

COPPER = [
	(90, 60, 40),    # Dark copper
	(140, 90, 60),   # Mid copper
	(180, 120, 80),  # Bright copper
]

ALUMINUM = [
	(120, 125, 130),
	(160, 165, 170),
	(200, 205, 210)
]

SHADOW = (10, 15, 20, 100)  # Semi-transparent shadow
SPECULAR = (240, 250, 255)  # Bright highlight
GLOW = (200, 150, 50, 150)  # Warm glow for energy/lights

# Enhanced sprite categories with specific lighting rules
CATEGORIES = {
	'drone': {
		'palette': STEEL,
		'glow_color': (100, 200, 255, 120),  # Cyan thruster
		'description': 'Add thruster glow, sensor dome highlights'
	},
	'fuel_tank': {
		'palette': STEEL,
		'hazard': True,
		'description': 'Cylindrical shading, hazard stripe highlights'
	},
	'solar_panel': {
		'palette': ALUMINUM,
		'glass_reflection': True,
		'description': 'Glass reflection gradients, frame depth'
	},
	'solar_broken': {
		'palette': COPPER,
		'torn_edges': True,
		'description': 'Torn edges with depth, dangling cables'
	},
	'antenna': {
		'palette': STEEL,
		'thin_shaft': True,
		'description': 'Metallic shaft with specular, highlights'
	},
	'sparking': {
		'palette': COPPER,
		'arc_bloom': True,
		'description': 'Bright arc bloom, metal fragment shadows'
	},
	'wreckage': {
		'palette': COPPER,
		'torn_hull': True,
		'description': 'Torn hull with internal shadows'
	},
	'hull_fragment': {
		'palette': STEEL,
		'panel_lines': True,
		'description': 'Panel line depth, torn edges'
	},
	'gas_vent': {
		'palette': STEEL,
		'gas_bloom': True,
		'description': 'Escaping gas bloom effect'
	},
	'beacon': {
		'palette': STEEL,
		'light_bloom': True,
		'description': 'Pulsing light bloom, cylindrical shading'
	},
	'trail': {
		'palette': COPPER,
		'gradient_fade': True,
		'description': 'Gradient fade with glow'
	},
	'distant_asteroid': {
		'palette': STEEL,
		'dim_lighting': True,
		'description': 'Subtle crater depth, dim lighting'
	},
	'nav_buoy': {
		'palette': ALUMINUM,
		'light_bloom': True,
		'description': 'Light beacon bloom, geometric body'
	},
	'hull_debris': {
		'palette': COPPER,
		'torn_hull': True,
		'description': 'Twisted metal with exposed structure'
	},
	'escape_pod': {
		'palette': STEEL,
		'light_bloom': True,
		'description': 'Capsule with window, thruster glow'
	},
	'satellite': {
		'palette': ALUMINUM,
		'glass_reflection': True,
		'description': 'Antenna specular, panel highlights'
	},
	'cargo': {
		'palette': STEEL,
		'hazard': True,
		'description': 'Perspective depth, warning stripes'
	},
	'asteroid': {
		'palette': STEEL,
		'craters': True,
		'description': 'Rocky surface with crater depth'
	},
}

def add_cast_shadow(img_array, width, height):
	"""Add semi-transparent cast shadow beneath object"""
	shadow_img = Image.new('RGBA', (width, height), (0, 0, 0, 0))
	shadow_draw = ImageDraw.Draw(shadow_img)

	# Shadow ellipse offset down-right
	shadow_x, shadow_y = 4, 6
	shadow_w = width * 0.8
	shadow_h = height * 0.3

	cx, cy = width // 2, height // 2
	shadow_draw.ellipse(
		[cx - shadow_w//2 + shadow_x, cy + shadow_h//2 + shadow_y,
		 cx + shadow_w//2 + shadow_x, cy + shadow_h + shadow_y],
		fill=SHADOW
	)

	shadow_img.paste(img_array, (0, 0), img_array)
	return shadow_img

def add_specular_highlights(img, num_highlights=2):
	"""Add bright specular highlights to edges"""
	draw = ImageDraw.Draw(img)
	width, height = img.size

	# Top-left edge highlights (light source from upper-left)
	highlight_positions = [
		(width // 4, height // 4),
		(width // 3, height // 3),
	]

	for x, y in highlight_positions[:num_highlights]:
		draw.ellipse([x-3, y-3, x+3, y+3], fill=SPECULAR)

	return img

def add_directional_lighting(img):
	"""Apply directional lighting gradient overlay"""
	width, height = img.size
	overlay = Image.new('RGBA', (width, height), (0, 0, 0, 0))

	# Create gradient from light (top-left) to dark (bottom-right)
	for y in range(height):
		for x in range(width):
			# Normalized position
			nx = x / width
			ny = y / height

			# Darker in bottom-right, lighter in top-left
			brightness = int(50 * (1 - (nx + ny) * 0.3))
			shade = max(0, min(255, brightness))

			overlay.putpixel((x, y), (0, 0, 0, min(50, shade // 3)))

	img = Image.alpha_composite(img, overlay)
	return img

def add_glow_effect(img, glow_color, intensity=0.3):
	"""Add bloom/glow effect to bright areas"""
	width, height = img.size
	glow = Image.new('RGBA', (width, height), (0, 0, 0, 0))
	draw = ImageDraw.Draw(glow)

	# Add glow spots at strategic locations
	for x in range(width // 4, width * 3 // 4, width // 3):
		for y in range(height // 4, height * 3 // 4, height // 3):
			draw.ellipse([x-8, y-8, x+8, y+8], fill=glow_color)

	return Image.blend(img, glow, intensity)

def enhance_sprite(input_path, output_path, category_key='default'):
	"""Apply pseudo-3D enhancement to base sprite"""
	try:
		# Load base sprite
		img = Image.open(input_path).convert('RGBA')
		width, height = img.size

		# Add cast shadow
		img = add_cast_shadow(img, width, height)

		# Add specular highlights
		img = add_specular_highlights(img, num_highlights=2)

		# Add directional lighting
		img = add_directional_lighting(img)

		# Category-specific enhancements
		if category_key in CATEGORIES:
			cat = CATEGORIES[category_key]

			if cat.get('light_bloom'):
				glow_color = (200, 150, 100, 100)
				img = add_glow_effect(img, glow_color, 0.15)

			if cat.get('arc_bloom'):
				glow_color = (255, 200, 100, 120)
				img = add_glow_effect(img, glow_color, 0.2)

			if cat.get('gas_bloom'):
				glow_color = (100, 200, 255, 80)
				img = add_glow_effect(img, glow_color, 0.15)

		# Save enhanced sprite
		img.save(output_path, 'PNG')
		return True

	except Exception as e:
		print(f"Error enhancing {input_path}: {e}")
		return False

def get_category_from_filename(filename):
	"""Extract category key from filename"""
	base = filename.replace('_1.png', '').replace('_2.png', '').replace('_3.png', '').replace('.png', '')

	for cat_key in CATEGORIES.keys():
		if cat_key.replace('_', '') in base.replace('_', ''):
			return cat_key

	return 'default'

def process_all_doodads():
	"""Process all base doodads in the doodads directory"""
	base_dir = '/home/titus/tydust/level-1/doodads'
	enhanced_dir = os.path.join(base_dir, 'enhanced')

	# Create enhanced directory if needed
	os.makedirs(enhanced_dir, exist_ok=True)

	# Find all base sprites (not in enhanced folder)
	base_sprites = []
	for png_file in glob.glob(os.path.join(base_dir, '*.png')):
		if 'enhanced' not in png_file and '3d' not in png_file:
			base_sprites.append(png_file)

	base_sprites.sort()

	processed = 0
	skipped = 0

	for input_path in base_sprites:
		filename = os.path.basename(input_path)
		output_path = os.path.join(enhanced_dir, filename)

		# Skip if already enhanced
		if os.path.exists(output_path):
			print(f"Skipping (already exists): {filename}")
			skipped += 1
			continue

		# Get category for specialized enhancements
		category = get_category_from_filename(filename)

		if enhance_sprite(input_path, output_path, category):
			print(f"Enhanced: {filename} (category: {category})")
			processed += 1
		else:
			print(f"Failed: {filename}")

	print(f"\n--- Summary ---")
	print(f"Processed: {processed} sprites")
	print(f"Skipped: {skipped} sprites (already enhanced)")
	print(f"Output directory: {enhanced_dir}")

if __name__ == '__main__':
	process_all_doodads()
