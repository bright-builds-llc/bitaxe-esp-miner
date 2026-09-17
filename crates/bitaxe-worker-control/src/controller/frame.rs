use super::*;

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub fn prepare_frame(
        &mut self,
        frame: &[u8],
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        self.session.noise_poll();
        let parsed = parse(frame);
        // Retained diagnostics have no effect authority. They remain readable
        // while independent native revocation and ordinary cleanup are pending.
        let (json_len, maybe_request) = match parsed {
            Ok((_, Some(request))) if request.command == "stratum_v2_status" => {
                return self.prepare_v2(&request, now)
            }
            Ok((_, None)) if self.pending_v2_cleanup() => {
                return self.prepare_possession(frame, now)
            }
            other => {
                self.enforce_clock(now)?;
                other?
            }
        };
        let Some(request) = maybe_request else {
            return self.prepare_possession(frame, now);
        };
        self.acknowledge_boot_restoration()?;
        let is_probe = request.command == "transport_probe";
        let prepared = self.prepare_controller(request, now)?;
        if is_probe {
            self.session
                .telemetry_cadence_probe_prepared(json_len, prepared.frame.len() - 1);
        }
        Ok(prepared)
    }
}
fn parse(frame: &[u8]) -> Result<(usize, Option<ControllerRequest>), WorkerControlError> {
    let json = strict_json_frame(frame).map_err(|_| WorkerControlError::InvalidFrame)?;
    let discriminator: FrameDiscriminator =
        serde_json::from_str(json).map_err(classify_json_error)?;
    if discriminator.profile.is_some() {
        return Ok((json.len(), None));
    }
    let request: ControllerRequest =
        serde_json::from_str(json).map_err(|_| WorkerControlError::InvalidRequest)?;
    request.validate()?;
    Ok((json.len(), Some(request)))
}
