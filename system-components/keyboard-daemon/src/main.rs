use crate::boot_animation::do_logo_scroll;
use crate::display::new_display;
#[cfg(feature = "simulator")]
use crate::display::Display;
use crate::midi_sender::start_midi_sink;
#[cfg(feature = "keyboard")]
use crate::midi_sender::MidiEvent;
use crate::user_interface::do_ui;
#[cfg(feature = "keyboard")]
use crate::user_interface::UIEvent;
use anyhow::Result;
use crossbeam::channel::unbounded;
#[cfg(feature = "keyboard")]
use crossbeam::channel::Sender;
use std::panic;
use std::thread::JoinHandle;

#[cfg(all(feature = "keyboard", feature = "simulator"))]
compile_error!("can't build for keyboard and simulator at the same time");

mod boot_animation;
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
    threads: &mut Vec<JoinHandle<Result<()>>>,
) -> Result<()> {
    use crate::gpio_driver::start_gpio_driver;
    use crate::keybed_driver::start_keybed_driver;

    threads.push(start_gpio_driver(midi_channel.clone(), ui_channel.clone())?);

    threads.push(start_keybed_driver(midi_channel.clone())?);

    Ok(())
}

// TODO: CI for both build targets

fn main() -> Result<()> {
    let mut display = new_display();
    println!("Display initialized");

    do_logo_scroll(&mut display);

    let (midi_sender, midi_receiver) = unbounded();
    let (ui_sender, ui_receiver) = unbounded();

    let mut threads = vec![];

    threads.push(start_midi_sink(midi_receiver));
    println!("Midi initialized");

    #[cfg(feature = "simulator")]
    {
        display.start_input_drivers(midi_sender.clone(), ui_sender.clone())?;
    }
    #[cfg(feature = "keyboard")]
    {
        start_input_drivers(midi_sender.clone(), ui_sender.clone(), &mut threads)?;
    }
    println!("Input initialized");

    do_ui(&mut display, ui_receiver, || {
        join_finished_threads(&mut threads)
    });
}

fn join_finished_threads(threads: &mut Vec<JoinHandle<Result<()>>>) {
    let finished_threads = threads.iter().enumerate().filter_map(|(i, thread)| {
        if thread.is_finished() {
            return Some(i);
        } else {
            None
        }
    });

    for i in finished_threads.rev() {
        match threads.remove(i).join() {
            Err(p) => panic::resume_unwind(p),
            Ok(Err(e)) => panic!("{}", e),
            Ok(Ok(_)) => return,
        }
    }
}
