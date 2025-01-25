use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};

use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use rppal::i2c::I2c;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use tinybmp::Bmp;

const SCREEN_WIDTH: f32 = 128.0;
const SCREEN_HEIGHT: f32 = 64.0;

const KEYSTATION_SCROLL_BMP: &[u8] = include_bytes!("../assets/keystation_scroll.bmp");

pub fn start_user_interface() -> anyhow::Result<JoinHandle<()>> {
    let i2c = I2c::new()?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().expect("Couldn't init display");

    let keystation_scroll = Bmp::from_slice(KEYSTATION_SCROLL_BMP).expect("couldn't load scroll");

    let thread_handle = thread::spawn(move || {
        do_logo_scroll(&mut display, keystation_scroll);
        do_ui(&mut display);
    });

    Ok(thread_handle)
}

fn do_ui<DI: WriteOnlyDataCommand, SIZE: DisplaySize>(
    display: &mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
) {
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(3)
        .build();

    let rectangle = Rectangle::new(
        Point::new(5, 5),
        Size::new(SCREEN_WIDTH as u32 - 10, SCREEN_HEIGHT as u32 - 10),
    )
    .into_styled(style);

    frame_loop(10f32, |t| {
        display.clear_buffer();

        rectangle.draw(display).unwrap();

        display.flush().unwrap();

        return true;
    });
}

fn do_logo_scroll<DI: WriteOnlyDataCommand, SIZE: DisplaySize>(
    display: &mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
    logo: Bmp<BinaryColor>,
) {
    let fps = 30f32;

    let animation_duration = 4f32;

    let x_start = SCREEN_WIDTH;
    let x_end = -(SCREEN_WIDTH + logo.size().width as f32);

    frame_loop(fps, |t| {
        display.clear_buffer();

        let logo_pos = Point::new(
            lerp(cycle_linear(t, secs(animation_duration)), x_start, x_end) as i32,
            0,
        );

        Image::new(&logo, logo_pos).draw(display).unwrap();

        display.flush().unwrap();

        return t < animation_duration;
    });
}
fn frame_loop(fps: f32, mut body: impl FnMut(f32) -> bool) {
    let frame_time = Duration::from_secs_f32(1.0 / fps);

    let start_time = Instant::now();
    let mut frame_start_time = Instant::now();
    loop {
        if !body(frame_start_time.duration_since(start_time).as_secs_f32()) {
            break;
        }
        sleep(frame_time.saturating_sub(Instant::now().duration_since(frame_start_time)));
        frame_start_time = Instant::now();
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
