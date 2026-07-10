use std::io::{stderr, stdout, Read, Write};
use std::process::Command;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::{fs, thread};

use anyhow::Result;
use crossbeam::channel::Sender;

use rs_tty::TTY;

use crate::midi_sender::MidiEvent;

const SERIAL_DEVICE: &str = "/dev/ttyUSB0";
const SERIAL_BAUD: u32 = 115_200;

const FIRMWARE_BIN: &str = "/usr/share/dials.elf";
const FIRMWARE_VERSION: &str = "/usr/share/dials-version.txt";
const FIRMWARE_HEADER: &str = "I am dials! :3 ";

pub fn start_dials_driver(midi_channel: Sender<MidiEvent>) -> Result<JoinHandle<Result<()>>> {
    let expected_firmware_version = fs::read_to_string(FIRMWARE_VERSION)?;

    let mut serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);
    serial.flush()?;

    Ok(thread::spawn(move || {

        let buffer = [0u8; 3];
        loop {
            match read_next_firmware_message(&mut serial, buffer) {
                FirmwareMessage::Version(version) => {
                    if version != format!("{}{}", FIRMWARE_HEADER, expected_firmware_version) {
                        println!(
                            "Firmware version doesn't match! \n{}{} \n{}",
                            FIRMWARE_HEADER, expected_firmware_version, version
                        );
                        serial = flash_firmware(serial);
                    } else {
                        println!("{}", version);
                    }
                }
                FirmwareMessage::Panic() => {
                    // TODO: Add watchdog to arduino, then we could set a timeout here and wait for it to restart
                    panic!("Arduino panicked!")
                }
            }
        }
    }))
}

fn flash_firmware(serial: TTY) -> TTY {
    // Temporarily take ownership of serial port, so we can drop it to close the file
    drop(serial);

    println!("Flashing dial firmware!");

    let status = Command::new("avrdude")
        .args([
            "-p",
            "atmega328p",
            "-c",
            "arduino",
            "-P",
            SERIAL_DEVICE,
            "-b",
            format!("{}", SERIAL_BAUD).as_ref(),
            "-e",
            "-D",
            "-U",
            format!("flash:w:{}:e", FIRMWARE_BIN).as_ref(),
        ])
        .stdout(stdout())
        .stderr(stderr())
        .spawn()
        .expect("Could not spawn avrdude")
        .wait()
        .unwrap();

    if !status.success() {
        panic!("avrdude failed to flash firmware")
    }

    println!("Dial firmware updated, waiting just a moment before proceeding...");
    sleep(Duration::from_secs_f32(0.1));

    TTY::open(SERIAL_DEVICE, SERIAL_BAUD)
}

enum FirmwareMessage {
    Version(String),
    Panic(),
}

fn read_next_firmware_message(serial: &mut TTY, mut buffer: [u8; 3]) -> FirmwareMessage {
    serial.read_exact(&mut buffer[0..1]).unwrap();

    loop {
        return match buffer[0] {
            b'V' => {
                serial.read_exact(&mut buffer[1..2]).unwrap();

                let str_len = buffer[1];
                let mut str_buf = vec![0u8; str_len as usize];

                serial.read_exact(&mut str_buf).unwrap();

                let version = String::from_utf8(str_buf).expect("Version string was not utf8");

                FirmwareMessage::Version(version)
            }
            b'P' | b'p' => FirmwareMessage::Panic(),
            _ => {
                // Who knows what we read
                continue;
            }
        };
    }
}
