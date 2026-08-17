// Both arduinos communicate in the same way
// Each message starts with just a single character to indicate the type, followed by some payload.
// The size of the payload is different for each message
// The only built-in messages are
//   - 'V': version, payload is a version string prefixed with its size
//   - 'P': panic, produced by the panic handler

use anyhow::{anyhow, Result};
use rs_tty::TTY;
use std::fs;
use std::io::{stderr, stdout, BufRead, BufReader, Read, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

const WATCHDOG_MS: u64 = 500;

pub struct Arduino<M, F: FnMut(&mut [u8; 3], &mut TTY) -> Result<M>> {
    firmware_header: String,
    expected_firmware_version: String,
    firmware_bin_path: &'static str,

    serial_device_path: &'static str,
    serial_baud: u32,
    serial_device: TTY,

    // Most messages are 3 bytes or smaller, the ones that are bigger
    //      have a size that can be determined from the first 3 bytes
    read_buffer: [u8; 3],
    read_message_fn: F,
}

impl<M, F: FnMut(&mut [u8; 3], &mut TTY) -> Result<M>> Arduino<M, F> {
    pub fn new(
        firmware_bin_path: &'static str,
        firmware_bin_version_path: &'static str,
        firmware_header: &'static str,
        serial_device_path: &'static str,
        serial_baud: u32,
        read_message_fn: F,
    ) -> Result<Self> {
        let firmware_version = fs::read_to_string(firmware_bin_version_path)?;
        let mut serial_device = TTY::open(serial_device_path, serial_baud)?;
        serial_device.flush()?;

        Ok(Self {
            firmware_header: firmware_header.to_string(),
            expected_firmware_version: format!("{}{}", firmware_header, firmware_version),
            firmware_bin_path,

            serial_device_path,
            serial_baud,
            serial_device,

            read_buffer: [0; 3],
            read_message_fn,
        })
    }

    pub fn read_next_message(&mut self) -> Result<M> {
        // Version/panic messages get handled here, and we'll keep reading until there's something to return
        loop {
            self.serial_device.read_exact(&mut self.read_buffer[0..1])?;
            return match self.read_buffer[0] {
                b'V' => {
                    self.handle_version()?;
                    continue;
                }
                b'P' => {
                    self.handle_panic()?;
                    continue;
                }
                _ => {
                    let message = (self.read_message_fn)(&mut self.read_buffer, &mut self.serial_device);
                    if let Err(e) = message {
                        println!("Error reading message: {}", e);

                        println!("Buffer: {:?}", self.read_buffer);

                        Err(e)
                    } else {
                        message
                    }
                }
            };
        }
    }

    fn handle_panic(&mut self) -> Result<()> {
        // Log the panic message and wait for the Arduino to recover

        let mut panic_message_buf = vec![b'P'];

        let mut buf_serial = BufReader::new(&mut self.serial_device);
        buf_serial.read_until(b'\x04', &mut panic_message_buf)?;

        let panic_message = String::from_utf8(panic_message_buf)?;
        println!("{} PANIC: {}", self.firmware_header, panic_message);

        println!("Waiting for watchdog to recover....");
        sleep(Duration::from_millis(WATCHDOG_MS));

        self.reopen_serial()?;
        Ok(())
    }

    fn handle_version(&mut self) -> Result<()> {
        // If the firmware version isn't what we expect then flash the correct version
        // Otherwise just print the version and move on

        self.serial_device.read_exact(&mut self.read_buffer[1..2])?;
        let str_len = self.read_buffer[1] as usize;

        let mut str_buf = vec![0u8; str_len];
        self.serial_device.read_exact(&mut str_buf)?;

        let version = String::from_utf8(str_buf)?;
        if version != self.expected_firmware_version {
            println!(
                "Firmware version mismatch!\n '{}' != '{}'",
                version, self.expected_firmware_version
            );

            let exit_status = Command::new("avrdude")
                .args([
                    "-p",
                    "atmega328p",
                    "-c",
                    "arduino",
                    "-P",
                    self.serial_device_path,
                    "-b",
                    format!("{}", self.serial_baud).as_ref(),
                    "-e",
                    "-D",
                    "-U",
                    format!("flash:w:{}:e", self.firmware_bin_path).as_ref(),
                ])
                .stdout(stdout())
                .stderr(stderr())
                .spawn()?
                .wait()?;

            if !exit_status.success() {
                return Err(anyhow!(
                    "Firmware flash failed. Process returned {}",
                    exit_status.code().unwrap()
                ));
            }

            println!("Firmware flash successful! Waiting for device to restart....");
            sleep(Duration::from_secs_f32(0.1));

            self.reopen_serial()?;
        } else {
            println!("{}", version);
        }

        Ok(())
    }

    fn reopen_serial(&mut self) -> Result<()> {
        // TODO: Does this leak a file descriptor? It's probably okay if it does
        std::mem::forget(std::mem::replace(
            &mut self.serial_device,
            TTY::open(self.serial_device_path, self.serial_baud)?,
        ));
        self.serial_device.flush()?;

        Ok(())
    }
}
