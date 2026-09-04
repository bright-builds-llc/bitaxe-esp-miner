use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;

use anyhow::{Context, Result};

pub(crate) struct ReceiveOnlyReader {
    file: File,
    port: String,
}

impl ReceiveOnlyReader {
    pub(crate) fn open(port: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc_flags())
            .open(port)
            .context("receive-only serial open failed")?;
        configure_serial(&file)?;
        Ok(Self {
            file,
            port: port.to_owned(),
        })
    }

    pub(crate) fn read_available(&mut self) -> Result<Vec<u8>> {
        let mut collected = Vec::new();
        loop {
            let mut buffer = [0_u8; 4096];
            match self.file.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => collected.extend_from_slice(&buffer[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error).context("receive-only serial read failed"),
            }
        }
        Ok(collected)
    }

    pub(crate) fn port(&self) -> &str {
        &self.port
    }
}

fn configure_serial(file: &File) -> Result<()> {
    let fd = file.as_raw_fd();
    let mut terminal = unsafe { std::mem::zeroed::<libc::termios>() };
    if unsafe { libc::tcgetattr(fd, &mut terminal) } != 0 {
        return Err(io::Error::last_os_error()).context("receive-only serial attributes failed");
    }
    unsafe { libc::cfmakeraw(&mut terminal) };
    if unsafe { libc::cfsetspeed(&mut terminal, libc::B115200) } != 0 {
        return Err(io::Error::last_os_error()).context("receive-only serial speed failed");
    }
    terminal.c_cflag |= libc::CLOCAL | libc::CREAD;
    terminal.c_cflag &= !libc::HUPCL;
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &terminal) } != 0 {
        return Err(io::Error::last_os_error()).context("receive-only serial setup failed");
    }
    Ok(())
}

const fn libc_flags() -> i32 {
    // Values are stable Darwin ABI constants: O_NOCTTY and O_NONBLOCK.
    0x0002_0000 | 0x0000_0004
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use std::io::Write;
    use std::os::fd::FromRawFd;
    use std::ptr;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn reader_delivers_binary_bytes_without_a_newline() {
        // Arrange
        let mut master_fd = -1;
        let mut slave_fd = -1;
        let mut slave_name = [0_i8; 1024];
        let opened = unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                slave_name.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        assert_eq!(opened, 0, "pseudo-terminal pair must open");
        let slave_path = unsafe { CStr::from_ptr(slave_name.as_ptr()) }
            .to_str()
            .expect("pseudo-terminal path must be UTF-8")
            .to_owned();
        assert_eq!(unsafe { libc::close(slave_fd) }, 0);
        let mut master = unsafe { File::from_raw_fd(master_fd) };
        let mut reader = ReceiveOnlyReader::open(&slave_path).expect("reader must open");
        let payload = [0x00, 0xa5, 0x7f];

        // Act
        master.write_all(&payload).expect("payload must be written");
        thread::sleep(Duration::from_millis(10));
        let observed = reader.read_available().expect("payload must be read");

        // Assert
        assert_eq!(observed, payload);
    }

    #[test]
    fn reader_disables_hangup_on_close() {
        // Arrange
        let mut master_fd = -1;
        let mut slave_fd = -1;
        let mut slave_name = [0_i8; 1024];
        let opened = unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                slave_name.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        assert_eq!(opened, 0, "pseudo-terminal pair must open");
        let slave_path = unsafe { CStr::from_ptr(slave_name.as_ptr()) }
            .to_str()
            .expect("pseudo-terminal path must be UTF-8");
        assert_eq!(unsafe { libc::close(slave_fd) }, 0);
        let _master = unsafe { File::from_raw_fd(master_fd) };

        // Act
        let reader = ReceiveOnlyReader::open(slave_path).expect("reader must open");
        let mut terminal = unsafe { std::mem::zeroed::<libc::termios>() };
        assert_eq!(
            unsafe { libc::tcgetattr(reader.file.as_raw_fd(), &mut terminal) },
            0
        );

        // Assert
        assert_eq!(terminal.c_cflag & libc::HUPCL, 0);
    }

    #[test]
    fn reader_source_cannot_write_or_change_modem_control_lines() {
        // Arrange
        let source = include_str!("receive_only.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");

        // Act / Assert
        for forbidden in [".write(", "write_all", "TIOCM", "ioctl(", "B1200"] {
            assert!(
                !source.contains(forbidden),
                "receive-only source contains forbidden operation {forbidden}"
            );
        }
    }
}
