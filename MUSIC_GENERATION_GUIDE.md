# Tydust Music Generation Guide

## Quick Start: Generate Darkwave Music

### Default Configuration
- **Model**: AudioCraft MusicGen (facebook/musicgen-small)
- **Duration**: 90 seconds
- **Format**: MP3 (192 kbps) + OGG (quality 6, ~120 kbps)
- **Sample Rate**: 44.1 kHz
- **Output Location**: `/home/titus/tydust/assets/music/darkwave/`

---

## Method 1: AudioCraft MusicGen (Recommended)

### IMPORTANT: Duration Requirements
⚠️ **AudioCraft MusicGen generates 30-second clips by default.** To create longer tracks:
- **60 seconds**: Generate **2 segments** and concatenate (2 × 30s)
- **90 seconds**: Generate **3 segments** and concatenate (3 × 30s)
- **120 seconds**: Generate **4 segments** and concatenate (4 × 30s)

All provided scripts handle this automatically. The duration parameter determines how many 30-second clips are concatenated.

### Setup (One-time)
```bash
source /home/titus/audiocraft_env/bin/activate
```

### Generate Music

#### Phase 1: Calm Exploration
```bash
python3 /home/titus/tydust/generate_phase1_calm_audiocraft_small.py
```
- **Prompt**: Synthwave 80s retro space exploration (or modify as needed)
- **Output**: `phase1_calm_audiocraft.mp3/ogg`

#### Phase 3: Combat
```bash
python3 /home/titus/tydust/generate_phase3_combat_audiocraft_small.py
```
- **Prompt**: Darkwave combat music, intense dark wave synthesizers
- **Output**: `phase3_combat_audiocraft.mp3/ogg`

#### Phase 4: Peak Intensity
```bash
python3 /home/titus/tydust/generate_phase4_peak_audiocraft.py
```
- **Prompt**: Darkwave peak intensity, maximum intensity driving rhythm
- **Output**: `phase4_peak_audiocraft.mp3/ogg`

#### Generate Custom Track
Create a new file `/home/titus/tydust/generate_phase_X_audiocraft.py`:

```python
#!/usr/bin/env python3
import os
import sys
from pathlib import Path

try:
    import torch
    import torchaudio
    from audiocraft.models import MusicGen
except ImportError:
    print("ERROR: audiocraft not available")
    sys.exit(1)

OUTPUT_DIR = Path("/home/titus/tydust/assets/music/darkwave")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

TRACK = {
    "name": "phase_X_audiocraft",
    "prompt": "YOUR_CUSTOM_PROMPT_HERE",
    "temperature": 1.0,
    "duration": 90,  # Change duration as needed
}

def generate_track():
    device = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"Loading MusicGen (small) on {device}...")

    model = MusicGen.get_pretrained('facebook/musicgen-small', device=device)

    print(f"Generating {TRACK['duration']}-second track...")
    model.set_generation_params(
        temperature=TRACK['temperature'],
        top_k=250,
        top_p=0.0,
        use_sampling=True,
    )

    wav = model.generate([TRACK['prompt']], progress=True)
    if wav.dim() == 3:
        wav = wav[0]

    # Resample 32kHz → 44.1kHz
    if wav.ndim == 1:
        wav = wav.unsqueeze(0)

    resampler = torchaudio.transforms.Resample(
        orig_freq=32000,
        new_freq=44100
    )
    wav = resampler(wav)

    # Save WAV
    wav_path = OUTPUT_DIR / f"{TRACK['name']}.wav"
    torchaudio.save(str(wav_path), wav, 44100)

    # Convert to OGG
    os.system(f'ffmpeg -i "{wav_path}" -c:a libvorbis -q:a 6 "{OUTPUT_DIR / f\"{TRACK[\"name\"]}.ogg\"}" -y 2>/dev/null')

    # Convert to MP3
    mp3_path = OUTPUT_DIR / f"{TRACK['name']}.mp3"
    os.system(f'ffmpeg -i "{wav_path}" -c:a libmp3lame -q:a 4 "{mp3_path}" -y 2>/dev/null')

    # Cleanup
    wav_path.unlink()

    print(f"✓ Generated: {mp3_path}")
    if mp3_path.exists():
        size_kb = mp3_path.stat().st_size / 1024
        print(f"  Size: {size_kb:.1f} KB")

if __name__ == "__main__":
    generate_track()
```

