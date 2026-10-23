//! BM1370 job-frame encoding.
//!
//! The job payload and job ids are the chip-neutral `crate::asic` types; the BM1370's ids
//! advance by 24 (`AsicFamily::Bm1370`).
//!
//! Reference: `reference/esp-miner/components/asic/bm1370.c` (`BM1370_send_work`) and
//! `components/asic/include/bm1370.h` (`BM1370_job`).

use crate::{
    asic::{JobId, WorkFields, WorkPayload},
    bm1366::packet::{FrameBytes, JobFrame, CMD_WRITE, GROUP_SINGLE, JOB_HEADER_TYPE},
    bm1370::Bm1370ProtocolFault,
};

pub const BM1370_JOB_FRAME_LEN: usize = 88;

/// Encodes one complete BM1370 work frame.
pub fn encode_work_frame(
    job_id: JobId,
    fields: WorkFields,
) -> Result<FrameBytes, Bm1370ProtocolFault> {
    let payload = WorkPayload::new(job_id, fields);
    let frame = JobFrame::new(JOB_HEADER_TYPE | GROUP_SINGLE | CMD_WRITE, payload.bytes())
        .map_err(|_| Bm1370ProtocolFault::InvalidLength {
            expected: BM1370_JOB_FRAME_LEN,
            actual: payload.bytes().len(),
        })?
        .into_bytes();
    if frame.as_slice().len() != BM1370_JOB_FRAME_LEN {
        return Err(Bm1370ProtocolFault::InvalidLength {
            expected: BM1370_JOB_FRAME_LEN,
            actual: frame.as_slice().len(),
        });
    }
    Ok(frame)
}
