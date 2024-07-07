use std::fs;
use std::io::{copy, Read};
use std::path::{Path, PathBuf};

use rmp::{decode, encode};

use rs_tty::TTY;

use flate2::read::{GzDecoder, ZlibDecoder};

const SERIAL_DEVICE: &str = "/dev/ttyGS1";
const SERIAL_BAUD: u32 = 115_200;

const GADGET_LUN_PATH: &str = "/sys/kernel/config/usb_gadget/keystation/functions/mass_storage.usb0/lun.0/file";

fn recv_payload(mut from: impl Read, dest: impl AsRef<Path>) {
    let payload_length =
        decode::read_u64(&mut from).expect("could not read length of update payload");

    println!("Receiving payload. {} bytes", payload_length);

    let mut dest = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&dest)
        .unwrap();

    let mut uncompressed_payload = ZlibDecoder::new(from).take(payload_length);        

    copy(&mut uncompressed_payload, &mut dest)
        .expect("could not write update payload");
}

const CMDLINE_PATH: &'static str = "/boot/cmdline.txt";
fn current_rootfs_partition() -> PathBuf {
    let kernel_cmdline = fs::read_to_string(CMDLINE_PATH).unwrap();
    let root_option = kernel_cmdline
        .split(" ")
        .find(|option| option.starts_with("root="))
        .and_then(|root_options| root_options.split_once("="))
        .and_then(|(_, root)| Some(root))
        .expect("Can't determine current rootfs");

    PathBuf::from(root_option)
}

fn unused_rootfs_partition() -> PathBuf {
    let current_partition = current_rootfs_partition();
    let current_part_name = current_partition.file_name().unwrap().to_str().unwrap();

    match current_part_name {
    "mmcblk0p2" => current_partition.with_file_name("mmcblk0p3"),
    "mmcblk0p3" => current_partition.with_file_name("mmcblk0p2"),
    _ => panic!("Unknown partition {}", current_part_name)
    }
}

fn main() {
    let mut serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);

    println!("Starting!");

    let buffer: &mut [u8] = &mut [0u8; 255];

    loop {
        let command = decode::read_str(&mut serial, buffer).unwrap();
        println!("{}", command);

        match command {
            "ping" => {
                encode::write_str(&mut serial, "pong").unwrap();
            }
            "update_rootfs" => {
                let unused_root = unused_rootfs_partition();

                fs::write(GADGET_LUN_PATH, unused_root.to_str().unwrap()).unwrap();

                // println!("{}", unused_root.display());
                //
                // recv_payload(&mut serial, unused_root);
            }
            "update_boot" => {
                let payload_path ="/tmp/boot_update_payload";
                recv_payload(&mut serial, payload_path);
                println!("{}", payload_path)
            }
            "reboot" => {
                unimplemented!();
            }
            &_ => {}
        }
    }
}
