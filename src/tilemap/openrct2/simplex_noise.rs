use super::height_map::HeightMap;
use super::settings::OpenRct2Settings;
use bevy::math::IVec2;
use rand::Rng;
use rand_pcg::Pcg32;

/// Port of OpenRCT2's simplex noise helper with fractional Brownian motion support.
pub struct SimplexNoise {
    perm: [u8; 512],
}

impl SimplexNoise {
    pub fn new(seed: u64) -> Self {
        let mut rng = Pcg32::new(seed, seed ^ 0x9E37_79B9_7F4A_7C15);
        let mut perm = [0u8; 512];
        for value in perm.iter_mut() {
            *value = rng.gen::<u8>();
        }
        Self { perm }
    }

    fn grad(hash: u8, x: f32, y: f32) -> f32 {
        let h = hash & 7;
        let u = if h < 4 { x } else { y };
        let v = if h < 4 { y } else { x };
        let u_term = if (h & 1) != 0 { -u } else { u };
        let v_term = if (h & 2) != 0 { -2.0 * v } else { 2.0 * v };
        u_term + v_term
    }

    fn fast_floor(x: f32) -> i32 {
        if x > 0.0 {
            x as i32
        } else {
            (x as i32) - 1
        }
    }

    fn sample(&self, x: f32, y: f32) -> f32 {
        const F2: f32 = 0.366_025_4;
        const G2: f32 = 0.211_324_87;

        let s = (x + y) * F2;
        let xs = x + s;
        let ys = y + s;
        let i = Self::fast_floor(xs);
        let j = Self::fast_floor(ys);

        let t = (i + j) as f32 * G2;
        let x0 = x - (i as f32 - t);
        let y0 = y - (j as f32 - t);

        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - i1 as f32 + G2;
        let y1 = y0 - j1 as f32 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;

        let mut n0 = 0.0;
        let mut n1 = 0.0;
        let mut n2 = 0.0;

        let mut t0 = 0.5 - x0 * x0 - y0 * y0;
        if t0 > 0.0 {
            t0 *= t0;
            let hash = self.perm[ii + self.perm[jj] as usize];
            n0 = t0 * t0 * Self::grad(hash, x0, y0);
        }

        let mut t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 > 0.0 {
            t1 *= t1;
            let hash = self.perm[ii + i1 as usize + self.perm[jj + j1 as usize] as usize];
            n1 = t1 * t1 * Self::grad(hash, x1, y1);
        }

        let mut t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 > 0.0 {
            t2 *= t2;
            let hash = self.perm[ii + 1 + self.perm[jj + 1] as usize];
            n2 = t2 * t2 * Self::grad(hash, x2, y2);
        }

        40.0 * (n0 + n1 + n2)
    }

    pub fn fractal_noise(
        &self,
        x: f32,
        y: f32,
        mut frequency: f32,
        octaves: i32,
        lacunarity: f32,
        persistence: f32,
    ) -> f32 {
        let mut total = 0.0;
        let mut amplitude = persistence;
        for _ in 0..octaves {
            total += self.sample(x * frequency, y * frequency) * amplitude;
            frequency *= lacunarity;
            amplitude *= persistence;
        }
        total
    }
}

/// Fill the height map with OpenRCT2-style simplex noise.
pub fn generate_simplex_noise(
    settings: &OpenRct2Settings,
    seed: u64,
    origin: IVec2,
    height_map: &mut HeightMap,
) {
    let noise = SimplexNoise::new(seed);
    let map_width_samples =
        (settings.map_size.x.max(1) as f32) * height_map.density as f32;
    let freq = settings.simplex_base_freq as f32 / 100.0 * (1.0 / map_width_samples);
    let octaves = settings.simplex_octaves.max(1);

    let low = (settings.heightmap_low / 2).max(0);
    let high = (settings.heightmap_high / 2 - low).max(1);

    for y in 0..height_map.height {
        for x in 0..height_map.width {
            let global_x = origin.x + x as i32;
            let global_y = origin.y + y as i32;
            let n = noise
                .fractal_noise(global_x as f32, global_y as f32, freq, octaves, 2.0, 0.65)
                .clamp(-1.0, 1.0);
            let normalised = (n + 1.0) * 0.5;
            let value = low as f32 + normalised * high as f32;
            height_map.set(IVec2::new(x as i32, y as i32), value.round() as u8);
        }
    }
}

/// Box blur smoothing identical to OpenRCT2's `smoothHeightMap`.
pub fn smooth_height_map(iterations: u32, height_map: &mut HeightMap) {
    if iterations == 0 {
        return;
    }

    let mut buffer = height_map.clone();
    for _ in 0..iterations {
        buffer.data_mut().copy_from_slice(height_map.data());
        for y in 1..height_map.height - 1 {
            for x in 1..height_map.width - 1 {
                let mut total = 0u32;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let sample = buffer.get(IVec2::new(x as i32 + dx, y as i32 + dy));
                        total += sample as u32;
                    }
                }
                let avg = (total / 9) as u8;
                height_map.set(IVec2::new(x as i32, y as i32), avg);
            }
        }
    }
}
