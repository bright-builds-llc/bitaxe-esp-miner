//! Independent wire checks and immutable public nonce facts for functional tests.
use crate::oracle;

/// Declared synthetic lease terms shared by signer and functional endpoint.
pub const SYNTHETIC_ENDPOINT_HOST: &str = "192.168.0.2";
pub const SYNTHETIC_ENDPOINT_PORT: u16 = 3333;
pub const SYNTHETIC_USER_IDENTITY: &str = "synthetic";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("fixture oracle rejected {0}")]
pub struct OracleError(pub &'static str);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Submission {
    pub channel_id: u32,
    pub sequence: u32,
    pub job_id: u32,
    pub nonce: u32,
    pub ntime: u32,
    pub version: u32,
}
#[derive(Debug, Clone)]
pub struct JobFacts {
    pub channel: u32,
    pub job: u32,
    pub previous: [u8; 32],
    pub merkle: [u8; 32],
    pub ntime: u32,
    pub version: u32,
    pub nbits: u32,
    pub nonce: u32,
    pub target: [u8; 32],
}
impl JobFacts {
    /// Public Bitcoin genesis vector; not the strict live regtest job profile.
    pub fn genesis() -> Self {
        let merkle = [
            0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76,
            0x8f, 0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa,
            0x4b, 0x1e, 0x5e, 0x4a,
        ];
        Self {
            channel: 9,
            job: 7,
            previous: [0; 32],
            merkle,
            ntime: 1231006505,
            version: 1,
            nbits: 0x1d00ffff,
            nonce: 2083236893,
            target: oracle::TARGET,
        }
    }
    pub fn header(&self, nonce: u32) -> [u8; 80] {
        oracle::header(
            self.version,
            self.previous,
            self.merkle,
            self.ntime,
            self.nbits,
            nonce,
        )
    }
    pub fn validate_submission(
        &self,
        encoded: &[u8],
    ) -> Result<(Submission, [u8; 32]), OracleError> {
        let payload = payload(encoded, 0x8000, 0x1a)?;
        if payload.len() != 24 {
            return Err(OracleError("submission length"));
        }
        let value = Submission {
            channel_id: u32_at(payload, 0)?,
            sequence: u32_at(payload, 4)?,
            job_id: u32_at(payload, 8)?,
            nonce: u32_at(payload, 12)?,
            ntime: u32_at(payload, 16)?,
            version: u32_at(payload, 20)?,
        };
        if value.channel_id != self.channel
            || value.job_id != self.job
            || value.ntime != self.ntime
            || value.version != self.version
        {
            return Err(OracleError("submission context"));
        }
        let hash = oracle::hash(&self.header(value.nonce));
        if !oracle::meets_target(&hash, &self.target) {
            return Err(OracleError("submission target"));
        }
        Ok((value, hash))
    }
    /// Independently builds an acknowledgement only after proof was checked.
    pub fn acknowledgement(&self, value: Submission) -> Result<Vec<u8>, OracleError> {
        if value.channel_id != self.channel
            || value.job_id != self.job
            || value.ntime != self.ntime
            || value.version != self.version
            || !oracle::meets_target(&oracle::hash(&self.header(value.nonce)), &self.target)
        {
            return Err(OracleError("unvalidated ACK"));
        }
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.channel.to_le_bytes());
        payload.extend_from_slice(&value.sequence.to_le_bytes());
        payload.extend_from_slice(&1_u32.to_le_bytes());
        payload.extend_from_slice(&1024_u64.to_le_bytes());
        let mut result = vec![0, 0x80, 0x1c, 20, 0, 0];
        result.extend(payload);
        Ok(result)
    }
}
/// Checks ACK fields independently, including completed submission correlation.
pub fn validate_ack(
    encoded: &[u8],
    submission: Submission,
    written: bool,
) -> Result<(), OracleError> {
    let data = payload(encoded, 0x8000, 0x1c)?;
    if data.len() != 20
        || !written
        || u32_at(data, 0)? != submission.channel_id
        || u32_at(data, 4)? != submission.sequence
        || u32_at(data, 8)? != 1
        || data[12..20] != 1024_u64.to_le_bytes()
    {
        return Err(OracleError("ACK correlation"));
    }
    Ok(())
}
/// Checks generic production SetupConnection values using an independent cursor.
pub fn validate_setup(encoded: &[u8]) -> Result<(), OracleError> {
    let data = payload(encoded, 0, 0)?;
    let mut cursor = Cursor { data, offset: 0 };
    if cursor.take(5)? != [0, 2, 0, 2, 0]
        || cursor.take(4)? != 1_u32.to_le_bytes()
        || cursor.string()? != SYNTHETIC_ENDPOINT_HOST.as_bytes()
        || cursor.take(2)? != SYNTHETIC_ENDPOINT_PORT.to_le_bytes()
    {
        return Err(OracleError("setup facts"));
    }
    for expected in [b"synthetic".as_slice(), b"205", b"virtual-v1", b"synthetic"] {
        if cursor.string()? != expected {
            return Err(OracleError("setup identity"));
        }
    }
    if cursor.offset != data.len() {
        return Err(OracleError("setup trailing bytes"));
    }
    Ok(())
}
pub fn validate_open(encoded: &[u8]) -> Result<(), OracleError> {
    let data = payload(encoded, 0, 0x10)?;
    let mut cursor = Cursor { data, offset: 0 };
    if cursor.take(4)? != 1_u32.to_le_bytes()
        || cursor.string()? != SYNTHETIC_USER_IDENTITY.as_bytes()
    {
        return Err(OracleError("open identity"));
    }
    let rate = f32::from_bits(u32::from_le_bytes(
        cursor
            .take(4)?
            .try_into()
            .map_err(|_| OracleError("rate"))?,
    ));
    if !rate.is_finite()
        || rate <= 0.0
        || cursor.take(32)? != [0xff; 32]
        || cursor.offset != data.len()
    {
        return Err(OracleError("open facts"));
    }
    Ok(())
}
fn payload(encoded: &[u8], extension: u16, message: u8) -> Result<&[u8], OracleError> {
    if encoded.len() < 6 || encoded[0..2] != extension.to_le_bytes() || encoded[2] != message {
        return Err(OracleError("frame header"));
    }
    let length =
        usize::from(encoded[3]) | (usize::from(encoded[4]) << 8) | (usize::from(encoded[5]) << 16);
    if length != encoded.len() - 6 {
        return Err(OracleError("frame length"));
    }
    Ok(&encoded[6..])
}
fn u32_at(data: &[u8], at: usize) -> Result<u32, OracleError> {
    Ok(u32::from_le_bytes(
        data.get(at..at + 4)
            .ok_or(OracleError("u32 span"))?
            .try_into()
            .map_err(|_| OracleError("u32"))?,
    ))
}
struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, size: usize) -> Result<&'a [u8], OracleError> {
        let end = self
            .offset
            .checked_add(size)
            .ok_or(OracleError("cursor overflow"))?;
        let value = self
            .data
            .get(self.offset..end)
            .ok_or(OracleError("cursor span"))?;
        self.offset = end;
        Ok(value)
    }
    fn string(&mut self) -> Result<&'a [u8], OracleError> {
        let length = usize::from(self.take(1)?[0]);
        self.take(length)
    }
}
