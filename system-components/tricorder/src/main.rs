use std::env;

use rmp::{decode, encode};

use rs_tty::TTY;

const SERIAL_DEVICE: &str =
    "/dev/serial/by-id/usb-Moreland_Moreland_Associates_Keystation_I_am_a_keyboard_:3-if04";
const SERIAL_BAUD: u32 = 115_200;

const USE_MESSAGE: &str = "use: <command>";

fn main() {
    let mut serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);

    let mut args = env::args().skip(1);
    assert_eq!(args.len(), 1, "{}", USE_MESSAGE);

    let command = args.next().unwrap();
    encode::write_str(&mut serial, command.as_str()).unwrap();

    let buffer: &mut [u8] = &mut [0u8; 255];
    let resp = decode::read_str(&mut serial, buffer).unwrap();

    println!("{}", resp)
}
