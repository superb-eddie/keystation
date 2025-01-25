use std::thread;
use std::time::Duration;

use crossbeam::channel::Sender;
use rppal::gpio::{Gpio, Trigger};

use crate::midi_sender::MidiEvent;

const SUSTAIN_GPIO_PIN: u8 = 20;

// Start a thread to poll for gpio interrupts and translate them to events
pub fn start_gpio_driver(
    midi_channel: Sender<MidiEvent>,
) -> anyhow::Result<thread::JoinHandle<()>> {
    let mut sustain_pin = Gpio::new()?.get(SUSTAIN_GPIO_PIN)?.into_input_pulldown();

    sustain_pin.set_interrupt(Trigger::Both, Some(Duration::from_millis(1)))?;

    let thread_handle = thread::spawn(move || loop {
        let interrupt = sustain_pin.poll_interrupt(false, None).unwrap().unwrap();

        midi_channel
            .try_send(match interrupt.trigger {
                Trigger::RisingEdge => MidiEvent::SustainOn,
                Trigger::FallingEdge => MidiEvent::SustainOff,
                _ => {
                    panic!("Unexpected trigger type {}", interrupt.trigger)
                }
            })
            .expect("couldn't send midi event");
    });

    Ok(thread_handle)
}
