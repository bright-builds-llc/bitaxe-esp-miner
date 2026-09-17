//! Bounded authenticated frame I/O shared by channel and ordinary mining owners.
use super::{
    frame::Frame,
    noise::diagnostic::{Failure, Observer, Operation},
    noise::{NoiseTransport, ENCRYPTED_HEADER_LEN},
    session::SessionConfig,
    standard::{StandardEvent, StandardSession},
};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

pub trait ChannelObserver: Observer {
    fn validated_frame(&mut self, _frame: &Frame) -> Result<(), Failure> {
        Ok(())
    }
    fn protocol_event(&mut self, event: &StandardEvent) -> Result<(), Failure>;
}

/// Runs only setup/channel/job conversion. This API has no ASIC or submit input.
#[inline(never)]
pub fn run_channel(
    stream: &mut TcpStream,
    noise: &mut NoiseTransport,
    config: SessionConfig,
    observer: &mut impl ChannelObserver,
) -> Result<(), Failure> {
    let mut slot = Vec::new();
    slot.try_reserve_exact(1).map_err(|_| Failure::Allocation)?;
    initialize_session(&mut slot, config)?;
    let session = slot.first_mut().ok_or(Failure::Io)?;
    let setup = session.start().map_err(|_| Failure::Io)?;
    write_frame(stream, noise, &setup, observer)?;
    loop {
        let frame = read_frame(stream, noise, observer)?;
        let events = session.receive(&frame).map_err(|_| Failure::Io)?;
        observer.validated_frame(&frame)?;
        for event in events {
            observer.protocol_event(&event)?;
            match event {
                StandardEvent::Send(frame) => write_frame(stream, noise, &frame, observer)?,
                StandardEvent::Work { .. } => return Ok(()),
                _ => {}
            }
        }
    }
}
#[inline(never)]
fn initialize_session(
    slot: &mut Vec<StandardSession>,
    config: SessionConfig,
) -> Result<(), Failure> {
    slot.push(StandardSession::new(config).map_err(|_| Failure::Io)?);
    Ok(())
}
fn now(observer: &impl Observer) -> Result<u64, Failure> {
    observer.now_us().ok_or(Failure::Clock)
}
fn check(observer: &mut impl Observer) -> Result<(), Failure> {
    if observer.permitted() {
        Ok(())
    } else {
        Err(Failure::Cancelled)
    }
}
fn deadline(observer: &mut impl Observer, start: u64, limit: u64) -> Result<(), Failure> {
    check(observer)?;
    if now(observer)?.checked_sub(start).ok_or(Failure::Clock)? >= limit {
        Err(Failure::Timeout)
    } else {
        Ok(())
    }
}
fn retry(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
    )
}

/// Encrypts before beginning the separately observed write. Every I/O checkpoint
/// rechecks authority; queue acceptance is never a completion observation.
pub fn write_frame(
    stream: &mut TcpStream,
    noise: &mut NoiseTransport,
    frame: &Frame,
    observer: &mut impl Observer,
) -> Result<(), Failure> {
    check(observer)?;
    observer.operation_started(Operation::FrameEncrypt);
    let encoded = noise.encrypt_frame(frame);
    observer.operation_finished(Operation::FrameEncrypt, encoded.is_err());
    let encoded = encoded.map_err(|_| Failure::Io)?;
    check(observer)?;
    observer.operation_started(Operation::FrameWrite);
    let result = write_bytes(stream, &encoded, observer);
    observer.operation_finished(Operation::FrameWrite, result.is_err());
    result
}
fn write_bytes(
    stream: &mut TcpStream,
    bytes: &[u8],
    observer: &mut impl Observer,
) -> Result<(), Failure> {
    let start = now(observer)?;
    let mut offset = 0;
    while offset < bytes.len() {
        deadline(observer, start, 2_000_000)?;
        match stream.write(&bytes[offset..]) {
            Ok(0) => return Err(Failure::Eof),
            Ok(count) => offset += count,
            Err(e) if retry(&e) => std::thread::sleep(Duration::from_millis(2)),
            Err(_) => return Err(Failure::Io),
        }
    }
    loop {
        deadline(observer, start, 2_000_000)?;
        match stream.flush() {
            Ok(()) => {
                observer.frame_write_completed();
                return deadline(observer, start, 2_000_000);
            }
            Err(e) if retry(&e) => std::thread::sleep(Duration::from_millis(2)),
            Err(_) => return Err(Failure::Io),
        }
    }
}
/// Idle waiting is bounded by the owner's absolute authority, while the first
/// observed header byte starts one10-second header+payload completion deadline.
pub fn read_frame(
    stream: &mut TcpStream,
    noise: &mut NoiseTransport,
    observer: &mut impl Observer,
) -> Result<Frame, Failure> {
    let mut header = [0u8; ENCRYPTED_HEADER_LEN];
    let start = now(observer)?;
    observer.operation_started(Operation::FrameRead);
    let result = (|| {
        loop {
            deadline(observer, start, 10_000_000)?;
            match stream.read(&mut header[..1]) {
                Ok(0) => return Err(Failure::Eof),
                Ok(_) => break,
                Err(e) if retry(&e) => std::thread::sleep(Duration::from_millis(2)),
                Err(_) => return Err(Failure::Io),
            }
        }
        finish_frame(stream, noise, observer, start, &mut header)
    })();
    observer.operation_finished(Operation::FrameRead, result.is_err());
    result
}

