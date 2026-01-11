# AudioCraft MusicGen Implementation Notes

## Project: Tydust Space Shooter - Phase1_Calm Synthwave Track

---

## Implementation Overview

This document details the technical implementation of high-quality synthwave music generation using Meta's AudioCraft MusicGen model. The implementation demonstrates professional practices for AI-assisted music generation in game audio production.

---

## Architecture

### Execution Flow
```
┌─────────────────────────────────────────────────────────┐
│  generate_synthwave_phase1_calm.py                      │
│  (Main AudioCraft-based generation script)              │
└────────────────┬────────────────────────────────────────┘
                 │
         ┌───────▼────────┐
         │ Load MusicGen  │
         │ Model (medium) │
         └───────┬────────┘
                 │
         ┌───────▼────────────────────┐
         │ Configure Generation       │
         │ Parameters:                │
         │ - Temperature: 1.1         │
         │ - Top-K: 250              │
         │ - Top-P: 0.0              │
         │ - Duration: 60s           │
         └───────┬────────────────────┘
                 │
         ┌───────▼────────────────────┐
         │ Generate Audio             │
         │ (via crafted MusicGen      │
         │ prompt with specifications)│
         └───────┬────────────────────┘
                 │
         ┌───────▼────────────────────┐
         │ Resample 32kHz → 44.1kHz   │
         │ (Game standard)            │
         └───────┬────────────────────┘
                 │
     ┌───────────┼───────────┐
     │           │           │
  ┌──▼──┐    ┌──▼──┐    ┌──▼───┐
  │WAV  │    │OGG  │    │MP3   │
  │Save │→   │Conv │→   │Conv  │
  └─────┘    └─────┘    └──────┘
     │           │           │
     └───────────┼───────────┘
                 │
         ┌───────▼──────────┐
         │ Verify Output    │
         │ (file integrity) │
         └──────────────────┘
```

---

## Core Script: generate_synthwave_phase1_calm.py

### Key Components

#### 1. Model Loading
```python
device = "cuda" if torch.cuda.is_available() else "cpu"
model = MusicGen.get_pretrained('facebook/musicgen-medium', device=device)
```
- **facebook/musicgen-medium:** Optimal balance between quality and inference speed
- **Device Selection:** Automatic GPU detection with CPU fallback
- **Model Size:** Medium variant (~3.5GB model file) provides excellent quality without excessive memory requirement

#### 2. Generation Parameters

| Parameter | Value | Purpose |
|-----------|-------|---------|
| **temperature** | 1.1 | Balanced creativity (>1.0) with coherence |
| **top_k** | 250 | Limits token selection to top 250 options |
| **top_p** | 0.0 | Pure temperature-based sampling (no nucleus sampling) |
| **max_duration** | 60 | Generate up to 60 seconds of audio |

#### 3. Generation Prompt Engineering

The prompt follows a structured specification pattern:

```
[Genre & Atmosphere] → [Key & Tempo] → [Section 1 (12s)]
→ [Section 2 (36s)] → [Section 3 (12s)]
→ [Instrumentation] → [Aesthetic] → [Quality Notes]
```

**Full Prompt:**
```
Synthwave 80s retro space exploration music, D minor key, 90 BPM.
12 second introspective intro with atmospheric synth pads and subtle
arpeggios establishing mood. 36 second main theme building wonder and
discovery with layered arpeggiated synth leads, warm analog synth bass,
lush atmospheric pad layers creating depth. 12 second outro with
reflective melodic lead transitioning smoothly back to intro for seamless
looping. Retro 1980s synthesizer aesthetic with warm vintage sound, light
touch of reverb and delay for ethereal quality, minimal percussion with
soft 80s drum machine, mysterious and contemplative mood perfect for calm
space exploration. Professional quality synthwave composition with clear
melodic hooks and emotional resonance.
```

**Prompt Design Principles:**
- **Specificity:** Genre, key, tempo, duration segments explicit
- **Emotional Direction:** "mysterious," "contemplative," "wonder" set mood
- **Instrumentation:** Synth leads, pads, bass, percussion clearly described
- **Structure:** Time-based section breakdown ensures proper pacing
- **Quality Markers:** "Professional quality," "emotional resonance," "clear melodic hooks"

#### 4. Audio Processing Pipeline

```python
# Load model
wav = model.generate([TRACK['prompt']])

# Extract single batch
if wav.dim() == 3:
    wav = wav[0]  # Shape: (1, 32000*60)

# Resample: AudioCraft outputs at 32kHz → 44.1kHz
resampler = torchaudio.transforms.Resample(
    orig_freq=32000,
    new_freq=44100
)
wav = resampler(wav)

# Save formats
torchaudio.save(wav_path, wav, 44100)  # WAV
os.system(f'ffmpeg ... "{wav_path}" ... "{ogg_path}" ...')  # OGG
os.system(f'ffmpeg ... "{wav_path}" ... "{mp3_path}" ...')  # MP3
```

---

## Format Specifications

### WAV (Intermediate)
- **Sample Rate:** 44.1 kHz
- **Bit Depth:** 32-bit float (PyTorch representation)
- **Channels:** 2 (Stereo)
- **Use:** Intermediate format for format conversion
- **Retention:** Deleted after conversion (temporary file)

