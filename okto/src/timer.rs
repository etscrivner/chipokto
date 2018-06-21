//! Chip8 timer types and traits

/// All Chip8 timers are 8-bit registers.
pub type Timer = u8;

/// Number of times that a timer should tick per second.
pub const TIMER_TICK_HZ: u32 = 60;

/// Delay timer register for Chip8
pub struct DelayTimer {
    /// The current value of the delay timer.
    pub value: Timer,
}

/// Implementation of delay timer functionality.
impl DelayTimer {
    /// Initialize a new timer in a non-countdown state.
    pub fn new() -> Self {
        Self { value: 0 }
    }
}

/// Trait implemented by all countdown timer systems.
pub trait CountdownTimer {
    /// Count down exactly one time. All Chip8 timers should call tick at a
    /// rate of 60Hz.
    fn tick(&mut self);
}

/// Implementation of countdown timer behavior for delay timer.
impl CountdownTimer for DelayTimer {
    /// Decrement the delay counter by one if it is greater than 0.
    fn tick(&mut self) {
        if self.value > 0 {
            self.value -= 1;
        }
    }
}
