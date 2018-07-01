//! Types, traits, and data structures for video display.
use super::{OktoError, OktoErrorKind, OktoResult};

/// Maximum height of the display
pub const DISPLAY_HEIGHT: usize = 64;
/// Maximum width of the display
pub const DISPLAY_WIDTH: usize = 128;
/// Maximum number of bytes in a sprite
pub const MAX_SPRITE_BYTES: usize = 15;
/// Number of pixels encoded in each byte of sprite data
pub const PIXELS_PER_BYTE: usize = 8;

/// Display state data
pub struct Display {
    /// Frame buffer for the video display
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::display;
    /// # let display = display::Display::new();
    /// assert_eq!(display::DISPLAY_HEIGHT, display.data.len());
    /// assert_eq!(display::DISPLAY_WIDTH, display.data[0].len());
    /// ```
    pub data: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    /// Indicates whether or not the display is in high resolution mode
    pub high_resolution: bool,
}

/// Implementation of the display
impl Display {
    /// Initialize a new display data structure. The display is initially in
    /// low resolution mode (64x32) and the frame buffer is initialized to all
    /// zero values.
    pub fn new() -> Self {
        Self {
            data: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            high_resolution: false,
        }
    }

    /// Clear the frame buffer for the display.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::display;
    /// # let mut display = display::Display::new();
    /// display.data[10][20] = 0x23;
    /// assert_eq!(0x23, display.data[10][20]);
    /// display.clear();
    /// assert_eq!(0x00, display.data[10][20]);
    /// ```
    pub fn clear(&mut self) {
        for row in 0..self.height() {
            for column in 0..self.width() {
                self.data[row][column] = 0;
            }
        }
    }

    /// Returns the current effective height of the display give its mode.
    ///
    /// # Examples
    ///
    /// When in low resolution mode, the display size should be halved:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::display;
    /// let mut display = display::Display::new();
    /// assert!(!display.high_resolution);
    /// assert_eq!((display::DISPLAY_HEIGHT / 2), display.height());
    /// assert_eq!((display::DISPLAY_WIDTH / 2), display.width());
    /// ```
    ///
    /// When in high resolution mode, the display should take up the full
    /// width and height:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::display;
    /// let mut display = display::Display::new();
    /// display.high_resolution = true;
    /// assert_eq!(display::DISPLAY_HEIGHT, display.height());
    /// assert_eq!(display::DISPLAY_WIDTH, display.width());
    /// ```
    pub fn height(&self) -> usize {
        if self.high_resolution {
            DISPLAY_HEIGHT
        } else {
            DISPLAY_HEIGHT / 2
        }
    }

    /// Returns the current effective width of the display given its mode.
    pub fn width(&self) -> usize {
        if self.high_resolution {
            DISPLAY_WIDTH
        } else {
            DISPLAY_WIDTH / 2
        }
    }

    /// Draw sprite data of a pre-specified size onto the screen at the given
    /// coordinates. Returns a value indicating whether or not any pixels were
    /// erased during the rendering process.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::display;
    /// # let mut display = display::Display::new();
    /// let pixels_erased = display.draw(0, 0, &[0xFF, 0x1F]).unwrap();
    /// assert!(!pixels_erased);
    /// assert_eq!(&display.data[0][0..8], &[1, 1, 1, 1, 1, 1, 1, 1]);
    /// assert_eq!(&display.data[1][0..8], &[0, 0, 0, 1, 1, 1, 1, 1]);
    /// ```
    pub fn draw(&mut self, x: usize, y: usize, sprite_data: &[u8]) -> OktoResult<bool> {
        if sprite_data.len() > MAX_SPRITE_BYTES {
            return Err(OktoError::new(OktoErrorKind::InvalidSprite));
        }

        let mut pixels_erased = false;

        for row in 0..sprite_data.len() {
            let ycoord = (y + row) % self.height();

            for column in 0..PIXELS_PER_BYTE {
                let desired_bit = PIXELS_PER_BYTE - column - 1;
                let pixel_data = (sprite_data[row] >> desired_bit) & 1;
                let xcoord = (x + column) % self.width();

                // If we will end up erasing a set pixel, set the flag register.
                if self.data[ycoord][xcoord] == 1 && pixel_data == 1 {
                    pixels_erased = true;
                }

                self.data[ycoord][xcoord] ^= pixel_data;
            }
        }

        Ok(pixels_erased)
    }

    pub fn draw_large(&mut self, x: usize, y: usize, sprite_data: &[u8]) -> OktoResult<bool> {
        if sprite_data.len() != 32 {
            return Err(OktoError::new(OktoErrorKind::AddressOutOfRange));
        }

        let mut pixels_erased = false;

        for yoffset in 0..16 {
            for xoffset in 0..2 {
                pixels_erased |= self.draw(
                    x + (xoffset * 8),
                    y + yoffset,
                    &[sprite_data[(yoffset * 2) + xoffset]]
                )?;
            }
        }

        Ok(pixels_erased)
    }

    /// Shift the contents of the frame-buffer down the given number of lines.
    /// Previous lines are filled with zeros after scrolling.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::display;
    /// # let mut display = display::Display::new();
    /// display.draw(0, 0, &[0xFF, 0x1F]).unwrap();
    /// assert_eq!(&display.data[0][0..8], &[1, 1, 1, 1, 1, 1, 1, 1]);
    /// assert_eq!(&display.data[1][0..8], &[0, 0, 0, 1, 1, 1, 1, 1]);
    /// display.scroll_down(2);
    /// assert_eq!(&display.data[2][0..8], &[1, 1, 1, 1, 1, 1, 1, 1]);
    /// assert_eq!(&display.data[3][0..8], &[0, 0, 0, 1, 1, 1, 1, 1]);
    /// ```
    pub fn scroll_down(&mut self, num_lines: usize) {
        let mut new_buffer = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

        for height in num_lines..self.data.len() {
            for width in 0..self.data[height].len() {
                new_buffer[height][width] = self.data[(height - num_lines)][width];
            }
        }

        self.data = new_buffer;
    }

    /// Scroll the contents of the frame buffer 4 pixels to the left.
    pub fn scroll_left(&mut self) {
        let mut new_buffer = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

        for height in 0..DISPLAY_HEIGHT {
            for width in 0..(DISPLAY_WIDTH - 4) {
                new_buffer[height][width] = self.data[height][width + 4];
            }
        }

        self.data = new_buffer;
    }

    /// Scroll the contents of the frame buffer 4 pixels to the right.
    pub fn scroll_right(&mut self) {
        let mut new_buffer = [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

        for height in 0..DISPLAY_HEIGHT {
            for width in 4..DISPLAY_WIDTH {
                new_buffer[height][width] = self.data[height][width - 4];
            }
        }

        self.data = new_buffer;
    }
}
