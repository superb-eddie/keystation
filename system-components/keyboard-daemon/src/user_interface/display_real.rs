use std::fmt::Debug;

use anyhow::anyhow;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use rppal::i2c::I2c;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;

use crate::user_interface::display::Display;

pub fn new_display() -> impl Display {
    RealDisplay::new()
}

pub struct RealDisplay(
    Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
);

impl RealDisplay {
    fn new() -> Self {
        let i2c = I2c::new().expect("Couldn't init i2c interface");

        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().expect("Couldn't init display");

        Self(display)
    }
}

impl DrawTarget for RealDisplay {
    type Color = BinaryColor;
    type Error = anyhow::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.0.draw_iter(pixels).map_err(|e| anyhow!("{:?}", e))
    }
}

impl Dimensions for RealDisplay {
    fn bounding_box(&self) -> Rectangle {
        self.0.bounding_box()
    }
}

impl Display for RealDisplay {
    fn clear_buffer(&mut self) {
        self.0.clear_buffer()
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.0.flush().map_err(|e| anyhow!("{:?}", e))
    }
}
