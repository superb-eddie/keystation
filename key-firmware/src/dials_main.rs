#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::{pins};
use avr_device::atmega328p::{Peripherals};
use crate::millis::millis_init;
use crate::serial::{write_string, Serial};

mod millis;
mod serial;

const FIRMWARE_VERSION: &str = concat!(
    "I am dials! :3 ",
    include_str!(concat!(
        env!("OUT_DIR"),
        "/../../../key-firmware-version.txt"
    ))
);

fn send_version(serial: &mut Serial) {
    serial.write_byte(b'V');
    write_string(serial, FIRMWARE_VERSION);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let serial = serial_init!(dp, pins);    unsafe { avr_device::interrupt::enable() };

    unsafe { avr_device::interrupt::enable() };
    millis_init(dp.TC0);

    send_version(serial);

    loop {

    }
}