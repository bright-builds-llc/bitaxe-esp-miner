//! BM1397 one/four-midstate work payload and job-frame encoding.

use crate::bm1397::{
    protocol::{job_frame, Bm1397FrameBytes, CMD_WRITE, GROUP_SINGLE, JOB_HEADER_TYPE},
    Bm1397ProtocolFault,
};

pub const BM1397_JOB_PAYLOAD_LEN: usize = 146;
pub const BM1397_JOB_FRAME_LEN: usize = 152;
pub const JOB_ID_STEP: u8 = 4;
pub const JOB_ID_MODULUS: u8 = 128;

/// Job identifier used in transmitted BM1397 work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bm1397JobId(u8);

impl Bm1397JobId {
    #[must_use]
    pub const fn new(raw: u8) -> Self {
        Self(raw % JOB_ID_MODULUS)
    }

    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn advance(self) -> Self {
        Self::new((self.0 + JOB_ID_STEP) % JOB_ID_MODULUS)
    }
}

/// Closed one-or-four-midstate work representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1397Midstates {
    One([u8; 32]),
    Four([[u8; 32]; 4]),
}

impl Bm1397Midstates {
    #[must_use]
    pub const fn count(self) -> u8 {
        match self {
            Self::One(_) => 1,
            Self::Four(_) => 4,
        }
    }
}

/// Pool-derived work fields encoded into one BM1397 job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1397WorkFields {
    pub starting_nonce: [u8; 4],
    pub nbits: [u8; 4],
    pub ntime: [u8; 4],
    pub merkle4: [u8; 4],
    pub midstates: Bm1397Midstates,
}

/// Fixed-layout BM1397 work payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1397WorkPayload([u8; BM1397_JOB_PAYLOAD_LEN]);

impl Bm1397WorkPayload {
    #[must_use]
    pub fn new(job_id: Bm1397JobId, fields: Bm1397WorkFields) -> Self {
        let mut bytes = [0; BM1397_JOB_PAYLOAD_LEN];
        bytes[0] = job_id.raw();
        bytes[1] = fields.midstates.count();
        bytes[2..6].copy_from_slice(&fields.starting_nonce);
        bytes[6..10].copy_from_slice(&fields.nbits);
        bytes[10..14].copy_from_slice(&fields.ntime);
        bytes[14..18].copy_from_slice(&fields.merkle4);

        match fields.midstates {
            Bm1397Midstates::One(midstate) => bytes[18..50].copy_from_slice(&midstate),
            Bm1397Midstates::Four(midstates) => {
                for (index, midstate) in midstates.iter().enumerate() {
                    let start = 18 + index * 32;
                    bytes[start..start + 32].copy_from_slice(midstate);
                }
            }
        }
        Self(bytes)
    }

    #[must_use]
    pub const fn bytes(&self) -> &[u8; BM1397_JOB_PAYLOAD_LEN] {
        &self.0
    }
}

/// Encodes one complete BM1397 work frame.
pub fn encode_work_frame(
    job_id: Bm1397JobId,
    fields: Bm1397WorkFields,
) -> Result<Bm1397FrameBytes, Bm1397ProtocolFault> {
    let payload = Bm1397WorkPayload::new(job_id, fields);
    let frame = job_frame(JOB_HEADER_TYPE | GROUP_SINGLE | CMD_WRITE, payload.bytes())?;
    if frame.as_slice().len() != BM1397_JOB_FRAME_LEN {
        return Err(Bm1397ProtocolFault::InvalidLength {
            expected: BM1397_JOB_FRAME_LEN,
            actual: frame.as_slice().len(),
        });
    }
    Ok(frame)
}
