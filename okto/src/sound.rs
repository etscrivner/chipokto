//! Represents the sound subsystem of the Chip8
use super::timer;

use std::sync::{Arc, RwLock};

/// Number of times that the sound timer should tick per second
pub const SOUND_TIMER_TICK_HZ: u32 = 60;

/// Sound subsystem state data.
pub struct Sound {
    /// Sound timer
    pub timer: Arc<RwLock<timer::Timer>>,
}

/// Implementation of sound subsystem
impl Sound {
    /// Initialize a new sound system in a non-playing state.
    pub fn new() -> Self {
        Self {
            timer: Arc::new(RwLock::new(0)),
        }
    }
}

/// Implementation of countdown timer behavior for sound system.
impl timer::CountdownTimer for Sound {
    /// Decrement the sound counter by 1 if it is greater than 0.
    fn tick(&mut self) -> () {
        let mut timer = self.timer.write().unwrap();
        if *timer > 0 {
            *timer -= 1;
        }
    }
}
