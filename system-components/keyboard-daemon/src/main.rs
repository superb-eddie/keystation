use std::thread::JoinHandle;

use crossbeam::channel::Sender;

use crate::midi_sender::{MidiEvent, start_midi_sender};
use crate::user_interface::{start_user_interface, UIEvent};

#[cfg(not(feature = "simulator"))]
mod gpio_driver;
#[cfg(not(feature = "simulator"))]
mod keybed_driver;
mod midi_sender;
mod user_interface;

#[cfg(not(feature = "simulator"))]
fn start_input_drivers(
    thread_handles: &mut Vec<JoinHandle<()>>,
    midi_channel: Sender<MidiEvent>,
    ui_channel: Sender<UIEvent>,
) -> anyhow::Result<()> {
    use crate::gpio_driver::start_gpio_driver;
    use crate::keybed_driver::start_keybed_driver;

    let gpio_thread = start_gpio_driver(midi_channel.clone(), ui_channel.clone())?;
    thread_handles.push(gpio_thread);

    let keybed_thread = start_keybed_driver(midi_channel.clone())?;
    thread_handles.push(keybed_thread);

    Ok(())
}

#[cfg(feature = "simulator")]
fn start_input_drivers(
    thread_handles: &mut Vec<JoinHandle<()>>,
    midi_channel: Sender<MidiEvent>,
    ui_channel: Sender<UIEvent>,
) -> anyhow::Result<()> {
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut thread_handles = vec![];

    let (ui_thread, ui_channel) = start_user_interface()?;
    thread_handles.push(ui_thread);

    let (midi_thread, midi_channel) = start_midi_sender()?;
    thread_handles.push(midi_thread);

    for thread_handle in thread_handles.drain(..) {
        thread_handle.join().unwrap();
    }

    Ok(())
}
