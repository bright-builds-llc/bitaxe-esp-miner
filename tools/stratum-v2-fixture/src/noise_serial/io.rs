use super::model::Cause;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub(super) fn pause(deadline: Instant, stage: &'static str) -> Result<(), Cause> {
    let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
        return Err(Cause::new(stage, "read", "timeout"));
    };
    std::thread::sleep(remaining.min(Duration::from_millis(2)));
    Ok(())
}

pub(super) fn read_exact(
    stream: &mut TcpStream,
    bytes: &mut [u8],
    deadline: Instant,
    received: &mut usize,
    stage: &'static str,
) -> Result<(), Cause> {
    while *received < bytes.len() {
        if Instant::now() >= deadline {
            return Err(Cause::new(stage, "read", "timeout"));
        }
        match stream.read(&mut bytes[*received..]) {
            Ok(0) => return Err(Cause::new(stage, "read", "eof")),
            Ok(count) => *received += count,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => pause(deadline, stage)?,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Cause::new(stage, "read", "io")),
        }
    }
    if Instant::now() >= deadline {
        return Err(Cause::new(stage, "read", "timeout"));
    }
    Ok(())
}

pub(super) fn write_all(
    stream: &mut TcpStream,
    bytes: &[u8],
    deadline: Instant,
    written: &mut usize,
) -> Result<(), Cause> {
    while *written < bytes.len() {
        if Instant::now() >= deadline {
            return Err(Cause::new("act_two_received", "write", "timeout"));
        }
        match stream.write(&bytes[*written..]) {
            Ok(0) => return Err(Cause::new("act_two_received", "write", "io")),
            Ok(count) => *written += count,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                pause(deadline, "act_two_received")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Cause::new("act_two_received", "write", "io")),
        }
    }
    if Instant::now() >= deadline {
        return Err(Cause::new("act_two_received", "write", "timeout"));
    }
    Ok(())
}