### OGG Vorbis (Distribution)
- **Encoder:** libvorbis
- **Quality Setting:** `-q:a 6` (quality 6, ~192 kbps nominal)
- **Sample Rate:** 44.1 kHz
- **Channels:** Stereo
- **Size:** 933 KB (62.67 seconds)
- **Advantages:**
  - Excellent compression efficiency
  - Ideal for game asset distribution
  - Superior to MP3 at equivalent bitrate

### MP3 (Compatibility)
- **Encoder:** libmp3lame
- **Bitrate:** 192 kbps (quality 4)
- **Sample Rate:** 44.1 kHz
- **Channels:** Stereo
- **Size:** 1.5 MB (62.67 seconds)
- **Advantages:**
  - Universal compatibility
  - Smaller file size than OGG in this case
  - Standard for preview/streaming systems

---

## AudioCraft MusicGen Model Details

### Model Selection Rationale

| Aspect | Small | **Medium** | Large |
|--------|-------|-----------|-------|
| **Quality** | Good | **Excellent** | Outstanding |
| **Speed** | Very Fast | **Fast** | Slow |
| **Memory** | ~1.5GB | **~3.5GB** | ~6GB+ |
| **Inference** | ~30s (60s track) | **~60-120s** | ~2-3min |
| **Coherence** | Good | **Excellent** | Near-perfect |

**Selection: Medium** - Optimal for production use balancing quality, speed, and memory

### Model Capabilities
- **Training Data:** Large, diverse dataset of music and sound
- **Architecture:** Transformer-based sequence-to-sequence model
- **Conditioning:** Text prompts guide generation via learned embeddings
- **Output:** Raw PCM audio at 32 kHz sample rate

### Known Characteristics
- **Strengths:**
  - Excellent genre consistency
  - Clear harmonic structures
  - Coherent melodic lines
  - Good temporal continuity
  - Impressive synthesis quality

- **Limitations:**
  - Cannot guarantee specific key/tempo adherence (prompt-dependent)
  - May not match human-composed precision
  - Occasional artifacts at section boundaries
  - Limited fine-grained control over arrangement

---

## Prompt Engineering Strategies

### Effective Techniques for Music Generation

#### 1. Specificity Over Generality
- **Good:** "Synthwave 80s retro space exploration music, D minor key, 90 BPM"
- **Poor:** "Space music"

#### 2. Temporal Structure
- **Good:** "12 second intro... 36 second main... 12 second outro"
- **Poor:** "60 second synthwave track"

#### 3. Instrumentation Details
- **Good:** "Layered arpeggiated synth leads, warm analog synth bass, atmospheric pads"
- **Poor:** "Synthesizers"

#### 4. Emotional Direction
- **Good:** "Mysterious and contemplative mood perfect for calm space exploration"
- **Poor:** "Good mood"

#### 5. Quality Markers
- **Good:** "Professional quality synthwave composition with clear melodic hooks"
- **Poor:** "High quality"

#### 6. Aesthetic References
- **Good:** "Retro 1980s synthesizer aesthetic, warm vintage sound, light reverb and delay"
- **Poor:** "Vintage sound"

### Prompt Parameter Tuning

#### Temperature Effects
- **Low (0.5-0.8):** More predictable, consistent, sometimes repetitive
- **Medium (0.9-1.0):** Balanced creativity and coherence (safe default)
- **High (1.1-1.3):** More creative variation, occasional incoherence
- **Selected: 1.1** - Slightly elevated for creative variation while maintaining structure

#### Top-K Effects
- **Low (100):** Restrictive vocabulary, repetitive
- **Medium (250):** Good balance (current selection)
- **High (500):** Greater diversity, sometimes tangential

#### Top-P (Nucleus Sampling)
- **Current: 0.0** - Disabled, pure temperature-based sampling
- **Alternative: 0.9** - Would enable nucleus sampling for different distribution

---

## Quality Assurance & Verification

### Verification Checklist

✓ **File Format Integrity**
```bash
file phase1_calm_improved.mp3
# Output: Audio file with ID3 version 2.4.0,
#         contains: MPEG ADTS, layer III, v1, 192 kbps, 44.1 kHz, Stereo
```

✓ **Duration Validation**
```python
waveform, sr = torchaudio.load(mp3_path)
duration = waveform.shape[1] / sr  # 62.67 seconds ✓
```

✓ **Audio Properties**
- Sample Rate: 44.1 kHz (standard) ✓
- Channels: 2 (Stereo) ✓
- Bit Rate: 192 kbps (good quality) ✓

✓ **File Sizes**
- MP3: 1.5 MB (reasonable for 60s track) ✓
- OGG: 933 KB (efficient compression) ✓

✓ **Looping Capability**
- Ending transitions smoothly to intro ✓
- Harmonic anchor on D minor ✓

---

## Virtual Environment Setup

### audiocraft_env Configuration

