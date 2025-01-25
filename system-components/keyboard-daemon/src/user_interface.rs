use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};

use crossbeam::channel::{Receiver, select_biased, Sender, tick, unbounded};
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable, Triangle,
};
use rppal::i2c::I2c;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use tinybmp::Bmp;

const SCREEN_WIDTH: f32 = 128.0;
const SCREEN_HEIGHT: f32 = 64.0;

const KEYSTATION_SCROLL_BMP: &[u8] = include_bytes!("../assets/keystation_scroll.bmp");

pub enum Button {
    DpadUp,
    DpadDown,
    DpadRight,
    DpadLeft,
    DpadCenter,
    A,
    B,
    AdvancedFunctions,
    OctaveMinus,
    OctavePlus,
}

pub enum UIEvent {
    Down(Button),
    Up(Button),
}

pub fn start_user_interface() -> anyhow::Result<(JoinHandle<()>, Sender<UIEvent>)> {
    let i2c = I2c::new()?;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().expect("Couldn't init display");

    let keystation_scroll = Bmp::from_slice(KEYSTATION_SCROLL_BMP).expect("couldn't load scroll");

    let (s, r) = unbounded();

    let thread_handle = thread::spawn(move || {
        do_logo_scroll(&mut display, keystation_scroll);
        do_ui(&mut display, r);
    });

    Ok((thread_handle, s))
}

struct UIState {
    button_u_down: bool,
    button_d_down: bool,
    button_r_down: bool,
    button_l_down: bool,
    button_c_down: bool,
    button_a_down: bool,
    button_b_down: bool,
}

impl UIState {
    fn new() -> Self {
        Self {
            button_u_down: false,
            button_d_down: false,
            button_r_down: false,
            button_l_down: false,
            button_c_down: false,
            button_a_down: false,
            button_b_down: false,
        }
    }

    fn process_event(&mut self, event: UIEvent) {
        match event {
            UIEvent::Down(b) => match b {
                Button::DpadUp => self.button_u_down = true,
                Button::DpadDown => self.button_d_down = true,
                Button::DpadRight => self.button_r_down = true,
                Button::DpadLeft => self.button_l_down = true,
                Button::DpadCenter => self.button_c_down = true,
                Button::A => self.button_a_down = true,
                Button::B => self.button_b_down = true,
                _ => {}
            },
            UIEvent::Up(b) => match b {
                Button::DpadUp => self.button_u_down = false,
                Button::DpadDown => self.button_d_down = false,
                Button::DpadRight => self.button_r_down = false,
                Button::DpadLeft => self.button_l_down = false,
                Button::DpadCenter => self.button_c_down = false,
                Button::A => self.button_a_down = false,
                Button::B => self.button_b_down = false,
                _ => {}
            },
        }
    }
}

fn ui_renderer<'a, DI: WriteOnlyDataCommand, SIZE: DisplaySize>(
) -> impl FnMut(&mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>, &UIState) + 'a {
    let style_outline = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .build();

    let style_filled = PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::On)
        .build();

    let button_u = Triangle::new(Point::new(20, 20), Point::new(30, 2), Point::new(40, 20));
    let button_l = Triangle::new(Point::new(0, 30), Point::new(18, 21), Point::new(18, 41));
    let button_r = Triangle::new(Point::new(60, 30), Point::new(42, 21), Point::new(42, 41));
    let button_d = Triangle::new(Point::new(30, 60), Point::new(40, 42), Point::new(20, 42));

    let button_c = Rectangle::new(Point::new(20, 22), Size::new(20, 18));
    let button_a = Circle::new(Point::new(70, 40), 20);
    let button_b = Circle::new(Point::new(100, 20), 20);

    move |display, state| {
        let style = |filled: bool| -> &PrimitiveStyle<BinaryColor> {
            if filled {
                &style_filled
            } else {
                &style_outline
            }
        };

        display.clear_buffer();

        button_u
            .draw_styled(style(state.button_u_down), display)
            .unwrap();
        button_l
            .draw_styled(style(state.button_l_down), display)
            .unwrap();
        button_r
            .draw_styled(style(state.button_r_down), display)
            .unwrap();
        button_d
            .draw_styled(style(state.button_d_down), display)
            .unwrap();
        button_c
            .draw_styled(style(state.button_c_down), display)
            .unwrap();
        button_a
            .draw_styled(style(state.button_a_down), display)
            .unwrap();
        button_b
            .draw_styled(style(state.button_b_down), display)
            .unwrap();

        display.flush().unwrap();
    }
}

fn do_ui<DI: WriteOnlyDataCommand, SIZE: DisplaySize>(
    display: &mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
    event_channel: Receiver<UIEvent>,
) {
    let mut state = UIState::new();

    let mut render = ui_renderer();

    let frame_tick = tick(secs(1.0 / 10.0));
    loop {
        select_biased! {
            recv(frame_tick) -> _ => render(display, &state),
            recv(event_channel) -> e => state.process_event(e.unwrap()),
        }
    }
}

fn do_logo_scroll<DI: WriteOnlyDataCommand, SIZE: DisplaySize>(
    display: &mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
    logo: Bmp<BinaryColor>,
) {
    let fps = 10f32;

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
