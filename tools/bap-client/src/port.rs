//! The USB-to-UART adapter as a [`Link`]: raw 115200 8-N-1, no flow control.
//!
//! The client opens only an explicit `/dev/cu.*` adapter node. The board's own
//! native-USB node (`/dev/cu.usbmodem*`) belongs to the repository's detector, flash
//! and Gate tooling, so it is refused.

use crate::exchange::Link;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
use std::time::Instant;

/// Why an adapter path is refused before it is opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathRefusal {
    NotCalloutNode,
    BoardUsbNode,
}

/// Admits only a macOS callout node that is not the board's native USB port.
pub fn admit_adapter_path(path: &str) -> Result<(), PathRefusal> {
    let Some(name) = path.strip_prefix("/dev/cu.") else {
        return Err(PathRefusal::NotCalloutNode);
    };
    if name.is_empty() || name.contains('/') {
        return Err(PathRefusal::NotCalloutNode);
    }
    if name.starts_with("usbmodem") {
        return Err(PathRefusal::BoardUsbNode);
    }
    Ok(())
}

pub struct AdapterLink {
    file: File,
    started: Instant,
}

impl AdapterLink {
    /// Opens a character device as a raw 115200 8-N-1 line and drops stale input.
    pub fn open(path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
            .open(path)?;
        if !file.metadata()?.file_type().is_char_device() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "adapter_not_a_serial_device",
            ));
        }
        configure(&file)?;
        Ok(Self {
            file,
            started: Instant::now(),
        })
    }
}

fn configure(file: &File) -> io::Result<()> {
    let fd = file.as_raw_fd();
    // SAFETY: termios is plain data; tcgetattr fills it for this open descriptor.
    let mut terminal = unsafe { std::mem::zeroed::<libc::termios>() };
    // SAFETY: fd is open for the life of `file`, and `terminal` is a valid termios.
    unsafe {
        if libc::tcgetattr(fd, &mut terminal) != 0 {
            return Err(io::Error::last_os_error());
        }
        libc::cfmakeraw(&mut terminal);
        if libc::cfsetspeed(&mut terminal, libc::B115200) != 0 {
            return Err(io::Error::last_os_error());
        }
        terminal.c_cflag |= libc::CLOCAL | libc::CREAD;
        terminal.c_cflag &= !(libc::HUPCL | libc::CRTSCTS | libc::CSTOPB | libc::PARENB);
        if libc::tcsetattr(fd, libc::TCSANOW, &terminal) != 0 {
            return Err(io::Error::last_os_error());
        }
        if libc::tcflush(fd, libc::TCIFLUSH) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

/// Which readiness to wait for.
#[derive(Clone, Copy)]
enum Readiness {
    Read,
    Write,
}

/// Waits up to `wait_ms` for the descriptor to become ready. `select(2)`, because
/// macOS `poll(2)` does not support character devices.
fn ready(file: &File, readiness: Readiness, wait_ms: u64) -> io::Result<bool> {
    let fd = file.as_raw_fd();
    if fd >= libc::FD_SETSIZE as i32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "descriptor_out_of_range",
        ));
    }
    loop {
        // SAFETY: fd_set is plain data, initialized by FD_ZERO before use, and fd is
        // an open descriptor below FD_SETSIZE.
        let result = unsafe {
            let mut set = std::mem::zeroed::<libc::fd_set>();
            libc::FD_ZERO(&mut set);
            libc::FD_SET(fd, &mut set);
            let mut timeout = libc::timeval {
                tv_sec: libc::time_t::try_from(wait_ms / 1_000).unwrap_or(libc::time_t::MAX),
                tv_usec: libc::suseconds_t::try_from((wait_ms % 1_000) * 1_000).unwrap_or(0),
            };
            let (read, write) = match readiness {
                Readiness::Read => (&mut set as *mut libc::fd_set, std::ptr::null_mut()),
                Readiness::Write => (std::ptr::null_mut(), &mut set as *mut libc::fd_set),
            };
            libc::select(fd + 1, read, write, std::ptr::null_mut(), &mut timeout)
        };
        if result >= 0 {
            return Ok(result > 0);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

impl Link for AdapterLink {
    fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        let mut remaining = bytes;
        while !remaining.is_empty() {
            match self.file.write(remaining) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(written) => remaining = &remaining[written..],
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    if !ready(&self.file, Readiness::Write, 1_000)? {
                        return Err(io::ErrorKind::TimedOut.into());
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn receive(&mut self, wait_ms: u64) -> io::Result<Vec<u8>> {
        if !ready(&self.file, Readiness::Read, wait_ms)? {
            return Ok(Vec::new());
        }
        let mut received = Vec::new();
        let mut buffer = [0_u8; 512];
        loop {
            match self.file.read(&mut buffer) {
                // End of file means the adapter went away; that is not a quiet line.
                Ok(0) if received.is_empty() => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "adapter_hung_up",
                    ))
                }
                Ok(0) => return Ok(received),
                Ok(count) => received.extend_from_slice(&buffer[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(received),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        }
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
mod tests;