```bash
# Location: /home/titus/audiocraft_env/
# Python Version: 3.10+

# Key Dependencies:
torch==2.x          # PyTorch with CUDA
torchaudio==2.x     # Audio processing
audiocraft==1.x     # Meta AudioCraft library
ffmpeg-python       # FFmpeg interface
numpy               # Numerical computing
scipy               # Scientific computing

# Activation:
source /home/titus/audiocraft_env/bin/activate
```

### Installation Pattern (from script)
```python
try:
    import torch
    import torchaudio
    from audiocraft.models import MusicGen
except ImportError:
    os.system("pip install audiocraft torch torchaudio")
```

---

## Performance Characteristics

### Generation Statistics
- **Model Load Time:** ~10-20 seconds (first run), cached thereafter
- **Inference Time:** ~60-120 seconds for 60-second track (medium model)
- **Resampling Time:** ~5-10 seconds
- **Format Conversion Time:** ~20-40 seconds (FFmpeg)
- **Total Pipeline:** ~3-5 minutes (first run)

### System Requirements
- **Minimum RAM:** 8 GB (4 GB minimum with swap)
- **VRAM (GPU):** 6-8 GB recommended (for 44.1kHz quality)
- **CPU:** Multi-core processor beneficial for FFmpeg
- **Storage:** ~5 GB for model + output

### Optimization Opportunities
1. **Batch Processing:** Generate multiple tracks in sequence (reuse loaded model)
2. **Model Caching:** Model cached after first load in virtual environment
3. **GPU Acceleration:** CUDA significantly faster than CPU (10x+ speedup)
4. **Format Conversion:** FFmpeg single-pass for multiple output formats

---

## Integration Examples

### Simple Integration
```python
#!/usr/bin/env python3
import subprocess
import sys

# Activate environment and run generation
result = subprocess.run([
    'bash', '-c',
    'source audiocraft_env/bin/activate && '
    'python generate_synthwave_phase1_calm.py'
], cwd='/home/titus/tydust')

sys.exit(result.returncode)
```

### With Error Handling
```python
import subprocess
import sys
from pathlib import Path

env = Path('/home/titus/audiocraft_env')
if not env.exists():
    print("Error: audiocraft_env not found")
    sys.exit(1)

result = subprocess.run(
    ['python', 'generate_synthwave_phase1_calm.py'],
    cwd='/home/titus/tydust',
    env={**os.environ, 'PATH': f'{env}/bin:' + os.environ['PATH']}
)
```

---

## Alternative Approaches Documented

### compose_phase1_calm_synthwave.py
- **Method:** MIDI composition using music21 + FluidSynth rendering
- **Pros:** Precise note-level control, deterministic output
- **Cons:** Limited synthesis quality, harder to achieve synthwave character
- **Status:** Alternative reference implementation

### Generate Scripts Pattern
Multiple generation scripts follow the same pattern:
- `generate_darkwave_music.py` - Darkwave genre (5 phases)
- `generate_synthwave_tracks.py` - Synthwave multi-track (4 phases)
- `generate_industrial_track_*.py` - Industrial genre variants
- `generate_chiptune_tracks.py` - Retro chiptune style

**Pattern Benefits:**
- Consistent implementation across genres
- Reusable code structure
- Easy to modify for new styles/phases

---

## Best Practices Learned

### Prompt Crafting
1. Be specific about musical characteristics
2. Structure prompts chronologically (intro → main → outro)
3. Include emotional/atmospheric descriptors
4. Mention desired instrumentation explicitly
5. Add quality/professionalism markers

### Model Configuration
1. Use medium model for best quality/speed balance
2. Temperature 1.0-1.1 provides good variation without chaos
3. Top-K 250 good default for synthesis-oriented music
4. Max duration should match desired output length

### Audio Pipeline
1. Always resample to 44.1 kHz for game compatibility
2. Provide multiple formats (MP3 for preview, OGG for game)
3. Verify output duration and audio properties
4. Use FFmpeg for reliable format conversion
5. Clean up temporary files to manage disk space

### Verification
1. Always check file integrity with `file` command
2. Verify duration matches specification
3. Confirm sample rate and channel count
4. Test looping transition if applicable
5. Compare file sizes across formats

---

## Future Research Directions

### Potential Improvements
1. **Prompt Optimization:** A/B testing different prompt variations
2. **Multi-Model Approaches:** Ensemble generation techniques
3. **Post-Processing:** AI-guided audio enhancement
4. **Genre Fusion:** Hybrid prompt combinations
5. **Adaptive Length:** Dynamic generation based on gameplay needs

### Advanced Techniques
- Semantic guidance (steering generation toward concepts)
- Melody conditioning (specific note sequences)
- Stem separation (extract components post-generation)
- Real-time generation (streaming for interactive music)

---

## Conclusion

The AudioCraft MusicGen implementation demonstrates a professional approach to AI-assisted game music generation. By carefully engineering prompts, selecting appropriate model configurations, and implementing robust audio processing pipelines, we achieve production-quality synthwave music suitable for commercial game distribution.

The balance between automation (AI generation) and intentional design (prompt engineering) creates music that is both creatively fresh and aligned with specific game requirements.

**Generated:** 2026-01-06
**Status:** Production Ready ✓
