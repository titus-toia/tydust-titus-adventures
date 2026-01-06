# Vertical Shooter Audio Design
## Industrial/Military Aesthetic (DemonStar Style)

---

## Audio Philosophy

This sound design follows an **industrial military** palette: metallic impacts, electronic systems, mechanical thrust, and weighty explosions. Sounds should feel **chunky and satisfying** while remaining readable during intense bullet-hell moments.

### Core Palette
- **Materials**: Metal, electronics, combustion, energy
- **Character**: Punchy, industrial, mechanical, aggressive
- **Frequency strategy**: HPF on most sounds to prevent mud during chaos

---

## Volume Hierarchy

| Category | Volume | Rationale |
|----------|--------|-----------|
| Player weapons | 0.4–0.5 | Core feedback, frequent |
| Enemy weapons | 0.25–0.35 | Present but not overwhelming |
| Explosions (small) | 0.35–0.45 | Satisfying but frequent |
| Explosions (large) | 0.5–0.65 | Impactful, less frequent |
| Power-ups | 0.45–0.55 | Rewarding, noticeable |
| UI sounds | 0.3–0.4 | Clear but not intrusive |
| Warnings | 0.5–0.6 | Must cut through action |
| Boss entrance | 0.6–0.7 | Dramatic moment |
| Background music | 0.2–0.3 | Atmosphere, never dominate |

---

## Player Ship Sounds

### Primary Weapon (Rapid Fire)
**Function**: Core gameplay feedback, fires 5-15x per second

| Spec | Value |
|------|-------|
| Duration | 80–120 ms |
| Volume | 0.4 |
| HPF | 200 Hz |
| Decay | 60–100 ms (tight) |

**AI Prompt (ElevenLabs/AudioGen):**
```
Short punchy electronic laser shot, metallic sci-fi zap with bright transient 2-4 kHz,
tight 80ms duration, HPF 200 Hz, crisp attack with minimal tail, industrial military
weapon sound, no reverb, no musical tone, suitable for rapid fire
```

**Variation prompt (for 3-4 variants):**
```
Pitch variance ±2 semitones, onset jitter 5-15ms for layered playback,
maintain consistent HPF and transient character
```

---

### Secondary Weapon - Missile Launch
**Function**: Heavier weapon, less frequent, more impact

| Spec | Value |
|------|-------|
| Duration | 300–450 ms |
| Volume | 0.5 |
| HPF | 150 Hz |
| Decay | 200–300 ms |

**AI Prompt:**
```
Rocket missile launch whoosh with ignition burst, industrial military propulsion,
initial combustion pop followed by rushing thrust trail, 350ms duration,
HPF 150 Hz, mid-frequency body 500Hz-2kHz, slight low rumble undertone,
mechanical military aesthetic, no musical tone
```

---

### Secondary Weapon - Bomb Drop
**Function**: Heavy ordnance, satisfying weight

| Spec | Value |
|------|-------|
| Duration | 250–350 ms |
| Volume | 0.45 |
| HPF | 120 Hz |

**AI Prompt:**
```
Heavy bomb release clunk followed by descending whistle, industrial military ordnance,
metallic mechanical release mechanism, 300ms, HPF 120 Hz, weighty satisfying drop,
no explosion (separate sound), no reverb
```

---

### Engine Thrust / Afterburner
**Function**: Movement feedback, loopable

| Spec | Value |
|------|-------|
| Duration | 2000 ms (loop) |
| Volume | 0.25 |
| HPF | 180 Hz |

**AI Prompt:**
```
Industrial spacecraft thruster hum, steady mechanical propulsion loop,
low-mid frequency rumble 150-400 Hz with subtle high-frequency turbine whine,
2 second seamless loop, HPF 180 Hz, consistent energy suitable for looping,
military jet engine character, no musical tone
```

---

### Shield Activation
**Function**: Defensive ability feedback

| Spec | Value |
|------|-------|
| Duration | 400–500 ms |
| Volume | 0.5 |
| HPF | 180 Hz |

**AI Prompt:**
```
Energy shield activation, electronic force field powering up, ascending electronic hum
with crystalline shimmer overtone, 450ms duration, HPF 180 Hz, sci-fi military
defensive system sound, bright 4-8 kHz shimmer with mid body, no musical tone
```

---

### Shield Hit (Absorbing Damage)
**Function**: Feedback that shield is working

| Spec | Value |
|------|-------|
| Duration | 150–200 ms |
| Volume | 0.45 |
| HPF | 200 Hz |

**AI Prompt:**
```
Electric energy shield impact, crackling absorption hit, bright electronic zap
with dispersing crackle tail, 180ms duration, HPF 200 Hz, transient 2-4 kHz,
sci-fi force field deflection, satisfying but not harsh, no reverb
```

---

### Player Ship Explosion (Death)
**Function**: Dramatic failure moment, needs weight

