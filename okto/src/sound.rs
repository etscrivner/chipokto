//! Represents the sound subsystem of the Chip8
use super::timer;

/// Number of times that the sound timer should tick per second
pub const SOUND_TIMER_TICK_HZ: u32 = 60;

/// Sound subsystem state data.
pub struct Sound {
    /// Sound timer
    pub timer: timer::Timer,
}

/// Implementation of sound subsystem
impl Sound {
    /// Initialize a new sound system in a non-playing state.
    pub fn new() -> Self {
        Self { timer: 0 }
    }
}

/// Implementation of countdown timer behavior for sound system.
impl timer::CountdownTimer for Sound {
    /// Decrement the sound counter by 1 if it is greater than 0.
    fn tick(&mut self) -> () {
        if self.timer > 0 {
            self.timer -= 1;
        }
    }
}
