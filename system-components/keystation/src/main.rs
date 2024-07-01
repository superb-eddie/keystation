use std::io::Write;

use rmp::{decode, encode};

use rs_tty::TTY;

const SERIAL_DEVICE: &str = "/dev/ttyGS1";
const SERIAL_BAUD: u32 = 115_200;

fn main() {
    let mut serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);
    serial.flush().unwrap();

    println!("Starting!");

    let buffer: &mut [u8] = &mut [0u8; 255];
    
    loop {
        let command = decode::read_str(&mut serial, buffer).unwrap();
        println!("{}", command);

        match command {
            "ping" => {
                encode::write_str(&mut serial, "pong").unwrap();
            }
            "start_update" => {}
            &_ => {}
        }
    }
}
