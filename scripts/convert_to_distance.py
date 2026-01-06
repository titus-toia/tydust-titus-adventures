#!/usr/bin/env python3
"""Convert level YAML from time-based to distance-based spawning."""

import yaml
import sys
import shutil
from pathlib import Path

def build_time_to_distance_map(phases):
	"""Build a mapping of time -> cumulative distance based on phases."""
	time_points = []
	cumulative_distance = 0.0

	for phase in phases:
		start_time = phase['start_time']
		end_time = phase['end_time']
		scroll_speed = phase['scroll_speed']
		duration = end_time - start_time

		# Distance at start of this phase
		start_distance = cumulative_distance
		# Distance at end of this phase
		end_distance = cumulative_distance + (duration * scroll_speed)

		time_points.append({
			'start_time': start_time,
			'end_time': end_time,
			'start_distance': start_distance,
			'end_distance': end_distance,
			'scroll_speed': scroll_speed,
		})

		cumulative_distance = end_distance

	return time_points

def time_to_distance(t, time_points):
	"""Convert a time value to distance based on phase scroll speeds."""
	for phase in time_points:
		if phase['start_time'] <= t < phase['end_time']:
			# Time within this phase
			time_in_phase = t - phase['start_time']
			return phase['start_distance'] + (time_in_phase * phase['scroll_speed'])

	# If past all phases, use last phase's end distance + extrapolation
	if time_points:
		last = time_points[-1]
		if t >= last['end_time']:
			time_past = t - last['end_time']
			return last['end_distance'] + (time_past * last['scroll_speed'])

	return t * 100.0  # Fallback

def convert_level(data):
	"""Convert level data from time-based to distance-based."""
	phases = data.get('phases', [])
	time_map = build_time_to_distance_map(phases)

	# Calculate total distance
	if time_map:
		total_distance = time_map[-1]['end_distance']
	else:
		total_distance = data.get('duration', 180.0) * 100.0

	# Replace duration with total_distance
	data['total_distance'] = round(total_distance, 1)
	if 'duration' in data:
		del data['duration']

	# Convert phases
	for i, phase in enumerate(phases):
		if i < len(time_map):
			phase['start_distance'] = round(time_map[i]['start_distance'], 1)
			phase['end_distance'] = round(time_map[i]['end_distance'], 1)
			del phase['start_time']
			del phase['end_time']

	# Convert enemy waves
	for wave in data.get('enemy_waves', []):
		if 'spawn_time' in wave:
			wave['spawn_distance'] = round(time_to_distance(wave['spawn_time'], time_map), 1)
			del wave['spawn_time']

	# Convert doodads
	for doodad in data.get('doodads', []):
		if 'spawn_time' in doodad:
			doodad['spawn_distance'] = round(time_to_distance(doodad['spawn_time'], time_map), 1)
			del doodad['spawn_time']

	# Convert events
	for event in data.get('events', []):
		if 'time' in event:
			event['distance'] = round(time_to_distance(event['time'], time_map), 1)
			del event['time']

	# Convert tutorials
	for tutorial in data.get('tutorials', []):
		if 'time' in tutorial:
			tutorial['distance'] = round(time_to_distance(tutorial['time'], time_map), 1)
			del tutorial['time']
		if 'duration' in tutorial:
			# Convert duration to display_distance (assume average scroll speed)
			avg_speed = total_distance / data.get('duration', 180.0) if 'duration' in data else 100.0
			tutorial['display_distance'] = round(tutorial['duration'] * 100.0, 1)  # ~100 gu/s default
			del tutorial['duration']

	return data

def main():
	if len(sys.argv) < 2:
		input_file = Path("assets/level-defs/level1.yaml")
	else:
		input_file = Path(sys.argv[1])

	if not input_file.exists():
		print(f"Error: {input_file} not found")
		sys.exit(1)

	# Backup original
	backup_file = input_file.with_suffix('.yaml.time_backup')
	shutil.copy(input_file, backup_file)
	print(f"Backed up original to {backup_file}")

	# Load YAML
	with open(input_file, 'r') as f:
		data = yaml.safe_load(f)

	# Store original duration for reference
	original_duration = data.get('duration', 180.0)

	# Convert
	converted = convert_level(data)

	# Write back
	with open(input_file, 'w') as f:
		yaml.dump(converted, f, default_flow_style=None, sort_keys=False, allow_unicode=True)

	print(f"Converted {input_file}")
	print(f"  Original duration: {original_duration}s")
	print(f"  Total distance: {converted['total_distance']} gu")
	print(f"  Phases: {len(converted.get('phases', []))}")
	print(f"  Enemy waves: {len(converted.get('enemy_waves', []))}")
	print(f"  Doodads: {len(converted.get('doodads', []))}")

if __name__ == "__main__":
	main()
