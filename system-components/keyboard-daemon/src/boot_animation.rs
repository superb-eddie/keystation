use crate::display::Display;
use embedded_graphics::image::Image;
use embedded_graphics::prelude::*;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tinybmp::Bmp;

const KEYSTATION_SCROLL_BMP: &[u8] = include_bytes!("../assets/keystation_scroll.bmp");

pub fn do_logo_scroll<D: Display>(display: &mut D) {
    let logo = Bmp::from_slice(KEYSTATION_SCROLL_BMP).expect("couldn't load scroll");

    let fps = 10f32;

    let animation_duration = 4f32;

    let display_size = display.bounding_box().size;

    let x_start = display_size.width as f32;
    let x_end = -((display_size.width + logo.size().width) as f32);

    // TODO: Simplify this using a ticker
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

    a + t * (b - a)
}

// returns a number that goes from 0.0 to 1.0 in duration
fn cycle_linear(t: f32, duration: impl Into<Duration>) -> f32 {
    let duration_secs = duration.into().as_secs_f32();

    (t % duration_secs) / duration_secs
}

fn secs(t: f32) -> Duration {
    Duration::from_secs_f32(t)
}
