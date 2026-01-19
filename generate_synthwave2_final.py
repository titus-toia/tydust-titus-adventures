#!/usr/bin/env python3
"""Generate 90s synthwave music for Tydust using MusicGen with GPU acceleration on macOS."""

import torch
import torchaudio
from transformers import AutoProcessor, MusicgenForConditionalGeneration
from pathlib import Path

def generate_synthwave_music():
    """Generate 5 phases of 90s synthwave music (90 seconds each)."""

    # Detect device
    if torch.backends.mps.is_available():
        device = "mps"
        print("Using Metal Performance Shaders (GPU) for acceleration")
    elif torch.cuda.is_available():
        device = "cuda"
        print("Using CUDA GPU for acceleration")
    else:
        device = "cpu"
        print("Using CPU (this will be slow)")

    # Load model and processor (using small model for balance of speed and quality)
    print(f"Loading MusicGen model from HuggingFace...")
    processor = AutoProcessor.from_pretrained("facebook/musicgen-small")
    model = MusicgenForConditionalGeneration.from_pretrained("facebook/musicgen-small")
    model = model.to(device)

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
    sampling_rate = model.config.audio_encoder.sampling_rate

    for i, phase in enumerate(phases, 1):
        print(f"\n[{i}/5] Generating {phase['name']}...")
        print(f"      Prompt: {phase['prompt']}")

        inputs = processor(
            text=[phase['prompt']],
            padding=True,
            return_tensors="pt"
        ).to(device)

        # Generate audio (4800 tokens ≈ 90 seconds)
        print("      Generating audio (90 seconds)...")
        with torch.no_grad():
            audio_values = model.generate(
                **inputs,
                do_sample=True,
                guidance_scale=3,
                max_new_tokens=4800  # Approximately 90 seconds
            )

        # Save as WAV
        output_path = output_dir / f"{phase['name']}.wav"
        audio_tensor = audio_values[0, 0].cpu()
        torchaudio.save(str(output_path), audio_tensor.unsqueeze(0), sampling_rate)

        file_size = output_path.stat().st_size / (1024 * 1024)
        print(f"      ✓ Saved ({file_size:.1f}MB)")

if __name__ == "__main__":
    generate_synthwave_music()
    print("\n✓ All synthwave-2 tracks generated successfully!")
