#!/usr/bin/env python3
"""
Generate procedural asteroid field for level 3.

Usage:
    python scripts/generate_asteroid_field.py [--count 75] [--start 1000] [--end 10000]
"""

import random
import argparse
from dataclasses import dataclass
from typing import List, Tuple

@dataclass
class Asteroid:
    enemy_type: str
    position: Tuple[float, float]
    velocity: Tuple[float, float]
    behavior: str  # "MoveStraight" or "Accelerate"
    acceleration: Tuple[float, float] = (0.0, 0.0)
    spawn_distance: float = 0.0

def generate_asteroid_field(
    count: int = 75,
    start_gu: float = 1000.0,
    end_gu: float = 10000.0,
    seed: int = None
) -> List[Asteroid]:
    """Generate a procedural asteroid field."""

    if seed is not None:
        random.seed(seed)

    asteroids = []

    # Asteroid type weights (more large asteroids for better gameplay)
    types = [
        ("SmallAsteroid", 0.15),
        ("MediumAsteroid", 0.35),
        ("LargeAsteroid", 0.50),
    ]

    # Viewport constraints (4:3 aspect ratio)
    x_min, x_max = -600.0, 600.0
    y_min, y_max = 500.0, 750.0  # Spawn above viewport

    # Base Y velocity (matches scroll speed)
    base_y_velocity = -100.0

    # Behavior distribution: 75% MoveStraight, 25% Accelerate
    accelerate_chance = 0.25

    # Generate spawn distances spread across the zone
    zone_length = end_gu - start_gu

    for i in range(count):
        # Spread asteroids across the zone with some clustering
        # Use a mix of uniform and clustered distribution
        if random.random() < 0.3:
            # Cluster around existing asteroids
            base_dist = random.uniform(start_gu, end_gu)
            spawn_dist = base_dist + random.gauss(0, 200)
        else:
            # Uniform distribution
            spawn_dist = start_gu + (i / count) * zone_length + random.uniform(-100, 100)

        spawn_dist = max(start_gu, min(end_gu, spawn_dist))

        # Choose asteroid type
        r = random.random()
        cumulative = 0.0
        asteroid_type = "MediumAsteroid"
        for atype, weight in types:
            cumulative += weight
            if r < cumulative:
                asteroid_type = atype
                break

        # Position
        x = random.uniform(x_min, x_max)
        y = random.uniform(y_min, y_max)

        # Velocity - X drift varies, Y is constant
        x_velocity = random.uniform(-50.0, 50.0)
        y_velocity = base_y_velocity

        # Behavior
        if random.random() < accelerate_chance:
            behavior = "Accelerate"
            # Small horizontal acceleration
            x_accel = random.uniform(-8.0, 8.0)
            acceleration = (x_accel, 0.0)
        else:
            behavior = "MoveStraight"
            acceleration = (0.0, 0.0)

        asteroids.append(Asteroid(
            enemy_type=asteroid_type,
            position=(round(x, 1), round(y, 1)),
            velocity=(x_velocity, y_velocity),
            behavior=behavior,
            acceleration=acceleration,
            spawn_distance=round(spawn_dist, 1)
        ))

    # Sort by spawn distance
    asteroids.sort(key=lambda a: a.spawn_distance)

    return asteroids

def group_into_waves(asteroids: List[Asteroid], max_gap: float = 150.0) -> List[List[Asteroid]]:
    """Group nearby asteroids into waves."""
    if not asteroids:
        return []

    waves = []
    current_wave = [asteroids[0]]

    for asteroid in asteroids[1:]:
        if asteroid.spawn_distance - current_wave[-1].spawn_distance < max_gap:
            current_wave.append(asteroid)
        else:
            waves.append(current_wave)
            current_wave = [asteroid]

    if current_wave:
        waves.append(current_wave)

    return waves

def format_yaml(waves: List[List[Asteroid]], count: int, start_gu: float, end_gu: float) -> str:
    """Format waves as YAML."""
    lines = [
        "# Procedurally generated asteroid field for level 3",
        f"# Zone: {int(start_gu)}-{int(end_gu)} GU (~{int((end_gu-start_gu)/1000)} screens, {count} asteroids)",
        "# Asteroids spawn ABOVE viewport (Y: 550-700) and scroll down into view",
        "# Horizontal spread constrained to 4:3 viewport (X: +/-650)",
        "# Patterns: Dense clusters, spiral streams, lanes, pairs, random fill",
        "# Behaviors: MoveStraight (75%), Accelerate (25%)",
        "# All asteroids have Y velocity = -100 (matches level 3 scroll speed)",
        "# X velocity varies per asteroid (sideways drift and oscillation)",
        "# Perfect for testing chain lightning mechanics across extended gameplay!",
        "",
        "enemy_waves:",
    ]

    for wave in waves:
        spawn_dist = wave[0].spawn_distance
        lines.append(f"# Wave at {spawn_dist} GU ({len(wave)} asteroids)")
        lines.append("- enemies:")

        for asteroid in wave:
            lines.append(f"  - enemy_type: {asteroid.enemy_type}")
            lines.append(f"    position: [{asteroid.position[0]}, {asteroid.position[1]}]")
            lines.append("    behaviors:")

            if asteroid.behavior == "MoveStraight":
                lines.append("      - type: MoveStraight")
                lines.append(f"        velocity: [{asteroid.velocity[0]}, {asteroid.velocity[1]}]")
            else:
                lines.append("      - type: Accelerate")
                lines.append(f"        initial_velocity: [{asteroid.velocity[0]}, {asteroid.velocity[1]}]")
                lines.append(f"        acceleration: [{asteroid.acceleration[0]}, {asteroid.acceleration[1]}]")

            lines.append("        duration: 20.0")
            lines.append("        transition: 'WaitForCompletion'")

        lines.append(f"  spawn_distance: {spawn_dist}")
        lines.append("")

    return "\n".join(lines)

def main():
    parser = argparse.ArgumentParser(description="Generate asteroid field for level 3")
    parser.add_argument("--count", type=int, default=75, help="Number of asteroids")
    parser.add_argument("--start", type=float, default=1000.0, help="Start distance (GU)")
    parser.add_argument("--end", type=float, default=10000.0, help="End distance (GU)")
    parser.add_argument("--seed", type=int, default=None, help="Random seed for reproducibility")
    parser.add_argument("--output", type=str, default=None, help="Output file path")
    args = parser.parse_args()

    asteroids = generate_asteroid_field(
        count=args.count,
        start_gu=args.start,
        end_gu=args.end,
        seed=args.seed
    )

    waves = group_into_waves(asteroids)
    yaml_content = format_yaml(waves, args.count, args.start, args.end)

    if args.output:
        with open(args.output, 'w') as f:
            f.write(yaml_content)
        print(f"Generated {args.count} asteroids in {len(waves)} waves -> {args.output}")
    else:
        print(yaml_content)

if __name__ == "__main__":
    main()
