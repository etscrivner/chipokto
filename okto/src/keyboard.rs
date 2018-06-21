//! Represents keyboard types, traits, and data structures

/// The number of keys available on the Chip8 keyboard.
pub const NUM_KEYS: usize = 16;

/// Type return by keyboard wait key callback
pub type WaitKeyResult<T> = Result<T, String>;

/// Enumeration of the states of keys in Chip8.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyState {
    Released,
    Pressed
}

/// Data structure representing the state of the Chip8 keyboard.
pub struct Keyboard<F>
where F: FnMut() -> WaitKeyResult<u8>
{
    /// State of the keys on the keyboard.
    pub keys: [KeyState; NUM_KEYS],
    
    // TODO: Reimplement the callback fn pointer as a trait object??
    
    /// Callback method invoked when wait key instruction occurs.
    pub wait_key_callback: Box<F>
}

/// Implementation of keyboard interfaces
impl<F> Keyboard<F>
where F: FnMut() -> WaitKeyResult<u8>
{
    /// Initialize a new keyboard structure with all keys released. Uses the
    /// nop callback function.
    pub fn new(wait_key_callback: Box<F>) -> Self {
        Self {
            keys: [KeyState::Released; NUM_KEYS],
            wait_key_callback: wait_key_callback
        }
    }
}

/// Null operation (nop) callback for waitkey. Does nothing and returns a
/// constant key value.
pub fn nop_wait_key_callback() -> WaitKeyResult<u8> {
    Ok(0x00)
}
