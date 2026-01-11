#!/usr/bin/env python3
"""
Asset QA Pipeline - Run after generating new assets
Automatically detects and fixes common issues with AI-generated images
"""

import os
import sys
from PIL import Image
import numpy as np
from pathlib import Path

class Colors:
	HEADER = '\033[95m'
	OKBLUE = '\033[94m'
	OKCYAN = '\033[96m'
	OKGREEN = '\033[92m'
	WARNING = '\033[93m'
	FAIL = '\033[91m'
	ENDC = '\033[0m'
	BOLD = '\033[1m'

def log_info(msg):
	print(f"{Colors.OKBLUE}ℹ {msg}{Colors.ENDC}")

def log_success(msg):
	print(f"{Colors.OKGREEN}✓ {msg}{Colors.ENDC}")

def log_warning(msg):
	print(f"{Colors.WARNING}⚠ {msg}{Colors.ENDC}")

def log_error(msg):
	print(f"{Colors.FAIL}✗ {msg}{Colors.ENDC}")

def check_white_background(filepath, threshold=220):
	"""Check if image has significant white/light pixels"""
	img = Image.open(filepath).convert('RGBA')
	data = list(img.getdata())

	white_pixels = sum(1 for p in data if p[0] > threshold and p[1] > threshold and p[2] > threshold and p[3] > 0)
	total_pixels = len(data)
	white_percent = (white_pixels / total_pixels) * 100

	return white_percent > 5, white_percent, white_pixels

def fix_white_background(filepath, threshold=220):
	"""Remove white/light backgrounds from PNG

	NOTE: For better background removal, use the 'remove-background' skill instead.
	This function only does simple threshold-based removal for quick QA checks.
	"""
	img = Image.open(filepath).convert('RGBA')
	data = list(img.getdata())

	new_data = []
	fixed_count = 0
	for p in data:
		if p[0] > threshold and p[1] > threshold and p[2] > threshold:
			new_data.append((0, 0, 0, 0))
			fixed_count += 1
		else:
			new_data.append(p)

	img.putdata(new_data)
	img.save(filepath)
	return fixed_count

def check_hard_edges(filepath):
	"""Check if image has hard edges (high alpha at borders)"""
	img = Image.open(filepath).convert('RGBA')
	data = np.array(img)
	height, width = data.shape[:2]

	# Sample edge pixels
	top_edge = data[0, :, 3].mean()
	bottom_edge = data[-1, :, 3].mean()
	left_edge = data[:, 0, 3].mean()
	right_edge = data[:, -1, 3].mean()

	avg_edge_alpha = (top_edge + bottom_edge + left_edge + right_edge) / 4

	return avg_edge_alpha > 200, avg_edge_alpha

def feather_edges(filepath, feather_size=80):
	"""Add alpha feathering to edges"""
	img = Image.open(filepath).convert('RGBA')
	width, height = img.size

	data = np.array(img, dtype=np.float32)
	mask = np.ones((height, width), dtype=np.float32)

	for i in range(feather_size):
		alpha = i / feather_size
		mask[i, :] = np.minimum(mask[i, :], alpha)
		mask[height - 1 - i, :] = np.minimum(mask[height - 1 - i, :], alpha)
		mask[:, i] = np.minimum(mask[:, i], alpha)
		mask[:, width - 1 - i] = np.minimum(mask[:, width - 1 - i], alpha)

	data[:, :, 3] = data[:, :, 3] * mask

	result = Image.fromarray(data.astype(np.uint8), 'RGBA')
	result.save(filepath)

def check_file_size(filepath):
	"""Check if file is suspiciously large"""
	size_bytes = os.path.getsize(filepath)
	size_kb = size_bytes / 1024
	size_mb = size_kb / 1024

	# Warn if over 2MB (raw AI output)
	is_large = size_kb > 2048

	return is_large, size_kb, size_mb