| Spec | Value |
|------|-------|
| Duration | 800–1200 ms |
| Volume | 0.65 |
| HPF | 80 Hz |

**AI Prompt:**
```
Massive spacecraft explosion with metallic debris, deep booming detonation with
extended rumble and scattering metal shrapnel, 1000ms duration, HPF 80 Hz,
sub-bass impact 60-100 Hz, mid crunch 400-800 Hz, high debris sparkle 4-8 kHz,
cinematic military destruction, short room reverb 0.3s 15% wet
```

---

### Power-Up Collection
**Function**: Reward feedback, must feel good

| Spec | Value |
|------|-------|
| Duration | 250–350 ms |
| Volume | 0.5 |
| HPF | 200 Hz |

**AI Prompt:**
```
Satisfying power-up collect sound, bright ascending electronic chime with
synthesized confirmation tone, 300ms duration, HPF 200 Hz, positive upward
pitch sweep 1-4 kHz, brief shimmer tail 6-10 kHz, rewarding pickup feedback,
military tech acquisition sound, non-musical but melodically pleasant
```

---

## Enemy Sounds

### Enemy Laser (Light)
**Function**: Threat indicator, frequent

| Spec | Value |
|------|-------|
| Duration | 60–100 ms |
| Volume | 0.3 |
| HPF | 250 Hz |

**AI Prompt:**
```
Enemy laser shot, thin electronic zap, shorter and higher pitched than player weapon,
70ms duration, HPF 250 Hz, bright transient 3-5 kHz, clearly distinct from player fire,
threatening but not dominant, no reverb, tight decay
```

---

### Enemy Cannon (Heavy)
**Function**: Dangerous threat, needs attention

| Spec | Value |
|------|-------|
| Duration | 150–200 ms |
| Volume | 0.4 |
| HPF | 150 Hz |

**AI Prompt:**
```
Heavy enemy cannon fire, chunky industrial shot with metallic punch,
180ms duration, HPF 150 Hz, weighty mid-frequency impact 300-600 Hz,
military artillery character, threatening and distinct from player weapons
```

---

### Enemy Explosion - Small
**Function**: Fodder enemy destruction

| Spec | Value |
|------|-------|
| Duration | 200–300 ms |
| Volume | 0.4 |
| HPF | 180 Hz |

**AI Prompt:**
```
Small metallic explosion, quick mechanical destruction with debris scatter,
250ms duration, HPF 180 Hz, punchy mid transient 400-800 Hz,
brief metal shrapnel detail, satisfying but compact, no reverb
```

---

### Enemy Explosion - Medium
**Function**: Larger enemy destruction

| Spec | Value |
|------|-------|
| Duration | 400–550 ms |
| Volume | 0.5 |
| HPF | 120 Hz |

**AI Prompt:**
```
Medium spacecraft explosion, fuller detonation with extended debris,
500ms duration, HPF 120 Hz, initial punch with rumbling decay,
metallic destruction with body 200-600 Hz, satisfying mid-size kill
```

---

### Enemy Explosion - Large
**Function**: Major enemy / mini-boss destruction

| Spec | Value |
|------|-------|
| Duration | 700–900 ms |
| Volume | 0.6 |
| HPF | 100 Hz |

**AI Prompt:**
```
Large spacecraft explosion, deep booming detonation with extended rumble,
800ms duration, HPF 100 Hz, powerful low-end impact 80-200 Hz,
crackling debris scatter, cinematic destruction, short room 0.25s 12% wet
```

---

### Boss Warning / Entrance
**Function**: Dramatic tension builder

| Spec | Value |
|------|-------|
| Duration | 2000–3000 ms |
| Volume | 0.65 |
| HPF | 60 Hz |

**AI Prompt:**
```
Ominous boss warning siren, deep industrial horn with mechanical tension,
descending then ascending threatening tone, 2500ms duration, HPF 60 Hz,
low brass-like frequency 80-200 Hz with distorted overtones,
military danger alert, dread-inducing approach sound, slight room reverb
```

---

### Boss Attack - Heavy Beam
**Function**: Major attack telegraph and execution

| Spec | Value |
|------|-------|
| Duration | 1500–2000 ms |
| Volume | 0.55 |
| HPF | 100 Hz |

**AI Prompt:**
```
Massive energy beam charging and firing, building electronic whine crescendo
followed by sustained heavy beam discharge, 1800ms total, HPF 100 Hz,
charging phase ascending 200-2000 Hz, firing phase sustained low rumble
with crackling energy, boss weapon devastation sound
```

---

## UI / Feedback Sounds

### Menu Selection / Confirm
**Function**: Navigation feedback

| Spec | Value |
|------|-------|
| Duration | 100–150 ms |
| Volume | 0.35 |
| HPF | 200 Hz |

