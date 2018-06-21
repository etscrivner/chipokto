//! Types, traits, and data structures for video display.

/// Maximum height of the display
pub const DISPLAY_HEIGHT: usize = 64;
/// Maximum width of the display
pub const DISPLAY_WIDTH: usize = 128;
/// Maximum number of bytes in a sprite
pub const MAX_SPRITE_BYTES: usize = 15;
/// Number of pixels encoded in each byte of sprite data
pub const PIXELS_PER_BYTE: usize = 8;

/// Display state data
pub struct Display{
    /// Frame buffer for the video display
    pub data: [[u8; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    /// Indicates whether or not the display is in high resolution mode
    pub high_resolution: bool
}

/// Implementation of the display
impl Display {
    /// Initialize a new display data structure. The display is initially in
    /// low resolution mode (64x32) and the frame buffer is initialized to all
    /// zero values.
    pub fn new() -> Self {
        Self {
            data: [[0; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
            high_resolution: false
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
        self.data = [[0; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
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
    /// let pixels_erased = display.draw(0, 0, 2, &[0xFF, 0x1F]);
    /// assert!(!pixels_erased);
    /// assert_eq!(&display.data[0][0..8], &[1, 1, 1, 1, 1, 1, 1, 1]);
    /// assert_eq!(&display.data[1][0..8], &[0, 0, 0, 1, 1, 1, 1, 1]);
    /// ```
    pub fn draw(&mut self,
                x: usize,
                y: usize,
                size_bytes: usize,
                sprite_data: &[u8]) -> bool
    {
        assert!(sprite_data.len() == size_bytes);
        assert!(size_bytes <= MAX_SPRITE_BYTES);

        let mut pixels_erased = false;

        for row in 0..size_bytes {
            for column in 0..PIXELS_PER_BYTE {
                let desired_bit = PIXELS_PER_BYTE - column - 1;
                let pixel_data = (sprite_data[row] >> desired_bit) & 1;
                let ycoord = (y + row) % self.height();
                let xcoord = (x + column) % self.width();

                // If we will end up erasing a set pixel, set the flag register.
                if self.data[ycoord][xcoord] == 1 && pixel_data == 1 {
                    pixels_erased = true;
                }

                self.data[ycoord][xcoord] ^= pixel_data;
            }
        }

        pixels_erased
    }
}
