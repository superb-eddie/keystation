use std::{env, fs};
use std::io::{copy, Read, Write};
use std::path::{Path, PathBuf};
use flate2::Compression;
use flate2::read::ZlibEncoder;

use rmp::{decode, encode};

use rs_tty::TTY;

const SERIAL_DEVICE: &str =
    "/dev/serial/by-id/usb-Moreland_Moreland_Associates_Keystation_I_am_a_keyboard_:3-if04";
const SERIAL_BAUD: u32 = 115_200;

fn send_payload(mut to: impl Write, payload_path: impl AsRef<Path>) {
    let payload_length = fs::metadata(&payload_path).unwrap().len();
    let mut payload = fs::OpenOptions::new()
        .read(true)
        .open(payload_path)
        .unwrap();

    println!("Sending update payload. {} bytes", payload_length);

    encode::write_u64(&mut to, payload_length).unwrap();

    let mut compressed_payload = ZlibEncoder::new(payload, Compression::fast());

    copy(&mut compressed_payload, &mut to).unwrap();
}

fn do_ping(mut serial: &mut TTY) -> bool {
    // TODO: This doesn't have a timeout, so we'll hang forever if the keystation component isn't working
    encode::write_str(&mut serial, "ping").unwrap();

    let buffer: &mut [u8] = &mut [0u8; 4];
    let resp = decode::read_str(&mut serial, buffer).unwrap();

    return resp == "pong";
}

const USE_MESSAGE: &'static str = "use: <rootfs|boot> <image_path>";

fn main() {
    let mut serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);

    if !do_ping(&mut serial) {
        panic!("ping failed!")
    };

    let mut args = env::args().skip(1);
    assert_eq!(args.len(), 2, "{}", USE_MESSAGE);

    let command = match args.next().unwrap().as_str() {
        "rootfs" => "update_rootfs",
        "boot" => "update_boot",
        _ => panic!("{}", USE_MESSAGE),
    };

    let image_path = PathBuf::from(args.next().unwrap());
    if !image_path.exists() {
        panic!("{}", USE_MESSAGE)
    }

    encode::write_str(&mut serial, command).unwrap();
    // send_payload(&mut serial, image_path)
}
