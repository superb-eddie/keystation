#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use crate::keybed::{KeyState, Keybed};
use crate::shift::ShiftRegister;
use arduino_hal::hal::wdt;
use arduino_hal::pins;
use arduino_hal::port::mode::{Floating, Input};
use arduino_hal::port::Pin;
use avr_device::atmega328p::Peripherals;
use shared::millis::millis_init;
use shared::serial::{write_msg_2u8, write_msg_str, write_msg_u8};
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
// 'P' messages are written by the shared panic handler
const MSG_VERSION: u8 = b'V';
const MSG_NOTE_UP: u8 = b'U';
const MSG_NOTE_DOWN: u8 = b'D';


#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let serial = serial_init!(dp, pins);

    unsafe { avr_device::interrupt::enable() };
    millis_init(dp.TC0);

    let mut watchdog = wdt::Wdt::new(dp.WDT, &dp.CPU.mcusr);
    watchdog.start(wdt::Timeout::Ms500).unwrap();

    write_msg_str(serial, MSG_VERSION, FIRMWARE_VERSION);

    let mut keybed = keybed_init!(pins);
    loop {
        // TODO: Scan should instead collect a list of which keys changed so we can send the messages in bulk
        keybed.scan(|key, state| match state {
            KeyState::Up => write_msg_u8(serial, MSG_NOTE_UP, key as u8),
            KeyState::Down(travel_time) => write_msg_2u8(serial, MSG_NOTE_DOWN, key as u8, travel_time as u8),
            _ => {}
        });

        serial.flush();
        // TODO: How could we detect if scanning goes beyond the watchdog timeout
        watchdog.feed();
    }
}
