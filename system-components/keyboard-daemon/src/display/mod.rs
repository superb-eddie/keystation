#[cfg(feature = "simulator")]
use crossbeam::channel::Sender;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

pub use display_impl::*;

#[cfg(feature = "simulator")]
use crate::midi_sender::MidiEvent;
#[cfg(feature = "simulator")]
use crate::user_interface::UIEvent;

#[cfg_attr(feature = "keyboard", path = "display_keyboard.rs")]
#[cfg_attr(feature = "simulator", path = "display_simulator.rs")]
mod display_impl;

pub trait Display: DrawTarget<Color = BinaryColor, Error = anyhow::Error> {
    fn clear_buffer(&mut self);
    fn flush(&mut self) -> anyhow::Result<()>;

    #[cfg(feature = "simulator")]
    fn start_input_drivers(
        &mut self,
        midi_channel: Sender<MidiEvent>,
        ui_channel: Sender<UIEvent>,
    ) -> anyhow::Result<()>;
}
