//! Chip8 memory access and loading
use super::cpu::{Address, Instruction, DEFAULT_PC_ADDRESS};
use super::{OktoError, OktoErrorKind, OktoResult};

/// The size of the Chip8 memory in bytes.
pub const MEMORY_SIZE_BYTES: usize = 0x1000;
/// The maximum size of a Chip8 ROM in bytes.
pub const MAX_ROM_SIZE_BYTES: usize = MEMORY_SIZE_BYTES - DEFAULT_PC_ADDRESS as usize;
/// The number of bytes per digit sprite.
pub const BYTES_PER_DIGIT_SPRITE: Address = 5;
/// The number of digit sprites.
pub const NUM_DIGIT_SPRITES: usize = 0x10 * BYTES_PER_DIGIT_SPRITE as usize;
/// Hexadecimal digits represented as 5 byte sprites.
pub const DIGIT_SPRITES: [u8; NUM_DIGIT_SPRITES] = [
    0xF0,
    0x90,
    0x90,
    0x90,
    0xF0, // 0
    0x20,
    0x60,
    0x20,
    0x20,
    0x70, // 1
    0xF0,
    0x10,
    0xF0,
    0x80,
    0xF0, // 2
    0xF0,
    0x10,
    0xF0,
    0x10,
    0xF0, // 3
    0x90,
    0x90,
    0xF0,
    0x10,
    0x10, // 4
    0xF0,
    0x80,
    0xF0,
    0x10,
    0xF0, // 5
    0xF0,
    0x80,
    0xF0,
    0x90,
    0xF0, // 6
    0xF0,
    0x10,
    0x20,
    0x40,
    0x40, // 7
    0xF0,
    0x90,
    0xF0,
    0x90,
    0xF0, // 8
    0xF0,
    0x90,
    0xF0,
    0x10,
    0xF0, // 9
    0xF0,
    0x90,
    0xF0,
    0x90,
    0x90, // A
    0xE0,
    0x90,
    0xE0,
    0x90,
    0xE0, // B
    0xF0,
    0x80,
    0x80,
    0x80,
    0xF0, // C
    0xE0,
    0x90,
    0x90,
    0x90,
    0xE0, // D
    0xF0,
    0x80,
    0xF0,
    0x80,
    0xF0, // E
    0xF0,
    0x80,
    0xF0,
    0x80,
    0x80, // F
];

/// Encapsulates memory subsystem for Chip8.
pub struct Memory {
    /// Byte array representing memory.
    pub data: [u8; MEMORY_SIZE_BYTES],
}

/// Creates a 16-bit value by concatenating two 8-bit values. Used to read
/// instructions from Chip8 memory.
///
/// # Examples
///
/// The function is extremely simple and can be used as follows:
///
/// ```
/// # extern crate okto;
/// # use okto::memory;
/// assert_eq!(0x1234, memory::bytes_to_word(&0x12, &0x34));
/// ```
pub fn bytes_to_word(high_byte: &u8, low_byte: &u8) -> u16 {
    ((*high_byte as u16) << 8) | (*low_byte as u16)
}

impl Memory {
    /// Initialize a new `Memory` data structure and clear it. Copies sprite
    /// data into reserved space in range 0x000 - 0x200.
    pub fn new() -> Self {
        let mut result = Self {
            data: [0; MEMORY_SIZE_BYTES],
        };
        result.data[0..NUM_DIGIT_SPRITES].copy_from_slice(&DIGIT_SPRITES);
        result
    }

    /// Returns the address of the sprite corresponding to the given hex digit.
    pub fn sprite_address_for_digit(&self, digit: u8) -> Option<Address> {
        if digit > 0xF {
            return None;
        }

        Some((digit as Address) * BYTES_PER_DIGIT_SPRITE)
    }