def process_asset(filepath, asset_type, auto_fix=True):
	"""Run all QA checks on a single asset"""
	issues = []
	fixes_applied = []

	filename = os.path.basename(filepath)

	# Check 1: White background
	has_white_bg, white_percent, white_pixels = check_white_background(filepath)
	if has_white_bg:
		issues.append(f"White background: {white_percent:.1f}% ({white_pixels} pixels)")
		if auto_fix:
			fixed = fix_white_background(filepath)
			fixes_applied.append(f"Removed {fixed} white pixels")

	# Check 2: Hard edges (only for backdrop/far parallax layers)
	# Skip feathering for cosmic phenomena (nebulae, dust, stars) - black space is part of the image
	cosmic_keywords = ['nebula', 'dust', 'star', 'supernova', 'cloud', 'cosmic', 'stellar']
	is_cosmic = any(keyword in filename.lower() for keyword in cosmic_keywords)

	if asset_type in ['backdrop', 'far', 'backgrounds'] and not is_cosmic:
		has_hard_edges, avg_alpha = check_hard_edges(filepath)
		if has_hard_edges:
			issues.append(f"Hard edges: avg edge alpha {avg_alpha:.0f}")
			if auto_fix:
				feather_edges(filepath, feather_size=80)
				fixes_applied.append(f"Feathered edges (80px)")

	# Check 3: File size
	is_large, size_kb, size_mb = check_file_size(filepath)
	if is_large:
		issues.append(f"Large file: {size_mb:.2f} MB")
		# Don't auto-fix size issues, just warn

	return issues, fixes_applied

def scan_directory(directory, asset_type, auto_fix=True):
	"""Scan all PNGs in a directory"""
	print(f"\n{Colors.BOLD}=== Scanning {asset_type}: {directory} ==={Colors.ENDC}")

	if not os.path.exists(directory):
		log_error(f"Directory not found: {directory}")
		return

	png_files = [f for f in os.listdir(directory) if f.endswith('.png')]

	if not png_files:
		log_info(f"No PNG files found")
		return

	total_issues = 0
	total_fixes = 0

	for filename in sorted(png_files):
		filepath = os.path.join(directory, filename)
		issues, fixes = process_asset(filepath, asset_type, auto_fix)

		if issues:
			total_issues += len(issues)
			log_warning(f"{filename}")
			for issue in issues:
				print(f"  - {issue}")
			if fixes:
				total_fixes += len(fixes)
				for fix in fixes:
					print(f"  {Colors.OKGREEN}→ {fix}{Colors.ENDC}")
		else:
			log_success(f"{filename} - OK")

	print(f"\n{Colors.BOLD}Summary:{Colors.ENDC} {len(png_files)} files, {total_issues} issues, {total_fixes} fixes applied")

def main():
	print(f"{Colors.HEADER}{Colors.BOLD}")
	print("╔════════════════════════════════════════╗")
	print("║     Asset QA Pipeline v1.0             ║")
	print("║  Auto-fix AI-generated image issues    ║")
	print("╚════════════════════════════════════════╝")
	print(Colors.ENDC)

	# Get project root (go up two directories from scripts/asset-processing/)
	script_dir = Path(__file__).parent
	project_root = script_dir.parent.parent
	assets_dir = project_root / "assets"

	if not assets_dir.exists():
		log_error(f"Assets directory not found: {assets_dir}")
		sys.exit(1)

	# Check if running in auto-fix mode
	auto_fix = "--no-fix" not in sys.argv
	if not auto_fix:
		log_info("Running in CHECK-ONLY mode (use --fix to apply fixes)")
	else:
		log_info("Running in AUTO-FIX mode (use --no-fix to only check)")

	# Scan each asset type
	directories = [
		(assets_dir / "backdrop", "backdrop"),
		(assets_dir / "far", "far"),
		(assets_dir / "backgrounds", "backgrounds"),
		(assets_dir / "doodads", "doodads"),
		(assets_dir / "structures", "structures"),
		(assets_dir / "sprites" / "ships", "ships"),
		(assets_dir / "particles", "particles"),
	]

	for directory, asset_type in directories:
		if directory.exists():
			scan_directory(str(directory), asset_type, auto_fix)

	print(f"\n{Colors.OKGREEN}{Colors.BOLD}✓ Asset QA pipeline complete!{Colors.ENDC}")

if __name__ == "__main__":
	main()
