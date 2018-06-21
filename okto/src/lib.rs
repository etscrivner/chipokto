//! Library for emulating the Chip8 and SuperChip8 virtual machines.
extern crate rand;

pub mod cpu;
pub mod display;
pub mod keyboard;
pub mod machine;
pub mod memory;
pub mod sound;
pub mod timer;

use std::error::Error;
use std::fmt;

/// Generic result type for emulator errors
pub type OktoResult<T> = std::result::Result<T, OktoError>;

/// A general emulator error
#[derive(Debug, PartialEq)]
pub struct OktoError {
    /// The kind of error that occurrred
    pub kind: OktoErrorKind,
}

/// A list specifying general categories of emulator errors.
#[derive(Debug, PartialEq)]
pub enum OktoErrorKind {
    /// CPU stack had too many items pushed onto it.
    StackOverflow,
    /// CPU stack had too many items popped off of it.
    StackUnderflow,
    /// Attempt to load too much data into memory.
    RomTooLarge,
    /// Attempt to access memory out of bounds.
    AddressOutOfRange,
    /// Invalid digit sprite requested
    InvalidDigitSprite,
    /// Invalid opcode given to the interpreter
    InvalidOpcode,
    /// Unknown error along with an error message
    Unknown(String),
}

/// Implementation of the error interface
impl OktoError {
    /// Initialize a new error
    pub fn new(kind: OktoErrorKind) -> Self {
        Self { kind: kind }
    }
}

/// Implementation of error interface for emulator error
impl Error for OktoError {
    /// Returns a description derived from the error kind.
    fn description(&self) -> &str {
        match &self.kind {
            OktoErrorKind::StackOverflow => "Stack overflow",
            OktoErrorKind::StackUnderflow => "Stack underflow",
            OktoErrorKind::RomTooLarge => "ROM too large",
            OktoErrorKind::AddressOutOfRange => "Address out of range",
            OktoErrorKind::InvalidDigitSprite => "Invalid digit sprite",
            OktoErrorKind::InvalidOpcode => "Invalid opcode",
            OktoErrorKind::Unknown(_) => "Unknown",
        }
    }
}

impl fmt::Display for OktoError {
    /// Display the error in a textual format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error('{:?}')", self.description())
    }
}
