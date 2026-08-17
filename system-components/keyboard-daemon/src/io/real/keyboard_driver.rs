use std::io::Read;
use std::thread;
use std::thread::JoinHandle;

use anyhow::{anyhow, Result};
use crossbeam::channel::Sender;
use midly::num::u7;
use midly::MidiMessage::{NoteOff, NoteOn};

use crate::io::io_impl::arduino::Arduino;
use crate::midi_sender::MidiEvent;
use rs_tty::TTY;

const SERIAL_DEVICE: &str = "/dev/ttyUSBkeyboard";
const SERIAL_BAUD: u32 = 115_200;

const FIRMWARE_BIN: &str = "/usr/share/keyboard.elf";
const FIRMWARE_VERSION: &str = "/usr/share/keyboard-version.txt";
const FIRMWARE_HEADER: &str = "I am a keyboard! :3 ";

enum Message {
    KeyDown(u8, u8),
    KeyUp(u8),
}

fn read_next_message(buffer: &mut [u8; 3], serial: &mut TTY) -> Result<Message> {
    match buffer[0] {
        b'D' => {
            serial.read_exact(&mut buffer[1..3])?;
            Ok(Message::KeyDown(buffer[1], buffer[2]))
        }
        b'U' => {
            serial.read_exact(&mut buffer[1..2])?;
            Ok(Message::KeyUp(buffer[1]))
        }
        _ => {
            // Who knows what we read
            Err(anyhow!("Unknown keyboard message..."))
        }
    }
}

pub fn start_keyboard_driver(midi_channel: Sender<MidiEvent>) -> Result<JoinHandle<Result<()>>> {
    let mut arduino = Arduino::new(
        FIRMWARE_BIN,
        FIRMWARE_VERSION,
        FIRMWARE_HEADER,
        SERIAL_DEVICE,
        SERIAL_BAUD,
        read_next_message,
    )?;

    Ok(thread::spawn(move || {
        let vel_curve = pow_curve(2.0);

        loop {
            midi_channel.try_send(match arduino.read_next_message()? {
                Message::KeyDown(key, travel_time) => NoteOn {
                    key: u7::new(note(key)),
                    vel: u7::new(calc_velocity(travel_time, &vel_curve)),
                },
                Message::KeyUp(key) => NoteOff {
                    key: u7::new(note(key)),
                    vel: Default::default(),
                },
            })?;
        }
    }))
}

// TODO: Support microtonal tunings
fn note(key: u8) -> u8 {
    // midi middle c = 60
    // keyboard middle c = 24
    let midi = 60 + (key - 24);
    println!("{} {}", key, midi);

    midi
}

// fn linear_curve(t: f32) -> f32 {
//     return t;
// }

fn pow_curve(pow: f32) -> impl Fn(f32) -> f32 {
    move |t: f32| t.powf(pow.clamp(0.0, 10.0))
}

fn calc_velocity(travel_time: u8, curve: impl FnOnce(f32) -> f32) -> u8 {
    // The firmware reports the time between each contact being pressed in whole milliseconds
    // Midi expects some number in [0-127]
    let min_travel_time = 1.0f32;
    let max_travel_time = 80.0f32;

    let clamped_travel_time = (travel_time as f32).clamp(min_travel_time, max_travel_time);

    let norm_travel_time =
        (clamped_travel_time - min_travel_time) / (max_travel_time - min_travel_time);

    let velocity = 127.0 - (curve(norm_travel_time) * 126.0);
    assert!(velocity <= 127.0);
    assert!(velocity > 0.0);

    velocity as u8
}
