#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::{default_serial, pins, Usart};
use arduino_hal::hal::port::{PD0, PD1};
use arduino_hal::pac::USART0;
use arduino_hal::port::mode::{Floating, Input, Output};
use arduino_hal::port::Pin;
use arduino_hal::prelude::*;
use avr_device::atmega328p::Peripherals;

use shift::ShiftRegister;

use crate::keybed::{Key, Keybed};
use crate::millis::millis_init;

mod keybed;
mod millis;
mod shift;

const SERIAL_BAUD: u32 = 115_200;
type Serial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;
panic_serial::impl_panic_handler!(Serial);

const FIRMWARE_VERSION: &str = concat!(
    "I am a keyboard :) ",
    include_str!(concat!(
        env!("OUT_DIR"),
        "/../../../key-firmware-version.txt"
    ))
);

// A serial message may start with either a 'D', 'U', 'V' or 'P'

fn write_string(serial: &mut Serial, value: &str) {
    let str_len = value.len();
    assert!(
        str_len < u8::MAX as usize,
        "String must be less than 255 characters"
    );
    serial.write_byte(str_len as u8);
    serial.write_str(value).unwrap()
}

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
    let serial = default_serial!(dp, pins, SERIAL_BAUD);
    let serial = share_serial_port_with_panic(serial);

    unsafe { avr_device::interrupt::enable() };
    millis_init(dp.TC0);

    send_version(serial);

    let mut keybed = keybed_init!(pins);
    loop {
        keybed.scan(|key, state| match state {
            Key::Up => send_note_up(serial, key as u8),
            Key::Down(travel_time) => send_note_down(serial, key as u8, travel_time as u8),
            _ => {}
        });
    }
}
