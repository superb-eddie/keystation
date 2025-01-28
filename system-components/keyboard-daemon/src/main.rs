#[cfg(all(feature = "keyboard", feature = "simulator"))]
compile_error!("can't build for keyboard and simulator at the same time");

use crate::display::new_display;
use crate::display::Display;
use crate::midi_sender::start_midi_sink;
use crate::user_interface::start_user_interface;
use crossbeam::channel::unbounded;

#[cfg(feature = "keyboard")]
use crate::midi_sender::MidiEvent;
#[cfg(feature = "keyboard")]
use crate::user_interface::UIEvent;
#[cfg(feature = "keyboard")]
use crossbeam::channel::Sender;

mod display;
#[cfg(feature = "keyboard")]
mod gpio_driver;
#[cfg(feature = "keyboard")]
mod keybed_driver;
mod midi_sender;
mod user_interface;

#[cfg(feature = "keyboard")]
fn start_input_drivers(
    midi_channel: Sender<MidiEvent>,
    ui_channel: Sender<UIEvent>,
) -> anyhow::Result<()> {
    use crate::gpio_driver::start_gpio_driver;
    use crate::keybed_driver::start_keybed_driver;

    start_gpio_driver(midi_channel.clone(), ui_channel.clone())?;

    start_keybed_driver(midi_channel.clone())?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let (midi_sender, midi_receiver) = unbounded();
    let (ui_sender, ui_receiver) = unbounded();

    start_midi_sink(midi_receiver)?;

    #[cfg(feature = "keyboard")]
    {
        start_input_drivers(midi_sender.clone(), ui_sender.clone())?;
    }

    let mut display = new_display();
    println!("Display initialized");

    #[cfg(feature = "simulator")]
    {
        display.start_input_drivers(midi_sender.clone(), ui_sender.clone())?;
    }

    start_user_interface(display, ui_receiver);
}
