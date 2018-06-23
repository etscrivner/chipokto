extern crate clap;
extern crate okto;
extern crate sdl2;

use std::io;

use clap::{App, Arg};

use okto::keyboard::WaitKeyResult;
use okto::machine::Machine;
use okto::timer::CountdownTimer;

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
    b: 255,
    g: 255,
    a: 255,
};

// TODO:
// - [X] Countdown timers
// - [X] Sound timers
// - [ ] Display updating
// - [ ] Sound

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
    fn new(wait_key_callback: F) -> Self {
        Self {
            machine: Machine::new(Box::new(wait_key_callback)),
            delta_last_tick_milliseconds: 0,
        }
    }

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

    fn update(&mut self, delta_time_milliseconds: u32) {
        self.delta_last_tick_milliseconds += delta_time_milliseconds;

        if self.delta_last_tick_milliseconds >= MILLISECONDS_PER_TICK {
            self.machine.sound.tick();
            self.machine.delay_timer.tick();
            self.delta_last_tick_milliseconds = 0;
        }
    }
}

fn main() -> io::Result<()> {
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
                    Event::KeyDown { .. } => {
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

    // Main loop
    'running: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        emulator_app.machine.step().unwrap();
        // Process events
        // emulator_app.process_events();

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
