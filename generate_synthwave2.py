#!/usr/bin/env python3
"""Generate 90s synthwave music for Tydust using AudioCraft."""

import torch
import torchaudio
from audiocraft.models import MusicGen
from pathlib import Path

def generate_synthwave_music():
    """Generate 5 phases of 90s synthwave music."""

    # Initialize model
    print("Loading MusicGen model...")
    model = MusicGen.get_pretrained('facebook/musicgen-medium')
    model.set_generation_params(
        duration=90,  # 90 seconds per track
        top_k=250,
        temperature=1.0,
        cfg_coef=3.0
    )

    # Define music phases with 90s synthwave characteristics
    phases = [
        {
            "name": "phase1_calm",
            "prompt": "90s synthwave, calm lo-fi atmospheric synth pad, retro neon vibes, relaxed tempo, dreamy"
        },
        {
            "name": "phase2_tension",
            "prompt": "90s synthwave, building tension, dark pulsing bassline, electronic drums, tense atmosphere"
        },
        {
            "name": "phase3_combat",
            "prompt": "90s synthwave, intense combat music, driving beat, aggressive synth leads, high energy action"
        },
        {
            "name": "phase4_peak",
            "prompt": "90s synthwave, epic climactic peak, soaring synth melodies, explosive drums, maximum intensity"
        },
        {
            "name": "phase5_boss",
            "prompt": "90s synthwave, boss battle theme, heavy distorted synth, thunderous drums, menacing dark vibe"
        }
    ]

    output_dir = Path("/Users/titustc/projects/tydust-titus-adventures/assets/music/synthwave-2")
    output_dir.mkdir(parents=True, exist_ok=True)

    for phase in phases:
        print(f"\nGenerating {phase['name']}...")
        print(f"Prompt: {phase['prompt']}")

        # Generate audio
        with torch.no_grad():
            wav = model.generate([phase['prompt']])

        # Save as WAV
        output_path = output_dir / f"{phase['name']}.wav"
        torchaudio.save(str(output_path), wav[0].cpu(), sample_rate=32000)
        print(f"✓ Saved to {output_path}")

if __name__ == "__main__":
    generate_synthwave_music()
    print("\nAll synthwave-2 tracks generated successfully!")
