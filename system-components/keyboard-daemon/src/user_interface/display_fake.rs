use anyhow::anyhow;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_simulator::SimulatorDisplay;

use crate::user_interface::display::Display;

pub fn new_display() -> impl Display {
    FakeDisplay::new()
}

pub struct FakeDisplay(SimulatorDisplay<BinaryColor>);

impl FakeDisplay {
    fn new() -> Self {
        let display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64));

        Self(display)
    }
}

impl DrawTarget for FakeDisplay {
    type Color = BinaryColor;
    type Error = anyhow::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0.draw_iter(pixels).map_err(|e| anyhow!(e))
    }
}

impl Dimensions for FakeDisplay {
    fn bounding_box(&self) -> Rectangle {
        self.0.bounding_box()
    }
}

impl Display for FakeDisplay {
    fn clear_buffer(&mut self) {
        self.0
            .clear(BinaryColor::Off)
            .expect("couldn't clear simulated display")
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