### Generation Parameters

| Parameter | Default | Range | Effect |
|-----------|---------|-------|--------|
| **temperature** | 1.0 | 0.5-1.5 | Higher = more creative/varied, Lower = more predictable |
| **top_k** | 250 | 100-500 | Sampling from top N tokens (higher = more diverse) |
| **top_p** | 0.0 | 0.0-1.0 | Nucleus sampling (0 = off, higher = more varied) |
| **duration** | 90 | 15-120 | Track length in seconds |

---

## Method 2: Code-to-Music (music21 + FluidSynth)

### Setup (One-time)
```bash
pip3 install music21 mido midi2audio pydub
```

### Generate Music

#### Phase 3: Combat (60 seconds)
```bash
python3 /home/titus/tydust/compose_darkwave_combat.py
```
- **Output**: `phase3_combat_code2music.mp3`
- **Duration**: ~60 seconds

#### Phase 4: Peak (90 seconds)
```bash
python3 /home/titus/tydust/compose_phase4_peak.py
```
- **Output**: `phase4_peak_code2music.mp3`
- **Duration**: ~90 seconds
- **BPM**: 125 (maximum intensity)

### Composition Structure

The code-to-music approach uses **algorithmic composition** with:

1. **6 instrumental tracks**:
   - Drums (MIDI channel 10, percussion)
   - Bass (Program 38 - Synth Bass 1)
   - Pad 1 (Program 88 - New Age Pad)
   - Pad 2 (Program 90 - Polysynth Pad)
   - Lead 1 (Program 80 - Square Lead)
   - Lead 2 (Program 81 - Sawtooth Lead)

2. **3-part structure** (typical for darkwave):
   - **Intro**: Building intensity, sparse elements
   - **Main**: Maximum intensity, dense orchestration
   - **Outro**: Resolution, maintained tension

3. **Rendering pipeline**:
   - music21 composition → MIDI export
   - MIDI instrument programming (mido)
   - FluidSynth WAV rendering (FluidR3_GM.sf2)
   - ffmpeg MP3 conversion (192 kbps)

### Customizing Composition

Edit the Python script to modify:
- **BPM**: `BPM = 115` (line ~15)
- **Duration segments**: `INTRO_DURATION`, `MAIN_DURATION`, `OUTRO_DURATION`
- **Key/scale**: Bass notes, lead melody arrays
- **Instruments**: MIDI program numbers (0-127 General MIDI)

---

## Method 3: Parallel Generation (Comparison)

To compare both methods simultaneously:

```bash
# Terminal 1: AudioCraft
source /home/titus/audiocraft_env/bin/activate
python3 /home/titus/tydust/generate_phase4_peak_audiocraft.py &

# Terminal 2: Code-to-Music
python3 /home/titus/tydust/compose_phase4_peak.py &

# Wait for both and compare
wait
ls -lh /home/titus/tydust/assets/music/darkwave/phase4_peak*
```

### Comparison Metrics

| Aspect | AudioCraft | Code-to-Music |
|--------|-----------|---------------|
| **Generation Speed** | 3-5 min | 2-3 min |
| **File Size (90s)** | 350-450 KB | 2+ MB |
| **Quality** | AI-generated, natural flow | Algorithmic, structured |
| **Customization** | Text prompt tuning | Code modification |
| **Consistency** | Variable (temperature-based) | Deterministic |
| **Resource Usage** | ~4-5 GB RAM, CUDA | ~1-2 GB RAM |

---

## File Organization

### Scripts
```
/home/titus/tydust/
├── generate_phase1_calm_audiocraft_small.py      # AudioCraft generation
├── generate_phase3_combat_audiocraft_small.py    # AudioCraft generation
├── generate_phase4_peak_audiocraft.py            # AudioCraft generation (90s)
├── compose_darkwave_combat.py                    # Code-to-music (60s)
└── compose_phase4_peak.py                        # Code-to-music (90s)
```

### Generated Audio
```
/home/titus/tydust/assets/music/darkwave/
├── phase1_calm.mp3                       # Original
├── phase1_calm_audiocraft.mp3/ogg        # AudioCraft version
├── phase3_combat.mp3                     # Original
├── phase3_combat_audiocraft.mp3/ogg      # AudioCraft version
├── phase3_combat_code2music.mp3          # Code-to-music version
├── phase4_peak.mp3                       # Original (if exists)
├── phase4_peak_audiocraft.mp3/ogg        # AudioCraft version
└── phase4_peak_code2music.mp3            # Code-to-music version
```

