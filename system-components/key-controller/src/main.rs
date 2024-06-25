use std::thread::sleep;
use std::time::{Duration, Instant};

use embedded_graphics::image::Image;
use embedded_graphics::prelude::*;
use rppal::i2c::I2c;
use ssd1306::{I2CDisplayInterface, prelude::*, Ssd1306};
use tinybmp::Bmp;

const SCREEN_WIDTH: f32 = 128.0;
const SCREEN_HEIGHT: f32 = 64.0;

const KEYSTATION_SCROLL_BMP: &[u8] = include_bytes!("../assets/keystation_scroll.bmp");

fn frame_loop(fps: f32, mut body: impl FnMut(f32)) {
    let frame_time = Duration::from_secs_f32(1.0 / fps);

    let start_time = Instant::now();
    loop {
        let frame_start_time = Instant::now();

        body(frame_start_time.duration_since(start_time).as_secs_f32());
        sleep(frame_time.saturating_sub(Instant::now().duration_since(frame_start_time)))
    }
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    assert!(t <= 1.0);
    assert!(0.0 <= t);

    return a + t * (b - a);
}

// returns a number that goes from 0.0 to 1.0 in duration
fn cycle_linear(t: f32, duration: impl Into<Duration>) -> f32 {
    let duration_secs = duration.into().as_secs_f32();

    return (t % duration_secs) / duration_secs;
}

fn secs(t: f32) -> Duration {
    return Duration::from_secs_f32(t);
}

fn main() {
    let i2c = I2c::new().expect("Could not i2c");

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let keystation_scroll = Bmp::from_slice(KEYSTATION_SCROLL_BMP).unwrap();

    let fps = 10f32;
    frame_loop(fps, |t| {
        display.clear_buffer();

        Image::new(
            &keystation_scroll,
            Point::new(
                lerp(
                    cycle_linear(t, secs(4.0)),
                    SCREEN_WIDTH,
                    -(SCREEN_WIDTH + keystation_scroll.size().width as f32),
                ) as i32,
                0,
            ),
        )
        .draw(&mut display)
        .unwrap();

        display.flush().unwrap();
    });
}
