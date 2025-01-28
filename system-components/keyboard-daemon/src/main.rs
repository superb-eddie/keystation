#[cfg(all(feature = "keyboard", feature = "simulator"))]
compile_error!("can't build for keyboard and simulator at the same time");

use crossbeam::channel::{unbounded, Sender};
use std::thread;
use std::time::Duration;

use crate::midi_sender::{start_midi_sink, MidiEvent};
use crate::user_interface::Button::DpadCenter;
use crate::user_interface::{start_user_interface, UIEvent};

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

#[cfg(feature = "simulator")]
fn start_input_drivers(
    midi_channel: Sender<MidiEvent>,
    ui_channel: Sender<UIEvent>,
) -> anyhow::Result<()> {
    thread::spawn(move || loop {
        ui_channel.send(UIEvent::Down(DpadCenter)).unwrap();
        thread::sleep(Duration::from_secs_f32(0.5));
        ui_channel.send(UIEvent::Up(DpadCenter)).unwrap();
        thread::sleep(Duration::from_secs_f32(0.5));
    });

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let (midi_sender, midi_receiver) = unbounded();
    let (ui_sender, ui_receiver) = unbounded();

    start_midi_sink(midi_receiver)?;

    start_input_drivers(midi_sender, ui_sender)?;

    start_user_interface(ui_receiver);
}
