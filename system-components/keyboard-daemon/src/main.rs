use crate::gpio_driver::start_gpio_driver;
use crate::keybed_driver::start_keybed_driver;
use crate::midi_sender::start_midi_sender;
use crate::user_interface::start_user_interface;

mod gpio_driver;
mod keybed_driver;
mod midi_sender;
mod user_interface;

fn main() -> anyhow::Result<()> {
    let mut thread_handles = vec![];

    let ui_thread = start_user_interface()?;
    thread_handles.push(ui_thread);

    let (midi_thread, midi_channel) = start_midi_sender()?;
    thread_handles.push(midi_thread);

    let gpio_thread = start_gpio_driver(midi_channel.clone())?;
    thread_handles.push(gpio_thread);

    let keybed_thread = start_keybed_driver(midi_channel.clone())?;
    thread_handles.push(keybed_thread);

    for thread_handle in thread_handles.drain(..) {
        thread_handle.join().unwrap();
    }

    Ok(())
}
