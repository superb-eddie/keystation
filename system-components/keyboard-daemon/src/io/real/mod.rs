use crate::io::io_impl::dials_driver::start_dials_driver;
use crate::io::io_impl::display::DisplayImpl;
use crate::io::io_impl::gpio_driver::start_gpio_driver;
use crate::io::io_impl::keyboard_driver::start_keyboard_driver;
use crate::midi_sender::MidiEvent;
use crate::user_interface::UIEvent;
use crate::Threads;
use anyhow::Result;
use crossbeam::channel::Sender;

mod keyboard_driver;
mod dials_driver;
mod gpio_driver;
pub(crate) mod display;

pub fn init_io(
    threads: &mut Threads,
    midi_channel: Sender<MidiEvent>,
    ui_channel: Sender<UIEvent>
) -> Result<impl crate::io::IO<DisplayImpl>> {

    threads.push(start_gpio_driver(midi_channel.clone(), ui_channel.clone())?);
    threads.push(start_dials_driver(midi_channel.clone())?);
    threads.push(start_keyboard_driver(midi_channel.clone())?);

    Ok(IO {
        display: DisplayImpl::new()
    })
}

pub struct IO {
    display: DisplayImpl
}

impl crate::io::IO<DisplayImpl> for IO {
    fn get_display(&mut self) -> &mut DisplayImpl {
        &mut self.display
    }
}