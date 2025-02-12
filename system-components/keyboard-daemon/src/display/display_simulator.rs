use std::process::exit;

use anyhow::anyhow;
use crossbeam::channel::Sender;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;
use embedded_graphics_simulator::sdl2::Keycode;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use midly::num::u7;

use crate::display::Display;
use crate::midi_sender::MidiEvent;
use crate::user_interface::{Button, UIEvent};

pub fn new_display() -> impl Display {
    FakeDisplay::new()
}

pub struct FakeDisplay {
    display: SimulatorDisplay<BinaryColor>,
    window: Window,

    midi_channel: Option<Sender<MidiEvent>>,
    ui_channel: Option<Sender<UIEvent>>,
}

impl FakeDisplay {
    fn new() -> Self {
        let display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64));

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::OledWhite)
            .build();
        let window = Window::new("Keystation Sim", &output_settings);

        Self {
            display,
            window,
            midi_channel: None,
            ui_channel: None,
        }
    }
}

impl DrawTarget for FakeDisplay {
    type Color = BinaryColor;
    type Error = anyhow::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels).map_err(|e| anyhow!(e))
    }
}

impl Dimensions for FakeDisplay {
    fn bounding_box(&self) -> Rectangle {
        self.display.bounding_box()
    }
}

impl Display for FakeDisplay {
    fn clear_buffer(&mut self) {
        self.display
            .clear(BinaryColor::Off)
            .expect("couldn't clear simulated display")
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.window.update(&self.display);

        // TODO: Goddam fucking borrow checker ruining my life
        let midi_channel = self.midi_channel.clone();
        let ui_channel = self.ui_channel.clone();

        for event in self.window.events() {
            match event {
                SimulatorEvent::Quit => exit(0),
                SimulatorEvent::KeyUp {
                    keycode, repeat, ..
                } => {
                    if repeat {
                        continue;
                    }

                    if let (Some(midi), Some(ui)) = (&midi_channel, &ui_channel) {
                        send_input_event(midi, ui, keycode, false)?
                    }
                }
                SimulatorEvent::KeyDown {
                    keycode, repeat, ..
                } => {
                    if repeat {
                        continue;
                    }

                    if let (Some(midi), Some(ui)) = (&midi_channel, &ui_channel) {
                        send_input_event(midi, ui, keycode, true)?
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn start_input_drivers(
        &mut self,
        midi_channel: Sender<MidiEvent>,
        ui_channel: Sender<UIEvent>,
    ) -> anyhow::Result<()> {
        self.midi_channel = Some(midi_channel);
        self.ui_channel = Some(ui_channel);

        Ok(())
    }
}

fn send_input_event(
    midi_channel: &Sender<MidiEvent>,
    ui_channel: &Sender<UIEvent>,
    keycode: Keycode,
    down: bool,
) -> anyhow::Result<()> {
    match keycode {
        // Display keys
        Keycode::Slash => ui_event(ui_channel, down, Button::DpadDown),
        Keycode::P => ui_event(ui_channel, down, Button::DpadUp),
        Keycode::L => ui_event(ui_channel, down, Button::DpadLeft),
        Keycode::Quote => ui_event(ui_channel, down, Button::DpadRight),
        Keycode::Semicolon => ui_event(ui_channel, down, Button::DpadCenter),
        Keycode::O => ui_event(ui_channel, down, Button::A),
        Keycode::LeftBracket => ui_event(ui_channel, down, Button::B),

        // Keyboard keys
        Keycode::A => midi_key_event(midi_channel, down, 48),
        Keycode::W => midi_key_event(midi_channel, down, 49),
        Keycode::S => midi_key_event(midi_channel, down, 50),
        Keycode::E => midi_key_event(midi_channel, down, 51),
        Keycode::D => midi_key_event(midi_channel, down, 52),
        Keycode::F => midi_key_event(midi_channel, down, 53),
        Keycode::T => midi_key_event(midi_channel, down, 54),
        Keycode::G => midi_key_event(midi_channel, down, 55),
        Keycode::Y => midi_key_event(midi_channel, down, 56),
        Keycode::H => midi_key_event(midi_channel, down, 57),

        Keycode::Space => midi_sustain_event(midi_channel, down),

        _ => Ok(()),
    }
}

fn ui_event(ui_channel: &Sender<UIEvent>, down: bool, button: Button) -> anyhow::Result<()> {
    ui_channel.send(if down {
        UIEvent::Down(button)
    } else {
        UIEvent::Up(button)
    })?;

    Ok(())
}

fn midi_key_event(midi_channel: &Sender<MidiEvent>, down: bool, pitch: u8) -> anyhow::Result<()> {
    midi_channel.send(if down {
        MidiEvent::NoteOn {
            key: u7::new(pitch),
            vel: u7::new(127 / 2),
        }
    } else {
        MidiEvent::NoteOff {
            key: u7::new(pitch),
            vel: Default::default(),
        }
    })?;

    Ok(())
}

fn midi_sustain_event(midi_channel: &Sender<MidiEvent>, down: bool) -> anyhow::Result<()> {
    midi_channel.send(if down {
        MidiEvent::Controller {
            controller: u7::new(0x40),
            value: u7::max_value(),
        }
    } else {
        MidiEvent::Controller {
            controller: u7::new(0x40),
            value: u7::default(),
        }
    })?;

    Ok(())
}
