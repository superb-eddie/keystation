// Original code stolen shamelessly from the second popular serial library (but better of the two) `serial2`

use std::{fs, io};
use std::io::Write;
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use nix::errno::Errno;
use nix::libc::{O_NOCTTY, O_NONBLOCK, POLLIN};
use nix::poll::{PollFd, PollFlags};
use nix::sys::termios::{BaudRate, cfmakeraw, cfsetspeed, ControlFlags, InputFlags, SetArg, tcgetattr, tcsetattr};
use nix::unistd::read;


fn poll(file: &fs::File, events: std::os::raw::c_short) -> bool {
    // TODO: Can we keep this around instead?
    PollFd::new(file.as_fd(), PollFlags::from_bits(events).unwrap()).any().unwrap()
}

fn set_termios(file: &mut fs::File, rate: BaudRate) {

    let mut termios = tcgetattr(file.as_fd()).expect("Could not get termios");

    cfmakeraw(&mut termios);

    // No flow control
    termios.input_flags &= !(InputFlags::IXON | InputFlags::IXOFF);
    termios.control_flags &= !ControlFlags::CRTSCTS;

    // No parity
    termios.control_flags &= !ControlFlags::PARODD & !ControlFlags::PARENB;

    // One stop bit
    termios.control_flags &= !ControlFlags::CSTOPB;

    // 8 bit words
    termios.control_flags |= ControlFlags::CS8;

    // Set baud rate
    cfsetspeed(&mut termios, rate).unwrap();
    
    tcsetattr(file.as_fd(), SetArg::TCSANOW, &termios).expect("Could not set termios");
}

fn open_tty(device: impl AsRef<Path>, baud_rate: u32) -> fs::File {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .custom_flags(O_NONBLOCK | O_NOCTTY)
        .open(device)
        .expect("Could not open serial device");

    set_termios(&mut file, baud_rate.try_into().unwrap());

    return file;
}

pub struct TTY {
    device: fs::File,
}

impl TTY {
    pub fn open(device: impl AsRef<Path>, baud_rate: u32) -> Self {
        TTY {
            device: open_tty(device, baud_rate),
        }
    }

    pub fn flush(&mut self) {
        self.device.flush().unwrap()
    }
}

impl io::Read for TTY {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        if !poll(&self.device, POLLIN) {
            return Err(Errno::ETIMEDOUT.into());
        }
        loop {
            match read(
                self.device.as_raw_fd(),
                buf
            ){
                Err(e) => match e {
                    Errno::EINTR => continue,
                    _ => return Err(e.into())
                },
                Ok(t) => return Ok(t)
            }
        }
    }
}
