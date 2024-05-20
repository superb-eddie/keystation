#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
mod shift;
mod millis;

use core::mem;
use arduino_hal::prelude::*;
use arduino_hal::port::mode::{Floating, Input, Output};
use arduino_hal::port::{Pin};
use arduino_hal::{default_serial, Usart};
use arduino_hal::hal::port::{PD0, PD1};
use arduino_hal::pac::USART0;
use shift::ShiftRegister;
use crate::millis::{millis, millis_init};

panic_serial::impl_panic_handler!(Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>);

// The key matrix has 8 inputs and 14 outputs to read from 49 total keys.
// Each key has two contacts, to calculate velocity.

#[derive(Copy, Clone)]
enum Key {
    // Key goes through these states in order
    // B up, A up
    Up,
    // B down, A up (Key may go to "up" after "partial")
    DownPartial(u32), // millis
    // B down, A down (Key may only go to "down" after "partial")
    Down(u32), // millis travel time
}

const KEYS: usize = 49;
// const KEY_TRAVEL: f32 = 3.5; // mm

#[arduino_hal::entry]
fn main() -> ! {
    unsafe { avr_device::interrupt::enable() };

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = default_serial!(dp, pins, 57600);
    let serial = share_serial_port_with_panic(serial);

    ufmt::uwriteln!(serial, "Oh, hello").unwrap_infallible();

    millis_init(dp.TC0);

    let mut shift = ShiftRegister::new(
        pins.d13.into_output().downgrade(),
        pins.d12.into_output().downgrade(),
        pins.d11.into_output().downgrade(),
    );

    let keys_a: [Pin<Input<Floating>>; 7] = [
        pins.d6.downgrade(),
        pins.d3.downgrade(),
        pins.a5.downgrade(),
        pins.a3.downgrade(),
        pins.a1.downgrade(),
        pins.d10.downgrade(),
        pins.d8.downgrade(),
    ];
    let keys_b: [Pin<Input<Floating>>; 7] = [
        pins.d5.downgrade(),
        pins.d4.downgrade(),
        pins.a4.downgrade(),
        pins.a2.downgrade(),
        pins.a0.downgrade(),
        pins.d9.downgrade(),
        pins.d7.downgrade(),
    ];

    let mut keys = [Key::Up; KEYS];

    ufmt::uwriteln!(serial, "I am piano :3").unwrap_infallible();

    shift.enable();

    loop {
        // Scan key matrix
        for i in 0..8 {
            if i == 0 {
                shift.push_high()
            } else {
                shift.push_low()
            }
            for j in 0..7 {
                let (key_index, _) = ((j * 8) + (if i == 0 { 8usize } else { i })).overflowing_sub(1);
                if key_index >= KEYS {
                    //  This matrix support more keys than we actually have
                    continue
                }

                // let key = &mut keys[key_index];
                let a_down = keys_a[j].is_high();
                let b_down = keys_b[j].is_high();

                if !b_down {
                    // Key is always up if B is up
                    keys[key_index] = Key::Up;
                    continue;
                }

                match (a_down, keys[key_index]) {
                    (false, Key::Up) => {
                        // Key is being pressed
                        keys[key_index] = Key::DownPartial(millis());
                    },
                    (true, Key::DownPartial(at)) => {
                        // Key was fully pressed
                        let travel_time = millis().saturating_sub(at);
                        keys[key_index] = Key::Down(travel_time);

                        ufmt::uwriteln!(serial, "Key {} - {}ms", key_index, travel_time).unwrap_infallible();
                    },
                    (false, Key::Down(_velocity)) => {
                        // Key is being lifted
                        keys[key_index] = Key::Up
                    },
                    _ => {
                    //     Nothing happens in every other case
                    },
                }
            }
        }
    }
}
