#!/usr/bin/env python3
"""Generate sci-fi weapon sound effects using AudioCraft AudioGen"""

import os
import torch
import torchaudio
from audiocraft.models import AudioGen

def generate_weapon_sounds():
	os.makedirs("assets/sounds", exist_ok=True)

	model = AudioGen.get_pretrained('facebook/audiogen-medium')

	sounds = [
		{
			"filename": "basic_blaster_fire.wav",
			"description": "sharp electric laser zap, quick bright energy bolt sound, sci-fi weapon",
			"duration": 0.3
		},
		{
			"filename": "plasma_cannon_fire.wav",
			"description": "heavy deep plasma discharge, powerful energy weapon, bass rumble with electrical crackle",
			"duration": 0.5
		},
		{
			"filename": "wave_gun_fire.wav",
			"description": "undulating wavy energy burst, modulating pitch swoosh, sinuous sci-fi weapon sound",
			"duration": 0.4
		},
		{
			"filename": "spread_shot_fire.wav",
			"description": "rapid scatter burst, multiple energy pellets firing, quick staccato pops",
			"duration": 0.3
		},
		{
			"filename": "missile_launch.wav",
			"description": "missile launch whoosh, rocket ignition and powerful thrust, deep launch sound",
			"duration": 0.6
		},
		{
			"filename": "laser_array_fire.wav",
			"description": "rapid succession of thin laser beams, fast pew-pew-pew burst",
			"duration": 0.2
		}
	]

	print("Generating weapon sound effects...")

	for sound in sounds:
		print(f"\nGenerating {sound['filename']}...")
		print(f"  Description: {sound['description']}")
		print(f"  Duration: {sound['duration']}s")

		model.set_generation_params(duration=sound['duration'])

		with torch.no_grad():
			wav = model.generate([sound['description']])

		output_path = os.path.join("assets/sounds", sound['filename'])
		torchaudio.save(output_path, wav[0].cpu(), sample_rate=16000)

		print(f"  ✓ Saved to {output_path}")

	print("\n✨ All weapon sounds generated successfully!")
	print(f"Files saved to: assets/sounds/")

if __name__ == "__main__":
	generate_weapon_sounds()
