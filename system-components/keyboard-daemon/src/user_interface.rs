use std::time::Duration;

use crossbeam::channel::{select_biased, tick, Receiver};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable, Triangle,
};

use crate::display::Display;

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

pub fn do_ui<D: Display>(
    display: &mut D,
    event_channel: Receiver<UIEvent>,
    mut frame_hook: impl FnMut(),
) -> ! {
    let mut state = UIState::new();

    let mut render = ui_renderer();

    let frame_tick = tick(Duration::from_secs_f32(1.0 / 10.0));
    loop {
        select_biased! {
            recv(frame_tick) -> _ => {
                render(display, &state);
                frame_hook()
            },
            recv(event_channel) -> e => state.process_event(e.unwrap()),
        }
    }
}

fn ui_renderer<'a, D: Display>() -> impl FnMut(&mut D, &UIState) + 'a {
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
