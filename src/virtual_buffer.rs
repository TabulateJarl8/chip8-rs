const VIRTUAL_WIDTH: usize = 64;
const VIRTUAL_HEIGHT: usize = 32;

pub enum PixelState {
    On,
    Off,
}

impl PixelState {
    pub const fn value(&self) -> u32 {
        match self {
            PixelState::On => 0xFFFFFF,
            PixelState::Off => 0x1A1A1A,
        }
    }
}

pub struct VirtualDisplay {
    buffer: Vec<u32>,
    scaled_width: usize,
    scaled_height: usize,
    scale_factor: usize,
}

impl VirtualDisplay {
    pub fn new(scale_factor: usize) -> Self {
        Self {
            buffer: vec![0_u32; (VIRTUAL_WIDTH * scale_factor) * (VIRTUAL_HEIGHT * scale_factor)],
            scaled_width: VIRTUAL_WIDTH * scale_factor,
            scaled_height: VIRTUAL_HEIGHT * scale_factor,
            scale_factor,
        }
    }

    pub const fn scaled_width(&self) -> usize {
        self.scaled_width
    }

    pub const fn scaled_height(&self) -> usize {
        self.scaled_height
    }

    pub fn to_framebuffer(&self) -> &[u32] {
        &self.buffer
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<u32> {
        if x >= VIRTUAL_WIDTH || y >= VIRTUAL_HEIGHT {
            return None;
        }

        let real_x = x * self.scale_factor;
        let real_y = y * self.scale_factor;
        let real_index = real_y * self.scaled_width + real_x;

        self.buffer.get(real_index).copied()
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: PixelState) {
        if x >= VIRTUAL_WIDTH || y >= VIRTUAL_HEIGHT {
            return;
        }

        let start_x = x * self.scale_factor;
        let start_y = y * self.scale_factor;
        let end_x = (x + 1) * self.scale_factor;
        let end_y = (y + 1) * self.scale_factor;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let index = y * self.scaled_width + x;
                if let Some(pixel) = self.buffer.get_mut(index) {
                    *pixel = value.value();
                }
            }
        }
    }
}

impl<'a> IntoIterator for &'a VirtualDisplay {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.iter()
    }
}

impl<'a> IntoIterator for &'a mut VirtualDisplay {
    type Item = &'a mut u32;
    type IntoIter = std::slice::IterMut<'a, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.iter_mut()
    }
}

impl IntoIterator for VirtualDisplay {
    type Item = u32;
    type IntoIter = std::vec::IntoIter<u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}
