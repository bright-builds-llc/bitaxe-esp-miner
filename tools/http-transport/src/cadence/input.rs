//! Bounded stdin handling without a blocked reader thread or endpoint logging.
use std::time::Instant;
use zeroize::Zeroizing;

pub(super) struct PrivateInput {
    fd: i32,
    bytes: Zeroizing<Vec<u8>>,
}

impl PrivateInput {
    pub(super) fn new(fd: i32) -> Self {
        Self {
            fd,
            bytes: Zeroizing::new(Vec::with_capacity(1024)),
        }
    }

    pub(super) fn initial_line(
        &mut self,
        deadline: Instant,
    ) -> Result<Zeroizing<Vec<u8>>, &'static str> {
        loop {
            if Instant::now() >= deadline {
                return Err("input_timeout");
            }
            match self.read_byte(50)? {
                Some(Some(b'\n')) => {
                    return Ok(std::mem::replace(
                        &mut self.bytes,
                        Zeroizing::new(Vec::with_capacity(64)),
                    ))
                }
                Some(Some(byte)) => self.push(byte, 1024)?,
                Some(None) => return Err("input_closed"),
                None => {}
            }
        }
    }

    pub(super) fn stop_requested(&mut self) -> Result<bool, &'static str> {
        while let Some(maybe_byte) = self.read_byte(0)? {
            match maybe_byte {
                None => {
                    return if self.bytes.is_empty() {
                        Ok(true)
                    } else {
                        Err("invalid_stop")
                    }
                }
                Some(b'\n') => {
                    #[derive(serde::Deserialize)]
                    #[serde(deny_unknown_fields)]
                    struct Stop<'a> {
                        op: &'a str,
                    }
                    let stop: Stop<'_> =
                        serde_json::from_slice(&self.bytes).map_err(|_| "invalid_stop")?;
                    return if stop.op == "stop" {
                        Ok(true)
                    } else {
                        Err("invalid_stop")
                    };
                }
                Some(byte) => self.push(byte, 64)?,
            }
        }
        Ok(false)
    }

    fn push(&mut self, byte: u8, limit: usize) -> Result<(), &'static str> {
        if self.bytes.len() >= limit {
            return Err("input_bound");
        }
        self.bytes.push(byte);
        Ok(())
    }

    fn read_byte(&self, timeout_ms: i32) -> Result<Option<Option<u8>>, &'static str> {
        let mut descriptor = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // The descriptor and byte buffer remain valid throughout their synchronous calls.
        let polled = unsafe { libc::poll(&mut descriptor, 1, timeout_ms) };
        if polled < 0 {
            return if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                Ok(None)
            } else {
                Err("input_failed")
            };
        }
        if polled == 0 {
            return Ok(None);
        }
        if descriptor.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
            return Err("input_failed");
        }
        let mut byte = 0_u8;
        // No other reader shares this owned stdin; poll established readability or EOF.
        match unsafe { libc::read(self.fd, (&mut byte as *mut u8).cast(), 1) } {
            0 => Ok(Some(None)),
            1 => Ok(Some(Some(byte))),
            _ => Err("input_failed"),
        }
    }
}