    /// Load a slice of bytes of a given size into memory starting at the
    /// given address.
    ///
    /// # Examples
    ///
    /// Loading a slice into memory is quite simple:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::memory::Memory;
    /// # let mut memory = Memory::new();
    /// let bytes: [u8; 6] = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    /// assert!(memory.load(&bytes, 0x200, bytes.len()).is_ok());
    /// ```
    ///
    /// An error is returned if the slice runs out of bounds:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::memory::Memory;
    /// # let mut memory = Memory::new();
    /// let bytes: [u8; 6] = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    /// assert!(memory.load(&bytes, 0xFFA, bytes.len()).is_err());
    /// ```
    pub fn load(&mut self, data: &[u8], start_address: Address, size: usize) -> OktoResult<()> {
        let start = start_address as usize;
        let end = (start_address as usize) + size;

        if end >= MEMORY_SIZE_BYTES as usize {
            return Err(OktoError::new(OktoErrorKind::RomTooLarge));
        }

        self.data[start..end].copy_from_slice(data);
        Ok(())
    }

    /// Write a byte of data at the given address in memory. If the address is
    /// invalid, then an error result is returned.
    ///
    /// # Examples
    ///
    /// A successful write simply returns a unit value:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::memory::Memory;
    /// # let mut memory = Memory::new();
    /// assert!(memory.write_byte(0x345, 0x12).is_ok());
    /// ```
    ///
    /// An attempt to write out of bounds will result in an error:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::memory::Memory;
    /// # let mut memory = Memory::new();
    /// assert!(memory.write_byte(0x1000, 0x12).is_err());
    /// ```
    pub fn write_byte(&mut self, address: Address, value: u8) -> OktoResult<()> {
        if address >= MEMORY_SIZE_BYTES as Address {
            return Err(OktoError::new(OktoErrorKind::AddressOutOfRange));
        }

        self.data[address as usize] = value;
        Ok(())
    }

    /// Read a slice of bytes from memory starting at the giving address.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::memory::Memory;
    /// # let mut memory = Memory::new();
    /// memory.write_byte(0x200, 0x1F);
    /// memory.write_byte(0x201, 0x3F);
    /// memory.write_byte(0x202, 0xF3);
    /// let result = memory.read_bytes(0x200, 3);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), &[0x1F, 0x3F, 0xF3]);
    /// let result = memory.read_bytes(0x200, 1);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), &[0x1F]);
    /// ```
    pub fn read_bytes(&self, address: Address, size_bytes: usize) -> OktoResult<&[u8]> {
        let start = address as usize;
        let end = start + size_bytes;
        if start >= MEMORY_SIZE_BYTES || end >= MEMORY_SIZE_BYTES {
            return Err(OktoError::new(OktoErrorKind::AddressOutOfRange));
        }

        Ok(&self.data[start..end])
    }

    /// Reads an instruction from the given address. Returns the reconstructed
    /// instruction on success, or None if the address is too high to read two
    /// bytes from memory (ex. 0xFFF).
    ///
    /// # Examples
    ///
    /// It reads values in big-endian order from memory:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::memory::Memory;
    /// # let mut memory = Memory::new();
    /// memory.write_byte(0x200, 0x3F).unwrap();
    /// memory.write_byte(0x201, 0x24).unwrap();
    /// assert_eq!(Some(0x3F24), memory.read_instruction(0x200));
    /// ```
    ///
    /// If the address is too high to read two bytes from, it returns none:
    ///
    /// ```
    /// # extern crate okto;
    /// # use okto::memory::Memory;
    /// # let mut memory = Memory::new();
    /// memory.write_byte(0xFFF, 0x3F).unwrap();
    /// assert_eq!(None, memory.read_instruction(0xFFF));
    /// ```
    pub fn read_instruction(&self, address: Address) -> Option<Instruction> {
        // If we can't read two bytes, then we cannot read an instruction
        if address >= (MEMORY_SIZE_BYTES - 1) as Address {
            return None;
        }

        let result = bytes_to_word(
            &self.data[address as usize],
            &self.data[(address + 1) as usize],
        );

        Some(result)
    }
}
