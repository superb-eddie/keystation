#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod faders;

use arduino_hal::hal::wdt;
use arduino_hal::pins;
use avr_device::atmega328p::Peripherals;
use shared::millis::millis_init;
use shared::serial::{write_msg_str, write_msg_u16};
use shared::serial_init;

// // Buttons
// int pinInButtonAF = 3;
// int pinOutLEDAF = 8;
// int pinInButtonPlus = 4;
// int pinOutLEDPlus = 7;
// int pinInButtonMinus = 5;
// int pinOutLEDMinus = 6;
//
// // Pedal
// int pinInPedal = 10;
//
// // Faders
// int pinInVolume = A7;
// int pinInPitch = A0;
// int pinInModulation = A1;

// Output Messages:
// P<panic message>\x04 - Firmware panic (from shared panic handler)
// V<u8 length><string> - Firmware version
const MSG_VERSION: u8 = b'V';

// F<u16 value> - Volume fader value
const MSG_FADER_VOLUME: u8 = b'F';

// G<u16 value> - Pitch fader value
const MSG_FADER_PITCH: u8 = b'G';

// H<u16 value> - Modulation fader value
const MSG_FADER_MODULATION: u8 = b'H';

// B<nibble id><nibble value> - Button control


const FIRMWARE_VERSION: &str = concat!(
    "I am dials! :3 ",
    include_str!(concat!(
        env!("OUT_DIR"),
        "/../../../dials-version.txt"
    ))
);

struct Values {
    volume: u16,
    pitch: u16,
    modulation: u16,
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);
    let serial = serial_init!(dp, pins);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    unsafe { avr_device::interrupt::enable() };
    millis_init(dp.TC0);

    let mut values = Values {
        volume: 0,
        pitch: 0,
        modulation: 0,
    };
    let mut faders = faders_init!(pins, adc);

    let mut watchdog = wdt::Wdt::new(dp.WDT, &dp.CPU.mcusr);
    watchdog.start(wdt::Timeout::Ms500).unwrap();

    write_msg_str(serial, MSG_VERSION, FIRMWARE_VERSION);
    loop {
        let volume_val = faders.read_volume(&mut adc);
        if values.volume != volume_val {
            values.volume = volume_val;
            write_msg_u16(serial, MSG_FADER_VOLUME, volume_val);
        }

        let pitch_val = faders.read_pitch(&mut adc);
        if values.pitch != pitch_val {
            values.pitch = pitch_val;
            write_msg_u16(serial, MSG_FADER_PITCH, pitch_val);
        }

        let modulation_val = faders.read_modulation(&mut adc);
        if values.modulation != modulation_val {
            values.modulation = modulation_val;
            write_msg_u16(serial, MSG_FADER_MODULATION, modulation_val);
            panic!("TESTTESTTEST");
        }

        watchdog.feed();
    }
}