**AI Prompt:**
```
Clean UI selection click, crisp electronic confirmation blip, military computer
interface sound, 120ms duration, HPF 200 Hz, bright transient 2-4 kHz,
professional and responsive, no reverb, non-musical
```

---

### Menu Back / Cancel
**Function**: Negative navigation feedback

| Spec | Value |
|------|-------|
| Duration | 120–180 ms |
| Volume | 0.3 |
| HPF | 200 Hz |

**AI Prompt:**
```
UI cancel sound, soft descending electronic blip, gentle rejection tone,
150ms duration, HPF 200 Hz, slightly lower pitched than confirm,
not harsh or punishing, subtle questioning quality, non-musical
```

---

### Score Tally / Counting
**Function**: End-of-level score feedback

| Spec | Value |
|------|-------|
| Duration | 50–80 ms |
| Volume | 0.25 |
| HPF | 250 Hz |

**AI Prompt:**
```
Quick score tick sound, rapid electronic counting blip, suitable for
fast repetition during score tally, 60ms duration, HPF 250 Hz,
bright clean tick 2-4 kHz, no tail, military computer tabulation
```

---

### Extra Life Gained
**Function**: Major reward feedback

| Spec | Value |
|------|-------|
| Duration | 500–700 ms |
| Volume | 0.55 |
| HPF | 150 Hz |

**AI Prompt:**
```
Extra life fanfare, triumphant ascending electronic flourish with
confirmation resolution, 600ms duration, HPF 150 Hz, bright celebratory
tone building to satisfying peak, military achievement sound,
rewarding and memorable, short room 0.2s 10% wet
```

---

### Warning Alarm (Low Health)
**Function**: Urgent player attention

| Spec | Value |
|------|-------|
| Duration | 400 ms (loop) |
| Volume | 0.5 |
| HPF | 200 Hz |

**AI Prompt:**
```
Urgent warning alarm beep, rapid two-tone alert pattern, military cockpit
danger warning, 400ms loop duration, HPF 200 Hz, alternating tones
800 Hz and 1200 Hz, anxiety-inducing but not painful, clear danger signal
```

---

### Level Complete Fanfare
**Function**: Victory celebration

| Spec | Value |
|------|-------|
| Duration | 1500–2000 ms |
| Volume | 0.6 |
| HPF | 120 Hz |

**AI Prompt:**
```
Level complete victory fanfare, triumphant military brass-style flourish
with electronic enhancement, ascending progression to satisfying resolution,
1800ms duration, HPF 120 Hz, celebratory and earned feeling,
industrial synth brass character, room reverb 0.3s 15% wet
```

---

### Game Over
**Function**: Failure state, somber but not punishing

| Spec | Value |
|------|-------|
| Duration | 2000–2500 ms |
| Volume | 0.5 |
| HPF | 100 Hz |

**AI Prompt:**
```
Game over sound, somber descending electronic tone with fading energy,
military shutdown sequence feeling, 2200ms duration, HPF 100 Hz,
not harsh or mocking, dignified ending with slight melancholy,
powering down systems character, room reverb 0.4s 20% wet
```

---

## Frequency Band Allocation Summary

| Sound Category | Primary Band | Avoid Conflict With |
|----------------|--------------|---------------------|
| Player weapons | 1–4 kHz | Enemy weapons |
| Enemy weapons | 2–5 kHz (higher) | Player weapons |
| Small explosions | 400–800 Hz | Large explosions |
| Large explosions | 80–400 Hz | Small explosions |
| UI sounds | 2–4 kHz | Weapons |
| Warnings | 800–1500 Hz | Everything (cuts through) |
| Power-ups | 1–6 kHz (sweep) | Static sounds |
| Engine loop | 150–400 Hz | Explosions |

---

## Implementation Notes

### Rapid Fire Handling
For weapons firing 10+ times per second:
- Use 3-4 pitch variants (±2 semitones)
- Apply onset jitter (5-15ms) to prevent phase issues
- Consider volume ducking on repeated rapid fires

### Explosion Layering
Large explosions should layer:
1. Initial transient punch (0-50ms)
2. Body rumble (50-400ms)
3. Debris tail (400-800ms)

### Music Ducking
During boss entrances and major events:
- Duck background music by 6-10dB
- Fade back in over 500-1000ms

---

## File Naming Convention

```
player_weapon_primary_01.wav
player_weapon_missile_01.wav
player_shield_activate.wav
player_shield_hit_01.wav
player_explosion.wav
player_powerup_collect.wav

enemy_weapon_laser_01.wav
enemy_weapon_cannon_01.wav
enemy_explosion_small_01.wav
enemy_explosion_medium_01.wav
enemy_explosion_large_01.wav
boss_warning.wav
boss_attack_beam.wav

ui_menu_select.wav
ui_menu_back.wav
ui_score_tick.wav
ui_extra_life.wav
ui_warning_alarm.wav
ui_level_complete.wav
ui_game_over.wav
```
