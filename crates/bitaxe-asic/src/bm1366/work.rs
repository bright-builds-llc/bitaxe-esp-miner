//! BM1366 diagnostic work payload encoding.
//!
//! Reference breadcrumb: `reference/esp-miner/components/asic/bm1366.c:BM1366_send_work`
//! and `reference/esp-miner/components/asic/include/bm1366.h:BM1366_job`,
//! parity checklist rows `ASIC-002`, `ASIC-003`, and `ASIC-008`.

use crate::Bm1366ProtocolFault;

use super::packet::{JobFrame, CMD_WRITE, GROUP_SINGLE, JOB_HEADER_TYPE};

pub const BM1366_JOB_PAYLOAD_LEN: usize = 82;
pub const BM1366_JOB_FRAME_LEN: usize = 88;
pub const BM1366_NUM_MIDSTATES: u8 = 0x01;
pub const JOB_ID_STEP: u8 = 8;
pub const JOB_ID_MODULUS: u8 = 128;
pub const JOB_ID_LOOKUP_MASK: u8 = 0xf8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bm1366JobId {
    id: u8,
}

impl Bm1366JobId {
    pub const fn new(id: u8) -> Self {
        Self {
            id: id % JOB_ID_MODULUS,
        }
    }

    pub const fn raw(self) -> u8 {
        self.id
    }

    pub const fn advance(self) -> Self {
        Self::new((self.id + JOB_ID_STEP) % JOB_ID_MODULUS)
    }

    pub const fn lookup_key(self) -> Self {
        Self::new(self.id & JOB_ID_LOOKUP_MASK)
    }

    pub const fn small_core_id(self) -> u8 {
        self.id & !JOB_ID_LOOKUP_MASK
    }
}

impl From<Bm1366JobId> for u8 {
    fn from(job_id: Bm1366JobId) -> Self {
        job_id.raw()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366WorkFields {
    pub starting_nonce: [u8; 4],
    pub nbits: [u8; 4],
    pub ntime: [u8; 4],
    pub merkle_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub version: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366WorkPayload {
    bytes: [u8; BM1366_JOB_PAYLOAD_LEN],
}

impl Bm1366WorkPayload {
    pub fn new(job_id: Bm1366JobId, fields: Bm1366WorkFields) -> Self {
        let mut bytes = [0; BM1366_JOB_PAYLOAD_LEN];

        bytes[0] = job_id.raw();
        bytes[1] = BM1366_NUM_MIDSTATES;
        bytes[2..6].copy_from_slice(&fields.starting_nonce);
        bytes[6..10].copy_from_slice(&fields.nbits);
        bytes[10..14].copy_from_slice(&fields.ntime);
        bytes[14..46].copy_from_slice(&fields.merkle_root);
        bytes[46..78].copy_from_slice(&fields.prev_block_hash);
        bytes[78..82].copy_from_slice(&fields.version);

        Self { bytes }
    }

    pub const fn bytes(&self) -> &[u8; BM1366_JOB_PAYLOAD_LEN] {
        &self.bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticWorkFrame {
    job_id: Bm1366JobId,
    payload: Bm1366WorkPayload,
    frame: JobFrame,
}

impl DiagnosticWorkFrame {
    pub const fn job_id(&self) -> Bm1366JobId {
        self.job_id
    }

    pub const fn payload(&self) -> &Bm1366WorkPayload {
        &self.payload
    }

    pub fn bytes(&self) -> &[u8] {
        self.frame.bytes()
    }

    pub fn into_frame(self) -> JobFrame {
        self.frame
    }
}

pub fn diagnostic_job_frame(
    job_id: Bm1366JobId,
    fields: Bm1366WorkFields,
) -> Result<DiagnosticWorkFrame, Bm1366ProtocolFault> {
    let payload = Bm1366WorkPayload::new(job_id, fields);
    let header = JOB_HEADER_TYPE | GROUP_SINGLE | CMD_WRITE;
    let frame = JobFrame::new(header, payload.bytes())?;

    if frame.bytes().len() != BM1366_JOB_FRAME_LEN {
        return Err(Bm1366ProtocolFault::InvalidLength {
            expected: BM1366_JOB_FRAME_LEN,
            actual: frame.bytes().len(),
        });
    }

    Ok(DiagnosticWorkFrame {
        job_id,
        payload,
        frame,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        diagnostic_job_frame, Bm1366JobId, Bm1366WorkFields, Bm1366WorkPayload,
        BM1366_JOB_FRAME_LEN, BM1366_JOB_PAYLOAD_LEN, BM1366_NUM_MIDSTATES,
    };

    fn sample_fields() -> Bm1366WorkFields {
        Bm1366WorkFields {
            starting_nonce: [0x01, 0x02, 0x03, 0x04],
            nbits: [0x05, 0x06, 0x07, 0x08],
            ntime: [0x09, 0x0a, 0x0b, 0x0c],
            merkle_root: [0x11; 32],
            prev_block_hash: [0x22; 32],
            version: [0x33, 0x34, 0x35, 0x36],
        }
    }

    #[test]
    fn bm1366_work_job_id_advance_steps_by_eight_and_wraps() {
        // Arrange
        let first = Bm1366JobId::new(0);
        let last = Bm1366JobId::new(120);

        // Act
        let advanced = first.advance();
        let wrapped = last.advance();

        // Assert
        assert_eq!(advanced.raw(), 8);
        assert_eq!(wrapped.raw(), 0);
    }

    #[test]
    fn bm1366_work_job_id_lookup_and_small_core_use_packed_low_bits() {
        // Arrange
        let job_id = Bm1366JobId::new(0x2f);

        // Act
        let lookup_key = job_id.lookup_key();
        let small_core_id = job_id.small_core_id();

        // Assert
        assert_eq!(lookup_key.raw(), 0x28);
        assert_eq!(small_core_id, 7);
    }

    #[test]
    fn bm1366_work_payload_encodes_fixed_layout() {
        // Arrange
        let job_id = Bm1366JobId::new(0x28);
        let fields = sample_fields();

        // Act
        let payload = Bm1366WorkPayload::new(job_id, fields);
        let bytes = payload.bytes();

        // Assert
        assert_eq!(bytes.len(), BM1366_JOB_PAYLOAD_LEN);
        assert_eq!(bytes[0], 0x28);
        assert_eq!(bytes[1], BM1366_NUM_MIDSTATES);
        assert_eq!(&bytes[2..6], &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(&bytes[78..82], &[0x33, 0x34, 0x35, 0x36]);
    }

    #[test]
    fn bm1366_work_diagnostic_job_frame_wraps_payload_in_job_frame() {
        // Arrange
        let job_id = Bm1366JobId::new(0x28);
        let fields = sample_fields();

        // Act
        let frame = diagnostic_job_frame(job_id, fields).expect("diagnostic frame should encode");

        // Assert
        assert_eq!(frame.bytes().len(), BM1366_JOB_FRAME_LEN);
        assert_eq!(frame.job_id().raw(), 0x28);
        assert_eq!(frame.payload().bytes()[0], 0x28);
    }
}
