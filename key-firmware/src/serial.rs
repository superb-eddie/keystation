use arduino_hal::port::Pin;
use arduino_hal::{Usart};
use arduino_hal::hal::port::{PD0, PD1};
use arduino_hal::port::mode::{Input, Output};
use arduino_hal::prelude::_ufmt_uWrite;
use avr_device::atmega328p::{USART0};

pub const SERIAL_BAUD: u32 = 115_200;
pub type Serial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;
panic_serial::impl_panic_handler!(Serial);

#[macro_export]
macro_rules! serial_init {
    ($periph:expr, $pins:expr) => {
            crate::serial::share_serial_port_with_panic(arduino_hal::default_serial!($periph, $pins, crate::serial::SERIAL_BAUD))
    }
}

pub fn write_string(serial: &mut Serial, value: &str) {
    let str_len = value.len();
    assert!(
        str_len < u8::MAX as usize,
        "String must be less than 255 characters"
    );
    serial.write_byte(str_len as u8);
    serial.write_str(value).unwrap()
}