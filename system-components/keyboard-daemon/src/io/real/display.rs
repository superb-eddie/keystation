use crate::io::Display;
use anyhow::anyhow;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;
use rppal::i2c::I2c;
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, DisplaySize128x64, I2CInterface};
use ssd1306::{I2CDisplayInterface, Ssd1306};

pub struct DisplayImpl(
    Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
);

impl DisplayImpl {
    pub(crate) fn new() -> Self {
        let i2c = I2c::new().expect("Couldn't init i2c interface");

        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().expect("Couldn't init display");

        Self(display)
    }
}

impl DrawTarget for DisplayImpl {
    type Color = BinaryColor;
    type Error = anyhow::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item=Pixel<Self::Color>>,
    {
        self.0.draw_iter(pixels).map_err(|e| anyhow!("{:?}", e))
    }
}

impl Dimensions for DisplayImpl {
    fn bounding_box(&self) -> Rectangle {
        self.0.bounding_box()
    }
}

impl Display for DisplayImpl {
    fn clear_buffer(&mut self) {
        self.0.clear_buffer()
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.0.flush().map_err(|e| anyhow!("{:?}", e))
    }
}
