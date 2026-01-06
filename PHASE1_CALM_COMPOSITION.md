# Phase 1 Calm Track - Rework Summary

## Track Information
- **File**: `orchestral_rock_phase1_calm_improved.mp3`
- **Location**: `/home/titus/tydust/assets/music/orchestral-rock/`
- **Duration**: 62.8 seconds (target: 60 seconds)
- **Tempo**: 90 BPM
- **Key**: D minor (mysterious/sci-fi atmosphere)
- **File Size**: 949 KB

## Composition Structure

### Intro (0-12 seconds = 18 quarter notes)
- **Pad Strings**: Sustained D minor pad notes (D3, A3, D3, F3, D3, A3) with slow velocity crescendo
- **Pad Synth**: Ethereal sustained chords - D minor triad (D2, A2, D3, F3) transitioning to D2, A2, D3, Bb3
- **French Horn**: Subtle background sustain (D4, F4) - very quiet, depth only

**Goal**: Establish mysterious atmosphere with long, sustained tones. Minimal movement, drawing listener into contemplation.

### Main Theme (12-48 seconds = 54 quarter notes)
- **Violin Melody**: 48 notes total forming 6 phrases of 8 notes each
  - Phrase 1: D minor ascending (D4-E4-F4-G4-A4-G4-F4-E4) - gentle exploration
  - Phrase 2: Higher Bb color (D4-F4-A4-Bb4-A4-F4-D4-E4) - building curiosity
  - Phrase 3: Upward reach (E4-G4-A4-C5-D5-C5-A4-G4) - sense of discovery
  - Phrase 4: Contemplative descent (D5-Bb4-A4-F4-E4-D4-F4-A4) - epic but introspective
  - Phrase 5: Building wonder (A4-C5-D5-E5-D5-C5-D5-Bb4) - expanding horizons
  - Phrase 6: Mysterious descent (G4-E4-D4-F4-G4-A4-Bb4-A4) - closing the exploration
  
- **Cello Bass**: Half-note pulse (D2, A2, D2, F2, repeating) for rhythmic foundation
- **Pad Synth**: Two long chords (27 quarter notes each) - D minor/Bb major color shift
- **French Horn**: Two sustained notes (27 quarter notes each) - subtle background support

**Goal**: Showcase the violin melody with contemplative, wondering quality. Chord progressions support a sense of entering an unknown space station.

### Outro/Loop Transition (48-60 seconds = 18 quarter notes)
- **Violin Melody**: Gentle 6-note descending phrase (D5-C5-A4-G4-F4-A4) then held D4 (6 quarter notes)
- **Cello Bass**: Single held D2 for entire outro (18 quarter notes)
- **Pad Synth**: Single held D minor chord (18 quarter notes) - fades slightly
- **French Horn**: Single held D4 for entire outro (18 quarter notes)

**Goal**: Gracefully transition melody back to foundation for loop. D minor resolution prepares for seamless repeat.

## Instrumentation & MIDI Programs

| Part | Instrument | MIDI Program | Velocity | Role |
|------|-----------|--------------|----------|------|
| Track 1 | Violin (Pad Strings) | 40 | 40-50 | Background sustain |
| Track 2 | Violin (Melody) | 40 | 85-93 | Primary melodic voice |
| Track 3 | Cello | 42 | 75-78 | Bass foundation & rhythm |
| Track 4 | French Horn | 60 | 30-48 | Subtle brass undertones |
| Track 5 | Pad Synth (New Age) | 88 | 35-68 | Atmospheric harmony |

## Velocity Guidelines Used

- **Lead melody (Violin)**: 85-93 - Bright, expressive, captures attention
- **Cello/Bass**: 75-78 - Solid foundation without overpowering
- **Pad synth**: 35-68 - Gradually builds atmosphere
- **Strings pad**: 40-50 - Subtle background
- **French horn**: 30-48 - Very subtle, adds depth without dominance

## Musical Characteristics

- **Melodic**: Uses D minor scale with color notes (Bb, C) for sci-fi mystery
- **Pacing**: Quarter note = ~0.667 seconds at 90 BPM, creating measured progression
- **Dynamics**: Slow build from intro (pp) through main theme (mp-mf) to outro resolution
- **Harmony**: D minor tonic with occasional Bb major inflections for harmonic color
- **Rhythm**: Half-note (2.0) and whole-note (3.0+) durations emphasize sustained, atmospheric quality

## Comparison with Original

| Aspect | Original | Improved |
|--------|----------|----------|
| Duration | ~120+ seconds | 62.8 seconds |
| Instrumentation | Unknown | 5 distinct parts |
| Atmosphere | Looping version | Structured intro-theme-outro |
| Key | Unknown | D minor (intentional) |
| Pace | Unknown | Contemplative, explores wonder |
| BPM | Varied | Fixed 90 BPM |

## Rendering Details

- **SoundFont**: FluidR3_GM.sf2 (General MIDI)
- **Sample Rate**: Standard (48kHz)
- **Bitrate**: 192kbps MP3
- **Dynamic Range Compression**: Applied to prevent clipping
- **Normalization**: -3dB headroom

## Notes for Use

1. **Looping**: The outro naturally transitions back to intro for seamless looping
2. **Mixing**: Front-loaded with quiet intro, builds through main theme, settles in outro
3. **Genre Fit**: Orchestral-rock classification maintained with string lead + subtle brass
4. **Game Context**: Perfect for exploration phase of space shooter - invokes wonder without action urgency
5. **Pacing**: 90 BPM matches game design for calm/exploration tempo

## Composition Method

Created using music21 library in Python:
- Algorithmic composition with explicit note sequences
- MIDI program assignment via mido library
- FluidSynth rendering with dynamic range compression
- Pydub MP3 export at 192kbps

See `/home/titus/tydust/compose_phase1_calm.py` for complete source code.
