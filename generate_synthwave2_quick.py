#!/usr/bin/env python3
"""Quick test - Generate 30 second synthwave clips for Tydust using HuggingFace MusicGen."""

import torch
import scipy.io.wavfile
from transformers import AutoProcessor, MusicgenForConditionalGeneration
from pathlib import Path

def generate_synthwave_music():
    """Generate 5 phases of synthwave music (30 seconds each for faster testing)."""

    # Load model and processor (using small model for faster generation)
    print("Loading MusicGen model from HuggingFace...")
    processor = AutoProcessor.from_pretrained("facebook/musicgen-small")
    model = MusicgenForConditionalGeneration.from_pretrained("facebook/musicgen-small")
    model = model.to("cpu")

    # Define music phases with 90s synthwave characteristics
    phases = [
        {"name": "phase1_calm", "prompt": "90s synthwave, calm atmospheric synth pad"},
        {"name": "phase2_tension", "prompt": "90s synthwave, building tension, dark pulsing"},
        {"name": "phase3_combat", "prompt": "90s synthwave, intense combat music, aggressive"},
        {"name": "phase4_peak", "prompt": "90s synthwave, epic climactic peak"},
        {"name": "phase5_boss", "prompt": "90s synthwave, boss battle, heavy distorted synth"}
    ]

    output_dir = Path("/workspace/assets/music/synthwave-2")
    output_dir.mkdir(parents=True, exist_ok=True)
    sampling_rate = model.config.audio_encoder.sampling_rate

    for i, phase in enumerate(phases, 1):
        print(f"\n[{i}/5] Generating {phase['name']}...")
        print(f"      Prompt: {phase['prompt']}")

        inputs = processor(
            text=[phase['prompt']],
            padding=True,
            return_tensors="pt"
        ).to("cpu")

        # Generate audio (30 seconds - faster for testing)
        print("      Generating audio...")
        with torch.no_grad():
            audio_values = model.generate(
                **inputs,
                do_sample=True,
                guidance_scale=3,
                max_new_tokens=256  # ~30 seconds
            )

        # Save as WAV
        output_path = output_dir / f"{phase['name']}.wav"
        audio_data = audio_values[0, 0].cpu().numpy()
        scipy.io.wavfile.write(str(output_path), sampling_rate, audio_data.astype('float32'))
        file_size = output_path.stat().st_size / (1024 * 1024)  # Size in MB
        print(f"      ✓ Saved ({file_size:.1f}MB)")

if __name__ == "__main__":
    generate_synthwave_music()
    print("\n✓ All synthwave-2 test tracks generated!")
