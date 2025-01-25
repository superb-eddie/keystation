use std::fs;
use std::io::{Read, stderr, stdout};
use std::path::PathBuf;
use std::process::Command;

use rmp::{decode, encode};

use rs_tty::TTY;

use anyhow::{anyhow, bail, Result};

const SERIAL_DEVICE: &str = "/dev/ttyGS1";
const SERIAL_BAUD: u32 = 115_200;

// The kernel is set up to act as a usb device with removable storage.
// We can direct it to expose a file by writing its path to this path/
const GADGET_LUN_PATH: &str =
    "/sys/kernel/config/usb_gadget/keystation/functions/mass_storage.usb0/lun.0/file";

const CMDLINE_PATH: &'static str = "/boot/cmdline.txt";

fn parse_cmdline_root(kernel_cmdline: &str) -> Option<&str> {
    return  kernel_cmdline
        .split(" ")
        .find(|option| option.starts_with("root="))
        .and_then(|root_options| root_options.split_once("="))
        .and_then(|(_, root)| Some(root))
}

fn current_rootfs_partition() -> Result<PathBuf> {
    let kernel_cmdline = fs::read_to_string(CMDLINE_PATH)?;
    let root_option = parse_cmdline_root(&kernel_cmdline).unwrap();

    Ok(PathBuf::from(root_option))
}

fn unused_rootfs_partition() -> Result<PathBuf> {
    let current_partition = current_rootfs_partition()?;
    let current_part_name = current_partition.file_name().unwrap().to_str().unwrap();

    Ok(match current_part_name {
        "mmcblk0p2" => current_partition.with_file_name("mmcblk0p3"),
        "mmcblk0p3" => current_partition.with_file_name("mmcblk0p2"),
        _ => bail!("Unknown partition {}", current_part_name),
    })
}

fn set_rootfs_partition(new_rootfs: &str) -> Result<()> {
    let kernel_cmdline = fs::read_to_string(CMDLINE_PATH)?;
    let root_option = parse_cmdline_root(&kernel_cmdline).ok_or(anyhow!("Couldn't get root option"))?;

    let new_cmdline = kernel_cmdline.replace(root_option, &new_rootfs);
    fs::write(CMDLINE_PATH, new_cmdline)?;

    Ok(())
}

fn reboot() -> Result<()> {
    let status = Command::new("reboot")
        .args(["now"])
        .stdout(stdout())
        .stderr(stderr())
        .spawn()?
        .wait()?;

    if !status.success() {
        bail!("Reboot command failed")
    }
    Ok(())
}

fn main() {
    let mut serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);

    println!("Starting!");
    let buffer: &mut [u8] = &mut [0u8; 255];
    loop {
        let command = match decode::read_str(&mut serial, buffer) {
            Err(e) => {
                eprintln!("{}", e);

                // Who know what kinda garbage we're seeing
                // Reopening the TTY will give us a "clean slate"
                drop(serial);
                serial = TTY::open(SERIAL_DEVICE, SERIAL_BAUD);

                continue
            }
            Ok(c) => {
                println!("{}", c);
                c
            }
        };

        match do_cmd(command.to_owned(), |resp| {
            encode::write_str(&mut serial, resp)?; Ok(())
        }) {
            Err(e) => {
                eprintln!("Error running cmd; {}", e);
            }
            Ok(_) => {}
        }
    }
}

fn do_cmd(command: String, resp: impl FnOnce(&str) -> Result<()>) -> Result<()> {
    match command.as_str() {
        "ping" => {
            resp("pong")
        }
        "start_update" => {
            // "insert" partition
            let unused_root = unused_rootfs_partition()?;
            fs::write(GADGET_LUN_PATH, unused_root.to_str().unwrap())?;

            resp("ok")
        }
        "end_update" => {
            // "remove" partition 
            fs::write(GADGET_LUN_PATH, "")?;

            // Swap roots
            let unused_root = unused_rootfs_partition()?;
            set_rootfs_partition(unused_root.to_str().unwrap())?;

            resp("ok")?;

            // Reboot
            reboot()?;

            Ok(())
        }
        &_ => {
            resp("unknown cmd")
        }
    }
}