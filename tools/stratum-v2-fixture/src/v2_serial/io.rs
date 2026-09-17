use super::model::{Error, Result};
use bitaxe_stratum::v2::frame::{Frame, FrameHeader};
use noise_sv2::NoiseCodec;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use zeroize::Zeroizing;
pub(super) struct SecretFrame {
    pub header: FrameHeader,
    pub payload: Zeroizing<Vec<u8>>,
}
fn check(deadline: Instant, stage: &'static str) -> Result<()> {
    if Instant::now() >= deadline {
        Err(Error::new(stage, "timeout"))
    } else {
        Ok(())
    }
}
fn pause(deadline: Instant, stage: &'static str) -> Result<()> {
    check(deadline, stage)?;
    std::thread::sleep(
        Duration::from_millis(2).min(deadline.saturating_duration_since(Instant::now())),
    );
    Ok(())
}
pub(super) fn exact(
    stream: &mut TcpStream,
    bytes: &mut [u8],
    deadline: Instant,
    stage: &'static str,
) -> Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        check(deadline, stage)?;
        match stream.read(&mut bytes[offset..]) {
            Ok(0) => return Err(Error::new(stage, "eof")),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => pause(deadline, stage)?,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Error::new(stage, "protocol")),
        }
    }
    check(deadline, stage)
}
pub(super) fn write(
    stream: &mut TcpStream,
    bytes: &[u8],
    lifetime: Instant,
    stage: &'static str,
) -> Result<()> {
    let deadline = lifetime.min(Instant::now() + Duration::from_secs(2));
    let mut offset = 0;
    while offset < bytes.len() {
        check(deadline, stage)?;
        match stream.write(&bytes[offset..]) {
            Ok(0) => return Err(Error::new(stage, "protocol")),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => pause(deadline, stage)?,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Error::new(stage, "protocol")),
        }
    }
    stream.flush().map_err(|_| Error::new(stage, "protocol"))?;
    check(deadline, stage)
}
/// Idle wait and complete-record deadline are distinct; fragments never extend either.
pub(super) fn read_frame(
    stream: &mut TcpStream,
    codec: &mut NoiseCodec,
    lifetime: Instant,
    expected_reply: bool,
    stage: &'static str,
) -> Result<Option<SecretFrame>> {
    let idle_deadline = if expected_reply {
        lifetime.min(Instant::now() + Duration::from_secs(10))
    } else {
        lifetime
    };
    let mut header = Zeroizing::new(vec![0; 22]);
    loop {
        check(idle_deadline, stage)?;
        match stream.read(&mut header[..1]) {
            Ok(0) => {
                check(idle_deadline, stage)?;
                return Ok(None);
            }
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => pause(idle_deadline, stage)?,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Error::new(stage, "protocol")),
        }
    }
    check(idle_deadline, stage)?;
    let deadline = if expected_reply {
        idle_deadline
    } else {
        lifetime.min(Instant::now() + Duration::from_secs(10))
    };
    exact(stream, &mut header[1..], deadline, stage)?;
    codec
        .decrypt(&mut *header)
        .map_err(|_| Error::new(stage, "authentication"))?;
    let parsed = FrameHeader::parse(&header).map_err(|_| Error::new(stage, "protocol"))?;
    if parsed.payload_len > 2048 {
        return Err(Error::new(stage, "evidence"));
    }
    let mut payload = Zeroizing::new(vec![
        0;
        if parsed.payload_len == 0 {
            0
        } else {
            parsed.payload_len + 16
        }
    ]);
    if !payload.is_empty() {
        exact(stream, &mut payload, deadline, stage)?;
        codec
            .decrypt(&mut *payload)
            .map_err(|_| Error::new(stage, "authentication"))?;
    }
    check(deadline, stage)?;
    if payload.len() != parsed.payload_len {
        return Err(Error::new(stage, "protocol"));
    }
    Ok(Some(SecretFrame {
        header: parsed,
        payload,
    }))
}
pub(super) fn send_frame(
    stream: &mut TcpStream,
    codec: &mut NoiseCodec,
    frame: &Frame,
    lifetime: Instant,
    stage: &'static str,
) -> Result<()> {
    let bytes = encrypt_frame(codec, frame, stage)?;
    write(stream, &bytes, lifetime, stage)
}
pub(super) fn encrypt_frame(
    codec: &mut NoiseCodec,
    frame: &Frame,
    stage: &'static str,
) -> Result<Zeroizing<Vec<u8>>> {
    if frame.payload().len() > 2048 {
        return Err(Error::new(stage, "evidence"));
    }
    let mut bytes = Zeroizing::new(frame.header.encode().to_vec());
    codec
        .encrypt(&mut *bytes)
        .map_err(|_| Error::new(stage, "authentication"))?;
    if !frame.payload().is_empty() {
        let mut body = Zeroizing::new(frame.payload().to_vec());
        codec
            .encrypt(&mut *body)
            .map_err(|_| Error::new(stage, "authentication"))?;
        bytes.extend_from_slice(&body);
    }
    Ok(bytes)
}
