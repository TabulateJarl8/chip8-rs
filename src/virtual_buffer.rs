use std::fmt::Debug;

const VIRTUAL_WIDTH: usize = 64;
const VIRTUAL_HEIGHT: usize = 32;
const SPRITE_WIDTH: usize = 8;

const PIXEL_ON: u32 = 0xFFFFFF;
const PIXEL_OFF: u32 = 0x1A1A1A;

pub struct VirtualDisplay {
    buffer: Vec<bool>,
    scaled_width: usize,
    scaled_height: usize,
    scale_factor: usize,
}

impl VirtualDisplay {
    pub fn new(scale_factor: usize) -> Self {
        Self {
            buffer: vec![false; (VIRTUAL_WIDTH * scale_factor) * (VIRTUAL_HEIGHT * scale_factor)],
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

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn to_framebuffer(&self) -> Vec<u32> {
        self.buffer
            .iter()
            .map(|v| match v {
                true => PIXEL_ON,
                false => PIXEL_OFF,
            })
            .collect::<Vec<u32>>()
    }

    pub fn get_pixel(&self, mut x: usize, mut y: usize) -> bool {
        x %= VIRTUAL_WIDTH;
        y %= VIRTUAL_HEIGHT;

        let real_x = x * self.scale_factor;
        let real_y = y * self.scale_factor;
        let real_index = real_y * self.scaled_width + real_x;

        self.buffer[real_index]
    }

    pub fn set_pixel(&mut self, mut x: usize, mut y: usize, state: bool) -> bool {
        x %= VIRTUAL_WIDTH;
        y %= VIRTUAL_HEIGHT;

        let start_x = x * self.scale_factor;
        let start_y = y * self.scale_factor;
        let end_x = (x + 1) * self.scale_factor;
        let end_y = (y + 1) * self.scale_factor;

        let mut collision = false;
        for y in start_y..end_y {
            for x in start_x..end_x {
                let index = y * self.scaled_width + x;
                if let Some(pixel) = self.buffer.get_mut(index) {
                    if *pixel == true && state == false {
                        collision = true;
                    }

                    *pixel ^= state;
                }
            }
        }

        collision
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, num_rows: usize, pixels: &[u8]) -> bool {
        let mut collision = false;

        for row in 0..num_rows {
            let bits = pixels[row];
            let coord_y = y + row;

            for bit in 0..8 {
                let coord_x = x + bit;

                let value = bits & (1 << 7 - bit);
                if value > 0 {
                    collision |= self.set_pixel(coord_x, coord_y, true);
                }

                if coord_x == VIRTUAL_WIDTH - 1 {
                    break;
                }
            }

            if coord_y == VIRTUAL_HEIGHT - 1 {
                break;
            }
        }

        collision
    }
}

impl Debug for VirtualDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualDisplay")
            .field("scaled_width", &self.scaled_width)
            .field("scaled_height", &self.scaled_height)
            .field("scale_factor", &self.scale_factor)
            .finish()
    }
}

impl<'a> IntoIterator for &'a VirtualDisplay {
    type Item = &'a bool;
    type IntoIter = std::slice::Iter<'a, bool>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.iter()
    }
}

impl<'a> IntoIterator for &'a mut VirtualDisplay {
    type Item = &'a mut bool;
    type IntoIter = std::slice::IterMut<'a, bool>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.iter_mut()
    }
}

impl IntoIterator for VirtualDisplay {
    type Item = bool;
    type IntoIter = std::vec::IntoIter<bool>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}
