#!/usr/bin/env python3
"""Scale level YAML positions from 768-unit to 1000-unit coordinate system."""

import yaml
import sys
import shutil
from pathlib import Path

SCALE_FACTOR = 1000.0 / 768.0  # ~1.302

def scale_position(pos):
	"""Scale a [x, y] position array."""
	return [round(pos[0] * SCALE_FACTOR, 1), round(pos[1] * SCALE_FACTOR, 1)]

def scale_velocity(vel):
	"""Scale a [vx, vy] velocity array."""
	return [round(vel[0] * SCALE_FACTOR, 1), round(vel[1] * SCALE_FACTOR, 1)]

def scale_level_data(data):
	"""Scale all positions and velocities in level data."""

	# Scale enemy waves
	if 'enemy_waves' in data:
		for wave in data['enemy_waves']:
			if 'enemies' in wave:
				for enemy in wave['enemies']:
					if 'position' in enemy:
						enemy['position'] = scale_position(enemy['position'])

	# Scale doodads
	if 'doodads' in data:
		for doodad in data['doodads']:
			if 'position' in doodad:
				doodad['position'] = scale_position(doodad['position'])
			if 'velocity' in doodad:
				doodad['velocity'] = scale_velocity(doodad['velocity'])

	# Scale events with positions (like BackgroundExplosion)
	if 'events' in data:
		for event in data['events']:
			if 'position' in event:
				event['position'] = scale_position(event['position'])

	# Scale phase scroll speeds
	if 'phases' in data:
		for phase in data['phases']:
			if 'scroll_speed' in phase:
				phase['scroll_speed'] = round(phase['scroll_speed'] * SCALE_FACTOR, 1)

	return data

def main():
	if len(sys.argv) < 2:
		input_file = Path("assets/levels/level_1.yaml")
	else:
		input_file = Path(sys.argv[1])

	if not input_file.exists():
		print(f"Error: {input_file} not found")
		sys.exit(1)

	# Backup original
	backup_file = input_file.with_suffix('.yaml.bak')
	shutil.copy(input_file, backup_file)
	print(f"Backed up to {backup_file}")

	# Load YAML
	with open(input_file, 'r') as f:
		data = yaml.safe_load(f)

	# Count items before scaling
	enemy_count = sum(len(w.get('enemies', [])) for w in data.get('enemy_waves', []))
	doodad_count = len(data.get('doodads', []))

	# Scale
	scaled_data = scale_level_data(data)

	# Write back
	with open(input_file, 'w') as f:
		yaml.dump(scaled_data, f, default_flow_style=None, sort_keys=False, allow_unicode=True)

	print(f"Scaled {enemy_count} enemies and {doodad_count} doodads")
	print(f"Scale factor: {SCALE_FACTOR:.3f} (768 -> 1000 units)")
	print(f"Updated {input_file}")

if __name__ == "__main__":
	main()
