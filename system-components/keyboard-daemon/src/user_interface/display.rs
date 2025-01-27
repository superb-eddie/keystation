use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

pub trait Display: DrawTarget<Color = BinaryColor, Error = anyhow::Error> {
    fn clear_buffer(&mut self);
    fn flush(&mut self) -> anyhow::Result<()>;
}
