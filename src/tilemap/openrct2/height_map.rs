use bevy::math::IVec2;

/// Mirror of OpenRCT2's `HeightMap` helper.
#[derive(Clone)]
pub struct HeightMap {
    pub width: usize,
    pub height: usize,
    pub density: u8,
    data: Vec<u8>,
}

impl HeightMap {
    pub fn new(target_width: usize, target_height: usize) -> Self {
        let len = target_width * target_height;
        Self {
            width: target_width,
            height: target_height,
            density: 1,
            data: vec![0; len],
        }
    }

    pub fn with_density(base_width: usize, base_height: usize, density: u8) -> Self {
        let width = base_width * density as usize;
        let height = base_height * density as usize;
        let len = width * height;
        Self {
            width,
            height,
            density,
            data: vec![0; len],
        }
    }

    #[inline]
    pub fn get(&self, pos: IVec2) -> u8 {
        let idx = self.index(pos);
        self.data[idx]
    }

    #[inline]
    pub fn set(&mut self, pos: IVec2, value: u8) {
        let idx = self.index(pos);
        self.data[idx] = value;
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    #[inline]
    fn index(&self, pos: IVec2) -> usize {
        let x = pos.x.clamp(0, (self.width - 1) as i32) as usize;
        let y = pos.y.clamp(0, (self.height - 1) as i32) as usize;
        y * self.width + x
    }
}
