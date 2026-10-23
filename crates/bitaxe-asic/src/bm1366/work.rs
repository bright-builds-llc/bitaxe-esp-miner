//! BM1366 diagnostic work payload encoding.
//!
//! Reference breadcrumb: `reference/esp-miner/components/asic/bm1366.c:BM1366_send_work`
//! and `reference/esp-miner/components/asic/include/bm1366.h:BM1366_job`,
//! parity checklist rows `ASIC-002`, `ASIC-003`, and `ASIC-008`.

use crate::Bm1366ProtocolFault;

use super::packet::{JobFrame, CMD_WRITE, GROUP_SINGLE, JOB_HEADER_TYPE};

pub use crate::asic::{
    JobId as Bm1366JobId, WorkFields as Bm1366WorkFields, WorkPayload as Bm1366WorkPayload,
    JOB_ID_MODULUS, JOB_PAYLOAD_LEN as BM1366_JOB_PAYLOAD_LEN, JOB_SLOT_MASK as JOB_ID_LOOKUP_MASK,
    NUM_MIDSTATES as BM1366_NUM_MIDSTATES,
};

pub const BM1366_JOB_FRAME_LEN: usize = 88;

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
        let advanced = first.advance(crate::asic::AsicFamily::Bm1366);
        let wrapped = last.advance(crate::asic::AsicFamily::Bm1366);

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
