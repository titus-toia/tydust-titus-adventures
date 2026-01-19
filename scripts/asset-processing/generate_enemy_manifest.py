#!/usr/bin/env python3
"""
Generate enemy sprite metadata manifest from alpha bounds.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
from PIL import Image
import yaml
import re


Bounds = Tuple[int, int, int, int]


@dataclass(frozen=True)
class EnemySource:
    name: str
    sprite_path: str
    gameplay_height_gu: float
    frames: Optional[List[str]] = None
    collision_shape: Optional[str] = None
    collision_scale: float = 1.0


ENEMIES: List[EnemySource] = [
    EnemySource("Scout", "enemies/scout.png", 60.0),
    EnemySource("ScoutSting", "enemies/scout_sting.png", 64.0, collision_shape="ellipse"),
    EnemySource("Fighter", "enemies/fighter.png", 80.0),
    EnemySource("HeavyGunship", "enemies/heavy_gunship.png", 150.0, collision_shape="ellipse"),
    EnemySource("Boss", "enemies/boss.png", 300.0, collision_shape="ellipse"),
    EnemySource("Interceptor", "enemies/interceptor.png", 50.0),
    EnemySource("Drone", "enemies/drone.png", 40.0),
    EnemySource("Bomber", "enemies/bomber.png", 120.0, collision_shape="ellipse"),
    EnemySource("Corvette", "enemies/corvette.png", 180.0, collision_shape="ellipse"),
    EnemySource(
        "Drill",
        "enemies/drill/drill_0.png",
        90.0,
        frames=[f"enemies/drill/drill_{i}.png" for i in range(6)],
        collision_shape="ellipse",
    ),
    EnemySource("SmallAsteroid", "enemies/small_asteroid.png", 30.0, collision_shape="circle"),
    EnemySource("MediumAsteroid", "enemies/medium_asteroid.png", 60.0, collision_shape="circle"),
    EnemySource("LargeAsteroid", "enemies/large_asteroid.png", 120.0, collision_shape="circle"),
    EnemySource("Asteroid", "enemies/large_asteroid.png", 120.0, collision_shape="circle"),
    EnemySource("AsteroidTurret", "enemies/asteroid_turret.png", 130.0, collision_shape="circle"),

    EnemySource("StationDebris", "enemies/station_debris.png", 80.0, collision_shape="circle"),
    EnemySource("Ironclad", "enemies/ironclad.png", 75.0, collision_shape="ellipse"),
]

AUTO_GAMEPLAY_SCALE = 0.3


def load_image(path: Path) -> Image.Image:
    return Image.open(path).convert("RGBA")


def compute_bounds(img: Image.Image, alpha_threshold: int) -> Optional[Bounds]:
    alpha = np.array(img)[:, :, 3]
    mask = alpha >= alpha_threshold
    if not mask.any():
        return None
    ys, xs = np.where(mask)
    return int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())


def union_bounds(bounds_list: List[Bounds]) -> Bounds:
    min_x = min(b[0] for b in bounds_list)
    min_y = min(b[1] for b in bounds_list)
    max_x = max(b[2] for b in bounds_list)
    max_y = max(b[3] for b in bounds_list)
    return min_x, min_y, max_x, max_y


def bounds_size(bounds: Bounds) -> Tuple[int, int]:
    return bounds[2] - bounds[0] + 1, bounds[3] - bounds[1] + 1


def center_offset_px(bounds: Bounds, texture_size: Tuple[int, int]) -> Tuple[float, float]:
    tex_w, tex_h = texture_size
    tex_cx = (tex_w - 1) / 2.0
    tex_cy = (tex_h - 1) / 2.0
    content_cx = (bounds[0] + bounds[2]) / 2.0
    content_cy = (bounds[1] + bounds[3]) / 2.0
    offset_x = content_cx - tex_cx
    offset_y = tex_cy - content_cy  # convert to +y up
    return round(offset_x, 3), round(offset_y, 3)


def pick_shape(default_shape: Optional[str], size: Tuple[int, int]) -> str:
    if default_shape:
        return default_shape
    w, h = size
    if w == 0 or h == 0:
        return "circle"
    ratio = max(w / h, h / w)
    return "ellipse" if ratio >= 1.2 else "circle"


def format_scalar(value):
    if isinstance(value, str):
        return json.dumps(value)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, float):
        text = f"{value:.4f}".rstrip("0").rstrip(".")
        return text if text else "0"
    return str(value)


def format_list(values):
    return "[" + ", ".join(format_scalar(v) for v in values) + "]"

def dump_list_of_dicts(indent: int, items: List[Dict]) -> List[str]:
	lines: List[str] = []
	prefix = " " * indent
	for item in items:
		lines.append(f"{prefix}-")
		for key, value in item.items():
			if isinstance(value, list):
				lines.append(f"{prefix}  {key}: {format_list(value)}")
			else:
				lines.append(f"{prefix}  {key}: {format_scalar(value)}")
	return lines


def dump_yaml(data: Dict) -> str:
    lines: List[str] = []
    lines.append(f"version: {data['version']}")
    lines.append("enemies:")
    for name, entry in data["enemies"].items():
        lines.append(f"  {name}:")
        for key, value in entry.items():
            if value is None:
                continue
            if isinstance(value, dict):
                lines.append(f"    {key}:")
                for nested_key, nested_value in value.items():
                    if nested_value is None:
                        continue
                    if isinstance(nested_value, list):
                        lines.append(f"      {nested_key}: {format_list(nested_value)}")
                    else:
                        lines.append(f"      {nested_key}: {format_scalar(nested_value)}")
            elif isinstance(value, list) and value and isinstance(value[0], dict):
                lines.append(f"    {key}:")
                lines.extend(dump_list_of_dicts(6, value))
            elif isinstance(value, list):
                lines.append(f"    {key}: {format_list(value)}")
            else:
                lines.append(f"    {key}: {format_scalar(value)}")
    return "\n".join(lines) + "\n"

def to_pascal_case(value: str) -> str:
	parts = re.split(r"[^a-zA-Z0-9]+", value)
	parts = [part for part in parts if part]
	if not parts:
		return "Unknown"
	return "".join(part[:1].upper() + part[1:] for part in parts)


def build_known_paths() -> Tuple[Dict[str, EnemySource], set]:
	known_paths = set()
	known_sources = {}
	for enemy in ENEMIES:
		known_sources[enemy.name] = enemy
		known_paths.add(enemy.sprite_path)
		if enemy.frames:
			for frame in enemy.frames:
				known_paths.add(frame)
	return known_sources, known_paths


def discover_enemy_sources(assets_root: Path, known_paths: set, known_names: set) -> List[EnemySource]:
	enemies_dir = assets_root / "enemies"
	if not enemies_dir.exists():
		return []

	all_pngs = sorted(enemies_dir.rglob("*.png"))
	unknown_paths = []
	for path in all_pngs:
		rel_path = path.relative_to(assets_root).as_posix()
		if rel_path in known_paths:
			continue
		unknown_paths.append(rel_path)

	grouped: Dict[str, List[Tuple[int, str]]] = {}
	singles: List[str] = []
	for rel_path in unknown_paths:
		stem = Path(rel_path).stem
		match = re.match(r"^(.*)_([0-9]+)$", stem)
		if match:
			base = match.group(1)
			index = int(match.group(2))
			key = f"{Path(rel_path).parent.as_posix()}/{base}"
			grouped.setdefault(key, []).append((index, rel_path))
		else:
			singles.append(rel_path)

	entries: List[EnemySource] = []
	used_paths = set()

	for base, frames in grouped.items():
		if len(frames) < 2:
			continue
		frames_sorted = [path for _, path in sorted(frames, key=lambda item: item[0])]
		used_paths.update(frames_sorted)
		name_seed = base.replace("/", "_")
		name = to_pascal_case(name_seed)
		unique_name = name
		suffix = 2
		while unique_name in known_names:
			unique_name = f"{name}{suffix}"
			suffix += 1
		known_names.add(unique_name)
		entries.append(
			EnemySource(
				unique_name,
				frames_sorted[0],
				gameplay_height_gu=0.0,
				frames=frames_sorted,
			)
		)

	for rel_path in singles:
		if rel_path in used_paths:
			continue
		name_seed = Path(rel_path).with_suffix("").as_posix().replace("/", "_")
		name = to_pascal_case(name_seed)
		unique_name = name
		suffix = 2
		while unique_name in known_names:
			unique_name = f"{name}{suffix}"
			suffix += 1
		known_names.add(unique_name)
		entries.append(
			EnemySource(
				unique_name,
				rel_path,
				gameplay_height_gu=0.0,
			)
		)

	return entries


def merge_entries(generated: Dict, existing: Dict) -> Dict:
	manual_keys = {
		"gameplay_height_gu",
		"collision_shape",
		"collision_scale",
		"sockets",
	}
	for key, value in existing.items():
		if key in manual_keys or key not in generated:
			generated[key] = value
	return generated


def build_manifest(
    assets_root: Path,
    visual_threshold: int,
    collision_threshold: int,
    existing_entries: Dict[str, Dict],
    sources: List[EnemySource],
) -> Dict:
    enemies: Dict[str, Dict] = {}

    for enemy in sources:
        frame_paths = enemy.frames or [enemy.sprite_path]
        resolved_frames = [assets_root / path for path in frame_paths]
        if any(not path.exists() for path in resolved_frames):
            print(f"⚠ Missing frames for {enemy.name}; skipping")
            continue

        frames_visual: List[Bounds] = []
        frames_collision: List[Bounds] = []
        texture_size: Optional[Tuple[int, int]] = None

        for frame_path in resolved_frames:
            img = load_image(frame_path)
            if texture_size is None:
                texture_size = img.size
            elif texture_size != img.size:
                print(f"⚠ Size mismatch in frames for {enemy.name}; skipping")
                texture_size = None
                break

            visual_bounds = compute_bounds(img, visual_threshold)
            collision_bounds = compute_bounds(img, collision_threshold)
            if visual_bounds is None or collision_bounds is None:
                print(f"⚠ No alpha bounds for {frame_path.name}; skipping {enemy.name}")
                texture_size = None
                break

            frames_visual.append(visual_bounds)
            frames_collision.append(collision_bounds)

        if texture_size is None:
            continue

        visual_union = union_bounds(frames_visual)
        collision_union = union_bounds(frames_collision)
        content_size = bounds_size(visual_union)
        collision_size = bounds_size(collision_union)
        content_offset = center_offset_px(visual_union, texture_size)
        collision_offset = center_offset_px(collision_union, texture_size)

        shape = pick_shape(enemy.collision_shape, collision_size)

        gameplay_height_gu = enemy.gameplay_height_gu
        if gameplay_height_gu <= 0.0:
            gameplay_height_gu = max(content_size[1] * AUTO_GAMEPLAY_SCALE, 1.0)

        entry: Dict = {
            "sprite_path": enemy.sprite_path,
            "texture_px": [int(texture_size[0]), int(texture_size[1])],
            "visual_bounds_px": list(visual_union),
            "collision_bounds_px": list(collision_union),
            "content_size_px": [int(content_size[0]), int(content_size[1])],
            "content_center_offset_px": [content_offset[0], content_offset[1]],
            "collision_center_offset_px": [collision_offset[0], collision_offset[1]],
            "gameplay_height_gu": float(gameplay_height_gu),
            "collision_shape": shape,
            "collision_scale": float(enemy.collision_scale),
        }

        if enemy.frames:
            entry["frame_group"] = {
                "frames": list(frame_paths),
                "union_visual_bounds_px": list(visual_union),
                "union_collision_bounds_px": list(collision_union),
            }

        existing = existing_entries.get(enemy.name)
        if existing:
            entry = merge_entries(entry, existing)

        enemies[enemy.name] = entry

    return {"version": 1, "enemies": enemies}


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate enemy manifest from alpha bounds.")
    parser.add_argument("--visual-threshold", type=int, default=16)
    parser.add_argument("--collision-threshold", type=int, default=64)
    parser.add_argument("--output", type=str, default="assets/enemies/enemy_manifest.yaml")
    parser.add_argument(
        "--destructive",
        action="store_true",
        help="Regenerate manifest without preserving manual overrides",
    )
    args = parser.parse_args()

    project_root = Path(__file__).resolve().parents[2]
    assets_root = project_root / "assets"
    output_path = project_root / args.output

    existing_entries: Dict[str, Dict] = {}
    if output_path.exists() and not args.destructive:
        try:
            existing = yaml.safe_load(output_path.read_text(encoding="utf-8")) or {}
            existing_entries = existing.get("enemies", {}) if isinstance(existing, dict) else {}
        except yaml.YAMLError as err:
            print(f"⚠ Failed to parse existing manifest (will overwrite): {err}")

    known_sources, known_paths = build_known_paths()
    known_names = set(known_sources.keys())
    auto_sources = discover_enemy_sources(assets_root, known_paths, known_names)
    all_sources = ENEMIES + auto_sources

    manifest = build_manifest(
        assets_root=assets_root,
        visual_threshold=args.visual_threshold,
        collision_threshold=args.collision_threshold,
        existing_entries=existing_entries,
        sources=all_sources,
    )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(dump_yaml(manifest), encoding="utf-8")

    print(f"✓ Wrote {len(manifest['enemies'])} entries to {output_path}")
    print(f"  visual_threshold={args.visual_threshold}, collision_threshold={args.collision_threshold}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
