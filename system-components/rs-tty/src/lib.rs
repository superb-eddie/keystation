// Original code heavily modified from the library `serial2`
// TODO: This whole thing should be re-written to leverage the safe libc wrappers that nix provides,
//          as currently we're only using the libc bindings that it provides.
// https://docs.rs/nix/latest/nix/index.html

use nix::libc::{O_NOCTTY, O_NONBLOCK};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::Duration;
use std::{fs, io};

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

fn poll(file: &fs::File, events: std::os::raw::c_short, timeout: Option<Duration>) -> io::Result<bool> {
    let timeout_raw = if let Some(timeout) = timeout {
        timeout.as_millis() as i32
    } else {
        -1
    };

    let mut poll_fd = libc::pollfd {
        fd: file.as_raw_fd(),
        events,
        revents: 0,
    };
    check(unsafe { libc::poll(&mut poll_fd, 1, timeout_raw) })?;
    Ok(poll_fd.revents != 0)
}

fn set_termios(file: &mut fs::File, rate: u32) -> io::Result<()> {
    // Get the current termios settings
    let mut termios: libc::termios2 = unsafe {
        let mut termios = std::mem::zeroed();
        check(libc::ioctl(
            file.as_raw_fd(),
            libc::TCGETS2 as _,
            &mut termios,
        ))?;

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

    check(unsafe { libc::ioctl(file.as_raw_fd(), libc::TCSETSW2 as _, &termios) })?;

    Ok(())
}

fn open_tty(device: impl AsRef<Path>, baud_rate: u32) -> io::Result<fs::File> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .custom_flags(O_NONBLOCK | O_NOCTTY)
        .open(device)?;

    set_termios(&mut file, baud_rate)?;

    Ok(file)
}

pub struct TTY {
    device: fs::File,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl TTY {
    pub fn open(device: impl AsRef<Path>, baud_rate: u32) -> io::Result<Self> {
        Ok(TTY {
            device: open_tty(device, baud_rate)?,
            read_timeout: None,
            write_timeout: None,
        })
    }

    // Mimics the timeout interface provided by std::TcpStream
    // Although curiously TcpStream's implementation does not require mut
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.read_timeout = if let Some(timeout) = timeout {
            if timeout.is_zero() {
                None
            } else {
                Some(timeout)
            }
        } else {
            None
        };
    }

    pub fn set_write_timeout(&mut self, timeout: Option<Duration>) {
        self.write_timeout = if let Some(timeout) = timeout {
            if timeout.is_zero() {
                None
            } else {
                Some(timeout)
            }
        } else {
            None
        };
    }
}

impl io::Read for TTY {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        if !poll(&self.device, libc::POLLIN, self.read_timeout)? {
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
        if !poll(&self.device, libc::POLLOUT, self.write_timeout)? {
            return Err(io::ErrorKind::TimedOut.into());
        }
        loop {
            let result = check_isize(unsafe {
                libc::write(
                    self.device.as_raw_fd(),
                    buf.as_ptr().cast(),
                    buf.len() as _,
                )
            });
            match result {
                Err(ref e) if e.raw_os_error() == Some(libc::EINTR) => continue,
                x => return x,
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        check(unsafe { libc::tcdrain(self.device.as_raw_fd()) })?;
        Ok(())
    }
}
