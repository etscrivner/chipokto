extern crate clap;
extern crate okto;
extern crate sdl2;

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};

use clap::{App, Arg};

use okto::keyboard::WaitKeyResult;
use okto::machine::Machine;
use okto::timer::{CountdownTimer, Timer};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

/// Number of milliseconds per Chip8 timer tick ~(1000 * 1 / 60).
const MILLISECONDS_PER_TICK: u32 = 17;
/// Background color used for drawing the absence of a pixel.
const BACKGROUND_COLOR: Color = Color {
    r: 0,
    b: 0,
    g: 0,
    a: 255,
};
/// Foreground color used for drawing the presence of a pixel.
const FOREGROUND_COLOR: Color = Color {
    r: 255,
    b: 192,
    g: 180,
    a: 255,
};

/// Data structure that will help us in generating a square sound wave whenever
/// the audio delay timer is non-zero.
struct SoundWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
    sound_timer: Arc<RwLock<Timer>>,
}

/// Implementation of SDL audio callback for our `SoundWave` type.
impl AudioCallback for SoundWave {
    /// Channel content type
    type Channel = f32;

    /// Generate a square audio wave whenever the sound timer is non-zero.
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            let timer = self.sound_timer.read().unwrap();
            if *timer > 0 {
                *x = if self.phase <= 0.5 {
                    self.volume
                } else {
                    -self.volume
                };
                self.phase = (self.phase + self.phase_inc) % 1.0;
            } else {
                *x = 0.0;
            }
        }
    }
}

/// Generalized data type store the emulator application itself.
struct EmulatorApp<F>
where
    F: FnMut() -> WaitKeyResult<u8>,
{
    machine: Machine<F>,
    delta_last_tick_milliseconds: u32,
}

impl<F> EmulatorApp<F>
where
    F: FnMut() -> WaitKeyResult<u8>,
{
    /// Default constructor.
    fn new(wait_key_callback: F) -> Self {
        Self {
            machine: Machine::new(Box::new(wait_key_callback)),
            delta_last_tick_milliseconds: 0,
        }
    }

    /// Execute the next instruction on the emulator.
    fn step(&mut self) {
        self.machine.step().unwrap();
    }

    /// Draw the contents of the framebuffer to the given canvas.
    fn draw(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(BACKGROUND_COLOR);
        canvas.clear();

        // Compute the sizes needed to properly render the frame buffer.
        let (window_width, window_height) = canvas.window().size();
        let (display_width, display_height) = (
            self.machine.display.width() as u32,
            self.machine.display.height() as u32,
        );
        let (rect_width, rect_height) =
            (window_width / display_width, window_height / display_height);

        // Draw an appropriately sized rectangle per pixel that is on in the frame buffer.
        canvas.set_draw_color(FOREGROUND_COLOR);
        for height in 0..display_height {
            for width in 0..display_width {
                if self.machine.display.data[height as usize][width as usize] == 1 {
                    canvas
                        .box_(
                            (width * rect_width) as i16,
                            (height * rect_height) as i16,
                            ((width * rect_width) + rect_width) as i16,
                            ((height * rect_height) + rect_height) as i16,
                            FOREGROUND_COLOR,
                        )
                        .unwrap();
                }
            }
        }

        canvas.present();
    }

    /// Update all time-dependent components of the machine.
    fn update(&mut self, delta_time_milliseconds: u32) {
        self.delta_last_tick_milliseconds += delta_time_milliseconds;

        if self.delta_last_tick_milliseconds >= MILLISECONDS_PER_TICK {
            self.machine.sound.tick();
            self.machine.delay_timer.tick();
            self.delta_last_tick_milliseconds = 0;
        }
    }

    /// Update state to indiate that a key was pressed.
    fn key_pressed(&mut self, key: u8) {
        self.machine.keyboard.keys[key as usize] = okto::keyboard::KeyState::Pressed;
    }

    /// Update state to indicate that a key was released.
    fn key_released(&mut self, key: u8) {
        self.machine.keyboard.keys[key as usize] = okto::keyboard::KeyState::Released;
    }
}

fn main() -> io::Result<()> {
    // Build key map
    let mut keymap: HashMap<sdl2::keyboard::Keycode, u8> = HashMap::new();
    keymap.insert(sdl2::keyboard::Keycode::Num1, 0x01);
    keymap.insert(sdl2::keyboard::Keycode::Num2, 0x02);
    keymap.insert(sdl2::keyboard::Keycode::Num3, 0x03);
    keymap.insert(sdl2::keyboard::Keycode::Num4, 0x0C);
    keymap.insert(sdl2::keyboard::Keycode::Q, 0x04);
    keymap.insert(sdl2::keyboard::Keycode::W, 0x05);
    keymap.insert(sdl2::keyboard::Keycode::E, 0x06);
    keymap.insert(sdl2::keyboard::Keycode::R, 0x0D);
    keymap.insert(sdl2::keyboard::Keycode::A, 0x07);
    keymap.insert(sdl2::keyboard::Keycode::S, 0x08);
    keymap.insert(sdl2::keyboard::Keycode::D, 0x09);
    keymap.insert(sdl2::keyboard::Keycode::F, 0x0E);
    keymap.insert(sdl2::keyboard::Keycode::Z, 0x0A);
    keymap.insert(sdl2::keyboard::Keycode::X, 0x00);
    keymap.insert(sdl2::keyboard::Keycode::C, 0x0B);
    keymap.insert(sdl2::keyboard::Keycode::V, 0x0F);

    // Parse required command-line arguments
    let matches = App::new("chipokto")
        .version("1.0")
        .author("Eric Scrivner <eric.t.scrivner@gmail.com>")
        .about("Run Chip8 game in a virtual machine")
        .arg(
            Arg::with_name("ROMFILE")
                .help("Path to Chip8 ROM file.")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Load ROM file
    let rom_path = matches.value_of("ROMFILE").unwrap();
    let rom_data = okto::read_rom_file(rom_path)?;

    // Initialize SDL2 for rendering, input, and audio.
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("chipokto", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    // Initialize the Okto emulator
    let wait_key_callback = || -> WaitKeyResult<u8> {
        loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if keymap.contains_key(&keycode) {
                            return Ok(keymap[&keycode]);
                        }
                        return Ok(0x00);
                    }
                    _ => {}
                }
            }
        }
    };

    let mut emulator_app = EmulatorApp::new(wait_key_callback);
    emulator_app
        .machine
        .memory
        .load(&rom_data, okto::cpu::DEFAULT_PC_ADDRESS, rom_data.len())
        .unwrap();

    let mut last_update_time = sdl_context.timer().unwrap().ticks();

    // Audio system
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| SoundWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
            sound_timer: emulator_app.machine.sound.timer.clone(),
        })
        .unwrap();

    device.resume();

    // Main loop
    'running: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keymap.contains_key(&keycode) {
                        emulator_app.key_pressed(keymap[&keycode]);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keymap.contains_key(&keycode) {
                        emulator_app.key_released(keymap[&keycode]);
                    }
                }
                _ => {}
            }
        }

        emulator_app.step();

        // Draw the contents of the framebuffer to the screen.
        emulator_app.draw(&mut canvas);

        // Update machine subsystems that are time-dependent.
        let current_time = sdl_context.timer().unwrap().ticks();
        let delta_time = current_time - last_update_time;
        emulator_app.update(delta_time);
        last_update_time = current_time;
    }

    Ok(())
}
