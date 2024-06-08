use std::fs;
use std::io::{Read, stderr, stdout, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::serial::SerialPort;

//  TODO: Handle errors more gracefully once code solidifies

mod midi;
mod serial;

// TODO: Detect the correct serial device. It shouldn't change, but just in case
const SERIAL_DEVICE: &str = "/dev/ttyUSB0";
const SERIAL_BAUD: u32 = 115_200;

const FIRMWARE_BIN: &str = "/usr/share/keystation-firmware.elf";
const FIRMWARE_VERSION: &str = "/usr/share/key-firmware-version.txt";
const FIRMWARE_HEADER: &str = "I am a keyboard! :3 ";

// TODO: Split threads for reading/writing

fn flash_firmware(serial: SerialPort) -> SerialPort {
    // Temporarily take ownership of serial port, so we can drop it to close the file
    drop(serial);

    println!("Flashing keyboard firmware!");

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

    println!("Keyboard firmware updated, waiting just a moment before proceeding...");
    sleep(Duration::from_secs_f32(0.1));

    return SerialPort::open(SERIAL_DEVICE, SERIAL_BAUD);
}

enum FirmwareMessage {
    Version(String),
    KeyDown(u8, u8),
    KeyUp(u8),
    Panic(),
}

fn read_firmware_message(serial: &mut SerialPort, mut buffer: [u8; 3]) -> Option<FirmwareMessage> {
    serial.read_exact(&mut buffer[0..1]).unwrap();

    return match buffer[0] {
        b'V' => {
            serial.read_exact(&mut buffer[1..2]).unwrap();

            let str_len = buffer[1];
            let mut str_buf = vec![0u8; str_len as usize];

            serial.read_exact(&mut str_buf).unwrap();

            let version = String::from_utf8(str_buf).expect("Version string was not utf8");

            Some(FirmwareMessage::Version(version))
        }
        b'D' => {
            serial.read_exact(&mut buffer[1..3]).unwrap();

            Some(FirmwareMessage::KeyDown(buffer[1], buffer[2]))
        }
        b'U' => {
            serial.read_exact(&mut buffer[1..2]).unwrap();

            Some(FirmwareMessage::KeyUp(buffer[1]))
        }
        b'P' | b'p' => Some(FirmwareMessage::Panic()),
        _ => {
            // Who knows what we read, but it's not a keypress!

            None
        }
    };
}

// TODO: Support microtonal tunings
fn note(key: u8) -> u8 {
    // midi middle c = 60
    // keyboard middle c = 24
    return key + (60 - 24);
}

fn main() {
    println!("Key Driver");

    let expected_firmware_version =
        fs::read_to_string(FIRMWARE_VERSION).expect("Could not read expected firmware version");

    let mut serial = SerialPort::open(SERIAL_DEVICE, SERIAL_BAUD);
    serial.flush();

    let mut midi_out = midi::start();

    let buffer = [0u8; 3];
    loop {
        let next_message = match read_firmware_message(&mut serial, buffer) {
            None => continue,
            Some(message) => message,
        };

        match next_message {
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
            FirmwareMessage::KeyDown(key, velocity) => {
                midi::note_on(&mut midi_out, note(key), velocity);
                // eprintln!("D {} {}", key, velocity);
            }
            FirmwareMessage::KeyUp(key) => {
                midi::note_off(&mut midi_out, note(key));
                // eprintln!("U {}", key);
            }
            FirmwareMessage::Panic() => {
                // TODO: Add watchdog to arduino, then we could set a timeout here and wait for it to restart
                panic!("Arduino panicked!")
            }
        }
    }
}
