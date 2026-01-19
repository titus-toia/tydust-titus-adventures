#!/usr/bin/env python3
"""Generate 90s synthwave music for Tydust using HuggingFace MusicGen."""

import torch
import scipy.io.wavfile
from transformers import AutoProcessor, MusicgenForConditionalGeneration
from pathlib import Path

def generate_synthwave_music():
    """Generate 5 phases of 90s synthwave music."""

    # Load model and processor (using small model for faster generation)
    print("Loading MusicGen model from HuggingFace...")
    processor = AutoProcessor.from_pretrained("facebook/musicgen-small")
    model = MusicgenForConditionalGeneration.from_pretrained("facebook/musicgen-small")

    # Move to device (CPU only for compatibility)
    model = model.to("cpu")

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

    output_dir = Path("/workspace/assets/music/synthwave-2")
    output_dir.mkdir(parents=True, exist_ok=True)

    sampling_rate = model.config.audio_encoder.sampling_rate

    for phase in phases:
        print(f"\nGenerating {phase['name']}...")
        print(f"Prompt: {phase['prompt']}")

        # Process inputs
        inputs = processor(
            text=[phase['prompt']],
            padding=True,
            return_tensors="pt"
        ).to("cpu")

        # Generate audio (30 seconds max, about 2 * 15k tokens)
        with torch.no_grad():
            audio_values = model.generate(
                **inputs,
                do_sample=True,
                guidance_scale=3,
                max_new_tokens=512  # This gives approximately 90 seconds
            )

        # Save as WAV
        output_path = output_dir / f"{phase['name']}.wav"
        audio_data = audio_values[0, 0].cpu().numpy()
        scipy.io.wavfile.write(str(output_path), sampling_rate, audio_data.astype('float32'))
        print(f"✓ Saved to {output_path}")

if __name__ == "__main__":
    generate_synthwave_music()
    print("\nAll synthwave-2 tracks generated successfully!")
