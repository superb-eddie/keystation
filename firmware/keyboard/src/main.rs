#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use crate::keybed::{KeyState, Keybed};
use crate::shift::ShiftRegister;
use arduino_hal::port::mode::{Floating, Input};
use arduino_hal::port::Pin;
use arduino_hal::pins;
use avr_device::atmega328p::Peripherals;
use shared::millis::millis_init;
use shared::serial::{write_string, Serial};
use shared::serial_init;

mod keybed;
mod shift;

const FIRMWARE_VERSION: &str = concat!(
    "I am a keyboard! :3 ",
    include_str!(concat!(
        env!("OUT_DIR"),
        "/../../../keyboard-version.txt"
    ))
);

// A serial message may start with either a 'D', 'U', 'V' or 'P'

fn send_version(serial: &mut Serial) {
    serial.write_byte(b'V');
    write_string(serial, FIRMWARE_VERSION);
}

fn send_note_down(serial: &mut Serial, key_index: u8, velocity: u8) {
    serial.write_byte(b'D');
    serial.write_byte(key_index);
    serial.write_byte(velocity);
}

fn send_note_up(serial: &mut Serial, key_index: u8) {
    serial.write_byte(b'U');
    serial.write_byte(key_index);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let serial = serial_init!(dp, pins);

    unsafe { avr_device::interrupt::enable() };
    millis_init(dp.TC0);

    send_version(serial);

    let mut keybed = keybed_init!(pins);
    loop {
        // TODO: Scan should instead collect a list of which keys changed so we can send the messages in bulk
        keybed.scan(|key, state| match state {
            KeyState::Up => send_note_up(serial, key as u8),
            KeyState::Down(travel_time) => send_note_down(serial, key as u8, travel_time as u8),
            _ => {}
        });
    }
}
