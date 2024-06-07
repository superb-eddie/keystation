
// const FIRMWARE_BIN: &str = "/usr/share/key-firmware.elf";
// const FIRMWARE_VERSION: &str = "/usr/share/key-firmware-version.txt";

use std::fs::File;
use std::io::{Read, stdout, Write};
use std::path::Path;
use std::time::Duration;
use crate::serial::SerialPort;
// use std::os::fd::{FromRawFd, IntoRawFd, RawFd};

mod serial;

// TODO: Detect the correct serial device. It shouldn't change, but just in case
const SERIAL_DEVICE: &str = "/dev/ttyUSB0";
const SERIAL_BAUD: u32 = 115_200;

// TODO: Split threads for reading/writing

fn main() {
    println!("Key Driver");

    let mut serial = SerialPort::open(SERIAL_DEVICE, SERIAL_BAUD);
    serial.flush();

    let mut buffer = [0u8; 3];
    loop {
        serial.read_exact(&mut buffer[0..1]).unwrap();

        match buffer[0] {
            b'V' => {
                serial.read_exact(&mut buffer[1..2]).unwrap();

                let str_len = buffer[1];
                let mut str_buf = vec![0u8; str_len as usize];

                serial.read_exact(&mut str_buf).unwrap();

                stdout().write(String::from_utf8(str_buf).expect("Version string was not").as_ref()).unwrap();
            }
            b'D' => {
                // A key was pressed!
                serial.read_exact(&mut buffer[1..3]).unwrap();
                println!("D {} {}", buffer[1], buffer[2])
            }
            b'U' => {
                serial.read_exact(&mut buffer[1..2]).unwrap();
                println!("P {} {}", buffer[1], buffer[2])
            }
            b'P' | b'p' => {
                // Could also
                // TODO: Add watchdog to arduino, then set a timeout here and wait for it to restart
                panic!("Arduino panicked!")
            }
            _ => {
                // Who knows what we read, but it's not a keypress!
            }
        }
    }



}

