// Original code stolen shamelessly from the second popular serial library (but better of the two) `serial2`

use std::{fs, io};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use nix::libc::{O_NOCTTY, O_NONBLOCK};

fn check(ret: i32) -> io::Result<i32> {
    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

/// Check the return value of a syscall for errors.
fn check_isize(ret: isize) -> io::Result<usize> {
    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}

fn poll(file: &fs::File, events: std::os::raw::c_short) -> io::Result<bool> {
    let mut poll_fd = libc::pollfd {
        fd: file.as_raw_fd(),
        events,
        revents: 0,
    };
    check(unsafe { libc::poll(&mut poll_fd, 1, -1) })?;
    Ok(poll_fd.revents != 0)
}

fn set_termios(file: &mut fs::File, rate: u32) {
    // Get the current termios settings
    let mut termios: libc::termios2 = unsafe {
        let mut termios = std::mem::zeroed();
        check(libc::ioctl(
            file.as_raw_fd(),
            libc::TCGETS2 as _,
            &mut termios,
        ))
        .unwrap();

        // Make raw to disable any OS shenanigans
        libc::cfmakeraw(&mut termios as *mut _ as *mut libc::termios);
        termios
    };

    // No flow control
    termios.c_iflag &= !(libc::IXON | libc::IXOFF);
    termios.c_cflag &= !libc::CRTSCTS;

    // No parity
    termios.c_cflag &= !libc::PARODD & !libc::PARENB;

    // One stop bit
    termios.c_cflag &= !libc::CSTOPB;

    // 8 bit words
    termios.c_cflag |= libc::CS8;

    // Set baud rate
    termios.c_cflag &= !(libc::CBAUD | libc::CIBAUD);
    termios.c_cflag |= libc::BOTHER;
    termios.c_cflag |= libc::BOTHER << libc::IBSHIFT;
    termios.c_ospeed = rate;
    termios.c_ispeed = rate;

    check(unsafe { libc::ioctl(file.as_raw_fd(), libc::TCSETSW2 as _, &termios) })
        .expect("could not set baud rate");
}

fn open_tty(device: impl AsRef<Path>, baud_rate: u32) -> fs::File {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .custom_flags(O_NONBLOCK | O_NOCTTY)
        .open(device)
        .expect("Could not open serial device");

    set_termios(&mut file, baud_rate);

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
}

impl io::Read for TTY {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        if !poll(&self.device, libc::POLLIN)? {
            return Err(io::ErrorKind::TimedOut.into());
        }
        loop {
            let result = check_isize(unsafe {
                libc::read(
                    self.device.as_raw_fd(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as _,
                )
            });
            match result {
                Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => continue,
                x => return x,
            }
        }
    }
}

impl io::Write for TTY {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !poll(&self.device, libc::POLLOUT)? {
            return Err(io::ErrorKind::TimedOut.into());
        }
        unsafe {
            loop {
                let result = check_isize(libc::write(
                    self.device.as_raw_fd(),
                    buf.as_ptr().cast(),
                    buf.len() as _,
                ));
                match result {
                    Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => continue,
                    x => return x,
                }
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        check(unsafe { libc::tcdrain(self.device.as_raw_fd()) })?;
        Ok(())
    }
}
