//! Heap-backed diagnostic completion without a large by-value transport result.
use super::*;

impl NoiseInitiator {
    /// Completes into a caller-reserved slot; no allocation occurs at the push.
    /// The caller must reserve fallibly before entering opaque crypto.
    #[inline(never)]
    pub fn complete_diagnostic_into(
        mut self,
        act_two: &[u8; ACT_TWO_LEN],
        unix_time_seconds: u32,
        output: &mut Vec<NoiseTransport>,
    ) -> Result<(), NoiseCompletionFailure> {
        if !output.is_empty() || output.capacity() == 0 || !self.act_one_sent {
            return Err(NoiseCompletionFailure::State);
        }
        let mut inner = self.inner.take().ok_or(NoiseCompletionFailure::State)?;
        store_result(
            output,
            inner.step_2_with_now(*act_two, unix_time_seconds),
            unix_time_seconds,
        )
    }
}

// This construction scratch must not remain live while SRI's verifier runs.
#[inline(never)]
fn store_result(
    output: &mut Vec<NoiseTransport>,
    result: Result<NoiseCodec, noise_sv2::Error>,
    unix_time_seconds: u32,
) -> Result<(), NoiseCompletionFailure> {
    let codec = result.map_err(|error| classify_completion_error(error, unix_time_seconds))?;
    output.push(NoiseTransport {
        codec,
        send_budget: NonceBudget::new(),
        receive_budget: NonceBudget::new(),
        maybe_pending_header: None,
        poisoned: false,
    });
    Ok(())
}
