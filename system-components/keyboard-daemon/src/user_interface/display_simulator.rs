use anyhow::anyhow;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::process::exit;

use crate::user_interface::display::Display;

pub fn new_display() -> impl Display {
    FakeDisplay::new()
}

pub struct FakeDisplay {
    display: SimulatorDisplay<BinaryColor>,
    window: Window,
}

impl FakeDisplay {
    fn new() -> Self {
        let display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64));

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::OledWhite)
            .build();
        let window = Window::new("Keystation Sim", &output_settings);

        Self { display, window }
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

        self.window.events().for_each(|(event)| match event {
            SimulatorEvent::Quit => exit(0),
            _ => {}
        });

        Ok(())
    }
}
