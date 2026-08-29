//! Private boot-time Stratum V2 Noise diagnostic owner.

use std::thread;
use std::time::{Duration, Instant};

use crate::settings_adapter::NoiseDiagnosticAdmission;
use crate::stratum_v2_session::transport::{
    run_noise_diagnostic, NoiseDiagnosticEvent, NoiseDiagnosticFailure,
};
use crate::stratum_v2_tcp_payload_replay::replay_deadline_ms;

const OWNER_THREAD_NAME: &str = "stratum-v2-noise-diag";
const OWNER_STACK_BYTES: usize = 24 * 1_024;
const WIFI_TIMEOUT: Duration = Duration::from_secs(60);
const WIFI_POLL: Duration = Duration::from_millis(100);
const MONITOR_ARM_DELAY: Duration = Duration::from_secs(10);

pub(crate) fn start(admission: NoiseDiagnosticAdmission) -> anyhow::Result<()> {
    thread::Builder::new()
        .name(OWNER_THREAD_NAME.to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(move || run(admission))?;
    Ok(())
}

fn run(admission: NoiseDiagnosticAdmission) {
    let mut transcript = DiagnosticTranscript::default();
    if admission.lease() == 0 {
        complete(&mut transcript, "state", false);
        return;
    }
    if !wait_for_wifi() {
        complete(&mut transcript, "connect", false);
        return;
    }
    thread::sleep(MONITOR_ARM_DELAY);
    transcript.record(NoiseDiagnosticEvent::Stage(
        crate::stratum_v2_session::transport::NoiseDiagnosticStage::MonitorArmed,
    ));
    let settings = match exact_primary_settings() {
        Ok(settings) => settings,
        Err(failure) => {
            complete(&mut transcript, failure.label(), false);
            return;
        }
    };
    match run_noise_diagnostic(settings, |event| transcript.record(event)) {
        Ok(()) => complete(&mut transcript, "accepted", true),
        Err(failure) => complete(&mut transcript, failure.label(), false),
    }
}

#[derive(Default)]
struct DiagnosticTranscript {
    events: Vec<NoiseDiagnosticEvent>,
    act_one_bytes_written: u16,
    proof_bytes_written: u16,
    terminal: Option<(&'static str, bool)>,
}

impl DiagnosticTranscript {
    fn record(&mut self, event: NoiseDiagnosticEvent) {
        match event {
            NoiseDiagnosticEvent::ActOneBytesWritten(count) => {
                self.act_one_bytes_written = count;
            }
            NoiseDiagnosticEvent::ProofBytesWritten(count) => {
                self.proof_bytes_written = count;
            }
            _ => {}
        }
        self.events.push(event);
        publish_event(event);
    }

    fn record_terminal(&mut self, category: &'static str, accepted: bool) {
        self.terminal = Some((category, accepted));
        publish_terminal(
            category,
            accepted,
            self.act_one_bytes_written,
            self.proof_bytes_written,
        );
    }

    fn replay(&self) {
        for event in &self.events {
            publish_event(*event);
        }
        if let Some((category, accepted)) = self.terminal {
            publish_terminal(
                category,
                accepted,
                self.act_one_bytes_written,
                self.proof_bytes_written,
            );
        }
    }
}

fn complete(transcript: &mut DiagnosticTranscript, category: &'static str, accepted: bool) {
    transcript.record_terminal(category, accepted);
    let replay_started = Instant::now();
    let mut ordinal = 1;
    while let Some(deadline_ms) = replay_deadline_ms(ordinal) {
        let deadline = Duration::from_millis(deadline_ms);
        let elapsed = replay_started.elapsed();
        if deadline > elapsed {
            thread::sleep(deadline - elapsed);
        }
        transcript.replay();
        ordinal = ordinal.saturating_add(1);
    }
}

fn exact_primary_settings(
) -> Result<crate::settings_adapter::V2PoolSettings, NoiseDiagnosticFailure> {
    let pool_set = crate::settings_adapter::read_stratum_v2_pool_set()
        .map_err(|_| NoiseDiagnosticFailure::Configure)?
        .ok_or(NoiseDiagnosticFailure::Configure)?;
    if pool_set.prefer_fallback || pool_set.fallback.is_some() {
        return Err(NoiseDiagnosticFailure::Configure);
    }
    let settings = pool_set.primary.ok_or(NoiseDiagnosticFailure::Configure)?;
    if settings.maybe_authority_public_key.is_none() {
        return Err(NoiseDiagnosticFailure::Configure);
    }
    Ok(settings)
}

fn wait_for_wifi() -> bool {
    let deadline = Instant::now() + WIFI_TIMEOUT;
    while Instant::now() < deadline {
        if crate::wifi_adapter::current_wifi_snapshot().wifi_status == "connected" {
            return true;
        }
        thread::sleep(WIFI_POLL);
    }
    false
}

fn publish_event(event: NoiseDiagnosticEvent) {
    match event {
        NoiseDiagnosticEvent::Stage(stage) => crate::info_retained(&format!(
            "stratum_v2_noise_diagnostic={{\"schema\":\"bitaxe-stratum-v2-noise-diagnostic-v2\",\"stage\":\"{}\"}}",
            stage.label()
        )),
        NoiseDiagnosticEvent::Timing(kind, duration_ms) => crate::info_retained(&format!(
            "stratum_v2_noise_timing={{\"schema\":\"bitaxe-stratum-v2-noise-timing-v2\",\"phase\":\"{}\",\"duration_ms\":{duration_ms}}}",
            kind.label()
        )),
        NoiseDiagnosticEvent::LocalPort(local_port) => crate::info_retained(&format!(
            "stratum_v2_noise_connection_private={{\"schema\":\"bitaxe-stratum-v2-noise-connection-private-v1\",\"local_port\":{local_port}}}"
        )),
        NoiseDiagnosticEvent::SocketError { phase, category } => crate::info_retained(&format!(
            "stratum_v2_noise_socket_error={{\"schema\":\"bitaxe-stratum-v2-noise-socket-error-v1\",\"phase\":\"{phase}\",\"category\":\"{category}\"}}"
        )),
        NoiseDiagnosticEvent::ActOneBytesWritten(count) => crate::info_retained(&format!(
            "stratum_v2_noise_send={{\"schema\":\"bitaxe-stratum-v2-noise-send-v1\",\"kind\":\"act_one\",\"bytes_written\":{count}}}"
        )),
        NoiseDiagnosticEvent::ProofBytesWritten(count) => crate::info_retained(&format!(
            "stratum_v2_noise_send={{\"schema\":\"bitaxe-stratum-v2-noise-send-v1\",\"kind\":\"proof\",\"bytes_written\":{count}}}"
        )),
    }
}

fn publish_terminal(
    category: &'static str,
    accepted: bool,
    act_one_bytes_written: u16,
    proof_bytes_written: u16,
) {
    crate::info_retained(&format!(
        "stratum_v2_noise_terminal={{\"schema\":\"bitaxe-stratum-v2-noise-terminal-v2\",\"category\":\"{category}\",\"accepted\":{accepted},\"authority_required\":true,\"act_one_bytes_written\":{act_one_bytes_written},\"proof_bytes_written\":{proof_bytes_written},\"lease_present\":true,\"mining_started\":false,\"asic_touched\":false,\"fan_touched\":false,\"voltage_touched\":false}}"
    ));
}