fn finish_frame(
    stream: &mut TcpStream,
    noise: &mut NoiseTransport,
    observer: &mut impl Observer,
    start: u64,
    header: &mut [u8; ENCRYPTED_HEADER_LEN],
) -> Result<Frame, Failure> {
    read_remaining(stream, &mut header[1..], observer, start)?;
    observer.operation_started(Operation::HeaderDecrypt);
    let pending = noise.decrypt_header(header);
    observer.operation_finished(Operation::HeaderDecrypt, pending.is_err());
    let pending = pending.map_err(|_| Failure::Io)?;
    deadline(observer, start, 10_000_000)?;
    let length = pending.encrypted_payload_len();
    if length > 2064 {
        return Err(Failure::Extra);
    }
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(length)
        .map_err(|_| Failure::Allocation)?;
    payload.resize(length, 0);
    read_remaining(stream, &mut payload, observer, start)?;
    observer.operation_started(Operation::PayloadDecrypt);
    let frame = noise.decrypt_payload(pending, &payload);
    observer.operation_finished(Operation::PayloadDecrypt, frame.is_err());
    let frame = frame.map_err(|_| Failure::Io)?;
    // An opaque decrypt may finish after revocation. Preserve that actual
    // observation without allowing another read, write, or work transition.
    observer.frame_authenticated(&frame)?;
    deadline(observer, start, 10_000_000)?;
    Ok(frame)
}
fn read_remaining(
    stream: &mut TcpStream,
    bytes: &mut [u8],
    observer: &mut impl Observer,
    start: u64,
) -> Result<(), Failure> {
    let mut offset = 0;
    while offset < bytes.len() {
        deadline(observer, start, 10_000_000)?;
        match stream.read(&mut bytes[offset..]) {
            Ok(0) => return Err(Failure::Eof),
            Ok(count) => offset += count,
            Err(e) if retry(&e) => std::thread::sleep(Duration::from_millis(2)),
            Err(_) => return Err(Failure::Io),
        }
    }
    deadline(observer, start, 10_000_000)
}

/// The ordinary owner polls this between outbound commands. No frame timer is
/// started until an actual header byte exists; authority still bounds idle time.
pub fn maybe_read_frame(
    stream: &mut TcpStream,
    noise: &mut NoiseTransport,
    observer: &mut impl Observer,
) -> Result<Option<Frame>, Failure> {
    check(observer)?;
    match stream.peek(&mut [0u8; 1]) {
        Ok(0) => Err(Failure::Eof),
        Ok(_) => read_frame(stream, noise, observer).map(Some),
        Err(e) if retry(&e) => Ok(None),
        Err(_) => Err(Failure::Io),
    }
}

#[cfg(test)]
mod tests;
