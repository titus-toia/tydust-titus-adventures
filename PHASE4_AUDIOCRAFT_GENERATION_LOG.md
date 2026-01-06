# Phase4 Peak AudioCraft Generation Log

## Generation Request
- **Timestamp**: 2026-01-06 13:54:00
- **Duration**: 90 seconds
- **Model**: facebook/musicgen-small
- **Format**: MP3 (192kbps) + OGG (quality 6, ~120kbps)
- **Sample Rate Target**: 44.1kHz
- **Genre**: Darkwave peak intensity

## Generation Prompt
```
Darkwave peak intensity music, maximum intensity driving rhythm. Intense dark wave
synthesizers with pulsing bass at 120+ BPM. Eerie synth lines at maximum drama,
urgent and overwhelming atmosphere. Deep bass pulses, layered dark synths, distorted
leads, chaotic and intense. Peak moment of darkwave music.
```

## Script Details
- **Script**: `/home/titus/tydust/generate_phase4_peak_audiocraft.py`
- **Output Dir**: `/home/titus/tydust/assets/music/darkwave/`
- **Output Files**:
  - `phase4_peak_audiocraft.mp3` (192kbps)
  - `phase4_peak_audiocraft.ogg` (quality 6, ~120kbps)

## Generation Process
1. Load facebook/musicgen-small model with CUDA if available
2. Generate 90-second audio at 32kHz sample rate
3. Resample to 44.1kHz using torchaudio
4. Convert to MP3 (192kbps quality 4)
5. Convert to OGG (Vorbis quality 6)
6. Clean up intermediate WAV files

## Execution Command
```bash
/home/titus/audiocraft_env/bin/python /home/titus/tydust/generate_phase4_peak_audiocraft.py
```

## Status
Generation in progress - started at 13:54 UTC
- Process ID: 9190
- Memory usage: ~3.5GB
- CPU time: 6+ minutes
- Expected completion: 14:00-14:05 UTC (generation + conversion)

## Expected Files After Completion
- `/home/titus/tydust/assets/music/darkwave/phase4_peak_audiocraft.mp3` (~400-500 KB)
- `/home/titus/tydust/assets/music/darkwave/phase4_peak_audiocraft.ogg` (~300-400 KB)
