use arduino_hal::hal::port::{PD0, PD1};
use arduino_hal::port::mode::{Input, Output};
use arduino_hal::port::Pin;
use arduino_hal::prelude::_ufmt_uWrite;
use arduino_hal::Usart;
use avr_device::atmega328p::USART0;
use core::panic;
use core::sync::atomic::{compiler_fence, Ordering};

pub const SERIAL_BAUD: u32 = 115_200;
pub static mut SERIAL_PORT: Option<Serial> = None;

pub type Serial = Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

pub fn handle_panic(info: &panic::PanicInfo) -> ! {
    if let Some(serial) = unsafe { SERIAL_PORT.as_mut() } {
        _ = serial.flush();

        // Firmware may have panicked mid-message. If we don't send enough bytes to end the
        //   previous message, then the read might stall or get de-synced.
        _ = serial.write_str("PPPPPPPPPPPPPPPANIC");

        if let Some(location) = info.location() {
            _ = ufmt::uwrite!(serial,"@ {}:{}:{}", location.file(), location.line(), location.column());
        }

        // Just in case reader is somehow stuck still
        _ = serial.write_str("PANIC PANIC");

        // End of transmission
        _ = serial.write_str("\x04");
        _ = serial.flush();
    }
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}

#[macro_export]
macro_rules! serial_init {
    ($periph:expr, $pins:expr) => {
        {
            #[inline(never)]
            #[panic_handler]
            fn panic(info: &core::panic::PanicInfo) -> ! {
                shared::serial::handle_panic(info)
            };

            unsafe {
                shared::serial::SERIAL_PORT = Some(arduino_hal::default_serial!($periph, $pins, shared::serial::SERIAL_BAUD));
                shared::serial::SERIAL_PORT.as_mut().unwrap()
            }
        }
    }
}

pub fn write_msg_str(serial: &mut Serial, header: u8, value: &str) {
    let str_len = value.len();
    assert!(
        str_len < u8::MAX as usize,
        "String must be less than 255 characters"
    );
    serial.write_byte(header);
    serial.write_byte(str_len as u8);
    serial.write_str(value).unwrap()
}

pub fn write_msg_u8(serial: &mut Serial, header: u8, value: u8) {
    serial.write_byte(header);
    serial.write_byte(value);
}

pub fn write_msg_2u8(serial: &mut Serial, header: u8, value1: u8, value2: u8) {
    serial.write_byte(header);
    serial.write_byte(value1);
    serial.write_byte(value2);
}

pub fn write_msg_u16(serial: &mut Serial, header: u8, value: u16) {
    let bytes = value.to_be_bytes();
    serial.write_byte(header);
    serial.write_byte(bytes[0]);
    serial.write_byte(bytes[1]);
}
