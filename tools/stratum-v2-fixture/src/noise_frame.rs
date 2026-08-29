use std::io::Read;
use std::net::TcpStream;

use anyhow::{Context, Result};
use bitaxe_stratum::v2::frame::{Frame, FrameHeader, FRAME_HEADER_LEN};
use noise_sv2::{NoiseCodec, AEAD_MAC_LEN};

#[cfg(test)]
pub(super) const DIAGNOSTIC_PROOF_EXTENSION: u16 = 0xffff;
#[cfg(test)]
pub(super) const DIAGNOSTIC_PROOF_MESSAGE: u8 = 0xff;
pub(super) const ENCRYPTED_HEADER_LEN: usize = FRAME_HEADER_LEN + AEAD_MAC_LEN;

#[cfg(test)]
pub(super) fn read_client_proof(stream: &mut TcpStream, codec: &mut NoiseCodec) -> Result<()> {
    let proof = read_noise_frame(stream, codec)?;
    if proof.header.extension_type != DIAGNOSTIC_PROOF_EXTENSION
        || proof.header.message_type != DIAGNOSTIC_PROOF_MESSAGE
        || !proof.payload().is_empty()
    {
        anyhow::bail!("encrypted diagnostic proof mismatch");
    }
    Ok(())
}

pub(super) fn read_noise_frame(stream: &mut TcpStream, codec: &mut NoiseCodec) -> Result<Frame> {
    let mut encrypted_header = vec![0; ENCRYPTED_HEADER_LEN];
    stream
        .read_exact(&mut encrypted_header)
        .context("read encrypted header")?;
    codec
        .decrypt(&mut encrypted_header)
        .map_err(|_| anyhow::anyhow!("decrypt frame header"))?;
    let header = FrameHeader::parse(&encrypted_header)?;
    let mut payload = if header.payload_len == 0 {
        Vec::new()
    } else {
        let mut encrypted = vec![0; header.payload_len + AEAD_MAC_LEN];
        stream
            .read_exact(&mut encrypted)
            .context("read encrypted payload")?;
        codec
            .decrypt(&mut encrypted)
            .map_err(|_| anyhow::anyhow!("decrypt frame payload"))?;
        encrypted
    };
    Ok(Frame::new(
        header.extension_type,
        header.message_type,
        std::mem::take(&mut payload),
    )?)
}
