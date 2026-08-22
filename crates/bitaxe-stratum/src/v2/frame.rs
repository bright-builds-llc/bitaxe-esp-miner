use super::{StratumV2Error, MAX_FRAME_PAYLOAD};

pub const FRAME_HEADER_LEN: usize = 6;
const MAX_U24: usize = 0x00ff_ffff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub extension_type: u16,
    pub message_type: u8,
    pub payload_len: usize,
}

impl FrameHeader {
    pub fn new(
        extension_type: u16,
        message_type: u8,
        payload_len: usize,
    ) -> Result<Self, StratumV2Error> {
        if payload_len > MAX_FRAME_PAYLOAD || payload_len > MAX_U24 {
            return Err(StratumV2Error::PayloadTooLarge {
                actual: payload_len,
                maximum: MAX_FRAME_PAYLOAD,
            });
        }
        Ok(Self {
            extension_type,
            message_type,
            payload_len,
        })
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, StratumV2Error> {
        if bytes.len() < FRAME_HEADER_LEN {
            return Err(StratumV2Error::TruncatedHeader);
        }
        let extension_type = u16::from_le_bytes([bytes[0], bytes[1]]);
        let message_type = bytes[2];
        let payload_len =
            usize::from(bytes[3]) | (usize::from(bytes[4]) << 8) | (usize::from(bytes[5]) << 16);
        Self::new(extension_type, message_type, payload_len)
    }

    #[must_use]
    pub fn encode(self) -> [u8; FRAME_HEADER_LEN] {
        let mut bytes = [0; FRAME_HEADER_LEN];
        bytes[0..2].copy_from_slice(&self.extension_type.to_le_bytes());
        bytes[2] = self.message_type;
        bytes[3] = self.payload_len as u8;
        bytes[4] = (self.payload_len >> 8) as u8;
        bytes[5] = (self.payload_len >> 16) as u8;
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub header: FrameHeader,
    payload: Vec<u8>,
}

impl Frame {
    pub fn new(
        extension_type: u16,
        message_type: u8,
        payload: Vec<u8>,
    ) -> Result<Self, StratumV2Error> {
        let header = FrameHeader::new(extension_type, message_type, payload.len())?;
        Ok(Self { header, payload })
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, StratumV2Error> {
        let header = FrameHeader::parse(bytes)?;
        let expected = FRAME_HEADER_LEN.checked_add(header.payload_len).ok_or(
            StratumV2Error::PayloadTooLarge {
                actual: header.payload_len,
                maximum: MAX_FRAME_PAYLOAD,
            },
        )?;
        if bytes.len() != expected {
            return Err(StratumV2Error::FrameLengthMismatch {
                expected,
                actual: bytes.len(),
            });
        }
        Ok(Self {
            header,
            payload: bytes[FRAME_HEADER_LEN..].to_vec(),
        })
    }

    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FRAME_HEADER_LEN + self.payload.len());
        bytes.extend_from_slice(&self.header.encode());
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_header_matches_pinned_six_byte_layout() {
        // Arrange
        let header = FrameHeader::new(0x8000, 0x1a, 0x000203).expect("bounded header");

        // Act
        let encoded = header.encode();
        let decoded = FrameHeader::parse(&encoded);

        // Assert
        assert_eq!(encoded, [0x00, 0x80, 0x1a, 0x03, 0x02, 0x00]);
        assert_eq!(decoded, Ok(header));
    }

    #[test]
    fn frame_rejects_truncation_trailing_bytes_and_oversized_payload() {
        // Arrange
        let valid = Frame::new(0, 1, vec![1, 2, 3])
            .expect("valid frame")
            .encode();
        let mut trailing = valid.clone();
        trailing.push(4);
        let oversized = vec![0; MAX_FRAME_PAYLOAD + 1];

        // Act
        let truncated_result = Frame::parse(&valid[..valid.len() - 1]);
        let trailing_result = Frame::parse(&trailing);
        let oversized_result = Frame::new(0, 1, oversized);

        // Assert
        assert!(matches!(
            truncated_result,
            Err(StratumV2Error::FrameLengthMismatch { .. })
        ));
        assert!(matches!(
            trailing_result,
            Err(StratumV2Error::FrameLengthMismatch { .. })
        ));
        assert!(matches!(
            oversized_result,
            Err(StratumV2Error::PayloadTooLarge { .. })
        ));
    }
}
