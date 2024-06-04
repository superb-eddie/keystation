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

type Serial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;
panic_serial::impl_panic_handler!(Serial);

const FIRMWARE_VERSION: &str = include!(concat!(
    env!("OUT_DIR"),
    "/../../../key-firmware-version.txt"
));

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let serial = default_serial!(dp, pins, 57600);
    let serial = share_serial_port_with_panic(serial);

    unsafe { avr_device::interrupt::enable() };
    millis_init(dp.TC0);

    ufmt::uwriteln!(serial, "Oh, hello").unwrap_infallible();
    ufmt::uwriteln!(serial, "I am piano :3").unwrap_infallible();

    let mut keybed = keybed_init!(pins);

    loop {
        keybed.scan();

        for (i, key) in keybed.keys.iter().enumerate() {
            if let Key::Down(velocity) = key {
                ufmt::uwriteln!(serial, "Key {} {}", i, velocity).unwrap_infallible();
            }
        }
    }
}
