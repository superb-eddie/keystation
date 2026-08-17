use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::BinaryColor;

// TODO: CI for both build targets
#[cfg(all(feature = "keyboard", feature = "simulator"))]
compile_error!("can't build for keyboard and simulator at the same time");

#[cfg_attr(feature = "keyboard", path = "real/mod.rs")]
#[cfg_attr(feature = "simulator", path = "simulated/mod.rs")]
pub mod io_impl;

pub use io_impl::init_io;

pub trait IO<D: Display> {
    fn get_display(&mut self) -> &mut D;
}

pub trait Display: DrawTarget<Color=BinaryColor, Error=anyhow::Error> {
    fn clear_buffer(&mut self);
    fn flush(&mut self) -> anyhow::Result<()>;
}