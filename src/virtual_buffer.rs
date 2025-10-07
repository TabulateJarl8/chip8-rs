use std::fmt::Debug;

/// The screen width that we're emulating
const VIRTUAL_WIDTH: usize = 64;
/// The screen height that we're emulating
const VIRTUAL_HEIGHT: usize = 32;

/// The RGBA value of a pixel being on
const PIXEL_ON: u32 = 0xFFFFFFFF;
/// The RGBA value of a pixel being off
const PIXEL_OFF: u32 = 0x1A1A1AFF;

/// A virtual display for rendering CHIP-8 graphics at a scaled resolution
///
/// This represents a simple boolean pixel buffer where pixels can either be on or off. It acts as
/// a smaller screen, and can upscale to the specified factor.
pub struct VirtualDisplay {
    /// The internal boolean pixel buffer. Stored as a 1D array
    buffer: Vec<bool>,
    /// The scaled up width in pixels of the display buffer
    scaled_width: usize,
    /// The scaled up height in pixels of the display buffer
    scaled_height: usize,
    /// The scaling factor used to convert virtual pixels to real pixels
    scale_factor: usize,
}

impl VirtualDisplay {
    /// Construct a new [`VirtualDisplay`] with a given scale factor.
    ///
    /// The total buffer dimentions are determined by the virtual size multiplied by the scale
    /// factor.
    ///
    /// # Arguments
    /// * `scale_factor` - The number of real pixels per virtual pixel
    ///
    /// # Example
    /// ```
    /// let display = VirtualDisplay::new(10);
    /// // given a 64x32 virtual size
    /// assert_eq!(display.scaled_width(), 640);
    /// ```
    pub fn new(scale_factor: usize) -> Self {
        Self {
            buffer: vec![false; (VIRTUAL_WIDTH * scale_factor) * (VIRTUAL_HEIGHT * scale_factor)],
            scaled_width: VIRTUAL_WIDTH * scale_factor,
            scaled_height: VIRTUAL_HEIGHT * scale_factor,
            scale_factor,
        }
    }

    /// Returnes the scaled width in pixels
    pub const fn scaled_width(&self) -> usize {
        self.scaled_width
    }

    /// Returnes the scaled height in pixels
    pub const fn scaled_height(&self) -> usize {
        self.scaled_height
    }

    /// Clears the entire display by turning off all pixels
    pub fn clear(&mut self) {
        log::trace!("Clearing display");
        self.buffer.fill(false);
    }

    /// Renders the internal buffer into a given RGBA byte frame.
    ///
    /// Each pixel is expanded into four bytes. [`PIXEL_ON`] and [`PIXEL_OFF`] define the colors
    /// for on and off pixels repectively.
    ///
    /// # Arguments
    ///
    /// * `frame` - A mutable slice of bytes where the RGBA data should be written
    ///
    /// # Panics
    ///
    /// If the provided frame is not large enough to hold the display data
    pub fn render_to_buffer(&self, frame: &mut [u8]) {
        for (index, pixel_on) in self.buffer.iter().enumerate() {
            let rgba = if *pixel_on { PIXEL_ON } else { PIXEL_OFF };

            let start = index * 4;
            frame[start..start + 4].copy_from_slice(&rgba.to_be_bytes());
        }
    }

    /// Returns the state of a virtual pixel at the given coordinates.
    ///
    /// Coordinates automatically wrap if they overflow.
    pub fn get_pixel(&self, mut x: usize, mut y: usize) -> bool {
        x %= VIRTUAL_WIDTH;
        y %= VIRTUAL_HEIGHT;

        let real_x = x * self.scale_factor;
        let real_y = y * self.scale_factor;
        let real_index = real_y * self.scaled_width + real_x;

        self.buffer[real_index]
    }

    /// Sets a virtual pixel at the given coordinates to the given state.
    ///
    /// Each virtual pixel affects a `scale_factor * scale_factor` block of real pixels.
    /// Pixels are XORed with the new state to allow for sprite drawing behavior.
    ///
    /// Returns `true` if setting the pixel caused a collision
    pub fn set_pixel(&mut self, mut x: usize, mut y: usize, state: bool) -> bool {
        let collision = self.get_pixel(x, y) && state;

        x %= VIRTUAL_WIDTH;
        y %= VIRTUAL_HEIGHT;

        let start_x = x * self.scale_factor;
        let start_y = y * self.scale_factor;
        let end_x = (x + 1) * self.scale_factor;
        let end_y = (y + 1) * self.scale_factor;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let index = y * self.scaled_width + x;
                if let Some(pixel) = self.buffer.get_mut(index) {
                    *pixel ^= state;
                }
            }
        }

        collision
    }

    /// Draws a sprite on the display at `(x, y)` using the provided bytes of pixel data.
    ///
    /// Each byte in `pixels` represents one row of 8 bits. Drawing wraps around the screen
    /// edges.
    ///
    /// Returns `true` if any pixel collisions occurred during drawing
    ///
    /// # Arguments
    /// * `x` - The x-coordinate of the sprite's top-left corner
    /// * `y` - The y-coordinate of the sprite's top-left corner
    /// * `num_rows` - The number of rows (bytes) in the sprite
    /// * `pixels` - The byte slice representing the sprite data
    /// * `clipping` - whether or not sprites should be clipped or wrapped on the edge
    pub fn draw_sprite(
        &mut self,
        x: usize,
        y: usize,
        num_rows: usize,
        pixels: &[u8],
        clip_sprite: bool,
    ) -> bool {
        let mut collision = false;

        for (row_index, row) in pixels.iter().enumerate().take(num_rows) {
            let coord_y = y + row_index;

            for bit in 0..8 {
                let coord_x = x + bit;

                let value = row & (1 << (7 - bit));
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
