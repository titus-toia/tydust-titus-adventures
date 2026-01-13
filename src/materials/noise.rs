use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Resource, Clone)]
pub struct EffectsNoiseTexture(pub Handle<Image>);

pub fn generate_noise_texture(images: &mut Assets<Image>) -> Handle<Image> {
	let size = 256usize;
	let mut data = vec![0u8; size * size * 4];

	for y in 0..size {
		for x in 0..size {
			let noise = perlin_noise(x as f32 / 32.0, y as f32 / 32.0);
			let value = ((noise * 0.5 + 0.5) * 255.0) as u8;
			let idx = (y * size + x) * 4;
			data[idx] = value;
			data[idx + 1] = value;
			data[idx + 2] = value;
			data[idx + 3] = 255;
		}
	}

	let image = Image::new(
		Extent3d {
			width: size as u32,
			height: size as u32,
			depth_or_array_layers: 1,
		},
		TextureDimension::D2,
		data,
		TextureFormat::Rgba8UnormSrgb,
		RenderAssetUsages::RENDER_WORLD,
	);

	images.add(image)
}

fn perlin_noise(x: f32, y: f32) -> f32 {
	let xi = x.floor() as i32;
	let yi = y.floor() as i32;
	let xf = x - x.floor();
	let yf = y - y.floor();

	let u = fade(xf);
	let v = fade(yf);

	let aa = hash_2d(xi, yi);
	let ab = hash_2d(xi, yi + 1);
	let ba = hash_2d(xi + 1, yi);
	let bb = hash_2d(xi + 1, yi + 1);

	let x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1.0, yf), u);
	let x2 = lerp(grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0), u);

	lerp(x1, x2, v)
}

fn fade(t: f32) -> f32 {
	t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a + t * (b - a)
}

fn hash_2d(x: i32, y: i32) -> i32 {
	let n = x.wrapping_add(y.wrapping_mul(57));
	let n = (n << 13) ^ n;
	n.wrapping_mul(n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221))
		.wrapping_add(1376312589) & 0x7fffffff
}

fn grad(hash: i32, x: f32, y: f32) -> f32 {
	let h = hash & 3;
	let u = if h < 2 { x } else { y };
	let v = if h < 2 { y } else { x };
	(if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}