---

## Best Practices

### AudioCraft Tips
1. **Prompt Engineering**: More detailed prompts = better results
   - ✓ Good: "Darkwave peak intensity, 120 BPM, eerie synths, pulsing bass, urgent atmosphere"
   - ✗ Vague: "darkwave music"

2. **Temperature Settings**:
   - 0.8-1.0: Balanced creativity/consistency
   - 1.1-1.5: Maximum variation (for comparisons)
   - 0.5-0.7: Conservative, predictable

3. **Duration**: Keep to 30-120 seconds for optimal results

### Code-to-Music Tips
1. **MIDI Programs**: Use General MIDI (0-127) for compatibility
2. **BPM Range**: 100-130 for darkwave intensity
3. **Note Density**: Increase during "peak" sections, sparse during "intro"
4. **Testing**: Generate short 30-second tracks first, then expand

---

## Troubleshooting

### AudioCraft Issues

**"audiocraft not found"**
```bash
source /home/titus/audiocraft_env/bin/activate
python3 -c "import audiocraft; print('OK')"
```

**"CUDA out of memory"**
- Switch to CPU: Remove `device = "cuda"` line
- Reduce duration
- Use smaller model (already using musicgen-small)

**"Generation too slow"**
- Check GPU: `nvidia-smi`
- Try CPU if GPU busy
- Generate shorter duration first

### Code-to-Music Issues

**"FluidSynth not found"**
```bash
sudo apt update && sudo apt install fluidsynth libfluidsynth-dev
```

**"MIDI rendering failed"**
- Verify soundfont: `ls /usr/share/sounds/sf2/FluidR3_GM.sf2`
- Check MIDI validity: `timidity phase_X_temp.mid`

**"MP3 conversion failed"**
```bash
sudo apt install ffmpeg
```

---

## Automation: Batch Generation

Generate all phases with comparison:

```bash
#!/bin/bash
PHASES=("1_calm" "3_combat" "4_peak")

for phase in "${PHASES[@]}"; do
    echo "Generating phase $phase..."

    # AudioCraft
    source /home/titus/audiocraft_env/bin/activate
    python3 /home/titus/tydust/generate_phase${phase}_audiocraft_small.py &

    # Code-to-music
    python3 /home/titus/tydust/compose_phase${phase}.py &

    wait
    echo "✓ Phase $phase complete"
done

echo "✓ All phases generated"
ls -lh /home/titus/tydust/assets/music/darkwave/
```

---

## Reference: Previous Successful Generations

### AudioCraft Phase 3 Combat (60s)
- **Model**: musicgen-small
- **File Size**: 299 KB (MP3), 337 KB (OGG)
- **Quality**: Professional, natural flow
- **Generation Time**: ~3-4 minutes

### Code-to-Music Phase 3 Combat (60s)
- **File Size**: 1.4 MB (MP3)
- **Quality**: Algorithmic, structured
- **Generation Time**: ~2-3 minutes

### Code-to-Music Phase 4 Peak (90s)
- **File Size**: 2.2 MB (MP3)
- **Duration**: 94.6 seconds (with overhead)
- **BPM**: 125
- **Quality**: Maximum intensity, 6 synth tracks

---

## Default Workflow (Recommended)

```bash
# 1. Setup (one-time)
source /home/titus/audiocraft_env/bin/activate

# 2. Generate single track (90 seconds, AudioCraft)
python3 /home/titus/tydust/generate_phase4_peak_audiocraft.py

# 3. Verify
ls -lh /home/titus/tydust/assets/music/darkwave/phase4_peak_audiocraft.*

# 4. Optional: Compare with code-to-music
python3 /home/titus/tydust/compose_phase4_peak.py
```

---

## Next Steps

1. **Batch generation**: Run all phases with both methods
2. **Quality comparison**: A/B test AudioCraft vs code-to-music
3. **Integration**: Update game music loader to use generated tracks
4. **Randomization**: Randomly select between versions at game startup
5. **Custom phases**: Generate other game phases (phase 2, phase 5)

---

*Last Updated: 2026-01-06*
*Music Generation Guide for Tydust Darkwave Synthesis*
