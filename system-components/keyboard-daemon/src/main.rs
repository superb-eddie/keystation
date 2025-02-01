#[cfg(feature = "keyboard")]
use crossbeam::channel::Sender;
use crossbeam::channel::unbounded;

#[cfg(feature = "simulator")]
use crate::display::Display;
use crate::display::new_display;
#[cfg(feature = "keyboard")]
use crate::midi_sender::MidiEvent;
use crate::midi_sender::start_midi_sink;
use crate::user_interface::start_user_interface;
#[cfg(feature = "keyboard")]
use crate::user_interface::UIEvent;

#[cfg(all(feature = "keyboard", feature = "simulator"))]
compile_error!("can't build for keyboard and simulator at the same time");

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

// TODO: more structured thread management

// TODO: CI for both build targets

fn main() -> anyhow::Result<()> {
    let (midi_sender, midi_receiver) = unbounded();
    let (ui_sender, ui_receiver) = unbounded();

    start_midi_sink(midi_receiver)?;
    println!("Midi initialized");

    let display = {
        #[cfg(feature = "simulator")]
        {
            let mut display = new_display();
            display.start_input_drivers(midi_sender.clone(), ui_sender.clone())?;
            display
        }
        #[cfg(feature = "keyboard")]
        {
            start_input_drivers(midi_sender.clone(), ui_sender.clone())?;
            new_display()
        }
    };
    println!("Input and Display initialized");

    start_user_interface(display, ui_receiver);
}
