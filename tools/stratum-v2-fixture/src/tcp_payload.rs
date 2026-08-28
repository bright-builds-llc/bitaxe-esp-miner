use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use super::FixtureProgress;

pub(super) fn read_tcp_payload(
    stream: &mut TcpStream,
    progress: &mut FixtureProgress,
) -> Result<()> {
    const EXPECTED: [u8; 64] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
    ];
    let mut payload = [0_u8; 64];
    let mut received = 0;
    while received < payload.len() {
        match stream.read(&mut payload[received..]) {
            Ok(0) => {
                progress.payload_read_category = "eof";
                progress.payload_bytes_received = received.try_into().unwrap_or(u16::MAX);
                bail!("payload ended before 64 bytes");
            }
            Ok(count) => received += count,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                progress.payload_read_category = "timeout";
                progress.payload_bytes_received = received.try_into().unwrap_or(u16::MAX);
                bail!("payload timed out before 64 bytes");
            }
            Err(error) => {
                progress.payload_read_category = "io";
                bail!("payload read failed: {error}");
            }
        }
    }
    progress.payload_bytes_received = 64;
    progress.payload_digest_match = payload == EXPECTED;
    progress.payload_read_category = if progress.payload_digest_match {
        "complete"
    } else {
        "mismatch"
    };
    if !progress.payload_digest_match {
        bail!("payload did not match fixed canary");
    }
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .context("set extra-byte timeout")?;
    let mut extra = [0_u8; 1];
    progress.extra_bytes_received = match stream.read(&mut extra) {
        Ok(0) => 0,
        Ok(count) => count.try_into().unwrap_or(u16::MAX),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            0
        }
        Err(error) => bail!("extra-byte read failed: {error}"),
    };
    if progress.extra_bytes_received != 0 {
        bail!("payload contained extra bytes");
    }
    Ok(())
}
