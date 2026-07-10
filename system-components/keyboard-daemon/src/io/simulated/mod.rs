use crate::io::io_impl::display::DisplayImpl;
use crate::midi_sender::MidiEvent;
use crate::user_interface::UIEvent;
use crate::Threads;
use anyhow::Result;
use crossbeam::channel::Sender;

mod display;

pub fn init_io(
    threads: &mut Threads,
    midi_channel: Sender<MidiEvent>,
    ui_channel: Sender<UIEvent>
) -> Result<impl crate::io::IO<DisplayImpl>> {

    Ok(IO {
        display: DisplayImpl::new(midi_channel, ui_channel),
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