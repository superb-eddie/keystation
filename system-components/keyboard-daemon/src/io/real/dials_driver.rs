use std::io::Read;
use std::thread;
use std::thread::JoinHandle;

use anyhow::{anyhow, Result};
use crossbeam::channel::Sender;

use crate::io::io_impl::arduino::Arduino;
use crate::midi_sender::MidiEvent;
use rs_tty::TTY;

const SERIAL_DEVICE: &str = "/dev/ttyUSBdials";
const SERIAL_BAUD: u32 = 115_200;

const FIRMWARE_BIN: &str = "/usr/share/dials.elf";
const FIRMWARE_VERSION: &str = "/usr/share/dials-version.txt";
const FIRMWARE_HEADER: &str = "I am dials! :3 ";

enum Message {
    Volume(u16),
    Pitch(u16),
    Modulation(u16),
}

fn read_next_message(buffer: &mut [u8; 3], serial: &mut TTY) -> Result<Message> {
    match buffer[0] {
        b'F' => {
            serial.read_exact(&mut buffer[1..3])?;
            Ok(Message::Volume(u16::from_be_bytes([buffer[0], buffer[1]])))
        }
        b'G' => {
            serial.read_exact(&mut buffer[1..3])?;
            Ok(Message::Pitch(u16::from_be_bytes([buffer[0], buffer[1]])))
        }
        b'H' => {
            serial.read_exact(&mut buffer[1..3])?;
            Ok(Message::Modulation(u16::from_be_bytes([buffer[0], buffer[1]])))
        }
        _ => {
            // Who knows what we read
            Err(anyhow!("Unknown dials message..."))
        }
    }
}

pub fn start_dials_driver(midi_channel: Sender<MidiEvent>) -> Result<JoinHandle<Result<()>>> {
    let mut arduino = Arduino::new(
        FIRMWARE_BIN,
        FIRMWARE_VERSION,
        FIRMWARE_HEADER,
        SERIAL_DEVICE,
        SERIAL_BAUD,
        read_next_message,
    )?;

    Ok(thread::spawn(move || loop {
        match arduino.read_next_message()? {
            Message::Volume(val) => {
                println!("Volume: {}", val);
            }
            Message::Pitch(val) => {
                println!("Pitch: {}", val);
            }
            Message::Modulation(val) => {
                println!("Modulation: {}", val);
            }
        }
    }))
}
