use std::fs;
use std::io::{Read, stderr, stdout, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use midir::{MidiOutput, MidiOutputConnection};
use midir::os::unix::VirtualOutput;

use rs_tty::TTY;

//  TODO: Handle errors more gracefully once code solidifies

// TODO: Detect the correct serial device. It shouldn't change, but just in case
const SERIAL_DEVICE: &str = "/dev/ttyUSB0";
const SERIAL_BAUD: u32 = 115_200;

const FIRMWARE_BIN: &str = "/usr/share/key-firmware.elf";
const FIRMWARE_VERSION: &str = "/usr/share/key-firmware-version.txt";
const FIRMWARE_HEADER: &str = "I am a keyboard! :3 ";

const MIDI_CLIENT_NAME: &str = "keystation";
const MIDI_PORT_NAME: &str = "midi_out";
const MIDI_CHANNEL: u8 = 0; // 0-15

const MIDI_NOTE_ON: u8 = 0x90 + MIDI_CHANNEL;
const MIDI_NOTE_OFF: u8 = 0x80 + MIDI_CHANNEL;

// TODO: Split threads for reading/writing

fn flash_firmware(serial: TTY) -> TTY {
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

    return TTY::open(SERIAL_DEVICE, SERIAL_BAUD);
}

enum FirmwareMessage {
    Version(String),
    KeyDown(u8, u8),
    KeyUp(u8),
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
            b'D' => {
                serial.read_exact(&mut buffer[1..3]).unwrap();

                FirmwareMessage::KeyDown(buffer[1], buffer[2])
            }
            b'U' => {
                serial.read_exact(&mut buffer[1..2]).unwrap();

                FirmwareMessage::KeyUp(buffer[1])
            }
            b'P' | b'p' => FirmwareMessage::Panic(),
            _ => {
                // Who knows what we read
                continue;
            }
        };
    }
}

// TODO: Support microtonal tunings
fn note(key: u8) -> u8 {
    // midi middle c = 60
    // keyboard middle c = 24
    let midi = 60 + (key - 24);
    println!("{} {}", key, midi);
    return midi
}

fn linear_curve(t: f32) -> f32 {
    return t
}

fn pow_curve(pow: f32) -> impl Fn(f32) -> f32 {
    move |t: f32| t.powf(pow.clamp(0.0, 10.0))
}

fn calc_velocity(travel_time: u8, curve: impl FnOnce(f32) -> f32) -> u8 {
    // The firmware reports the time between each contact being pressed in whole milliseconds
    // Midi expects some number in [0-127]
    let min_travel_time = 1.0f32;
    let max_travel_time = 80.0f32;

    let clamped_travel_time = (travel_time as f32).clamp(min_travel_time, max_travel_time);

    let norm_travel_time = (clamped_travel_time - min_travel_time) / (max_travel_time - min_travel_time);

    let velocity = 127.0 - (curve(norm_travel_time) * 126.0);
    assert!(velocity <= 127.0);
    assert!(velocity > 0.0);

    return velocity as u8;
}

fn note_on(midi_out: &mut MidiOutputConnection, note: u8, velocity: u8) {
    // notes 0-127
    // velocity 0-127

    let message = &[MIDI_NOTE_ON, note.min(127), velocity.min(127)];

    midi_out.send(message).unwrap();
}

fn note_off(midi_out: &mut MidiOutputConnection, note: u8) {
    let message = &[MIDI_NOTE_OFF, note.min(127), 0];

    midi_out.send(message).unwrap();
}

fn process_firmware_messages(mut serial: TTY, mut midi_port: MidiOutputConnection, expected_firmware_version: String) -> ! {
    let vel_curve = pow_curve(2.0);

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
            FirmwareMessage::KeyDown(key, travel_time) => {
                note_on(&mut midi_port, note(key), calc_velocity(travel_time, &vel_curve));
            }
            FirmwareMessage::KeyUp(key) => {
                note_off(&mut midi_port, note(key));
            }
            FirmwareMessage::Panic() => {
                // TODO: Add watchdog to arduino, then we could set a timeout here and wait for it to restart
                panic!("Arduino panicked!")
            }
        }
    }
}

fn main() {
    println!("Key Driver");

    let expected_firmware_version =
        fs::read_to_string(FIRMWARE_VERSION).expect("Could not read expected firmware version");

    let mut serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);
    serial.flush().expect("Couldn't flush serial port");

    let midi_out = MidiOutput::new(MIDI_CLIENT_NAME).unwrap();

    let firmware_midi_port = midi_out.create_virtual(MIDI_PORT_NAME).unwrap();
    process_firmware_messages(serial, firmware_midi_port, expected_firmware_version)

}
