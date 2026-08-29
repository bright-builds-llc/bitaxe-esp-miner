//! Private boot-time STR-005 TCP payload diagnostic owner.

use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::{Duration, Instant};

use crate::settings_adapter::{TcpPayloadDiagnosticAdmission, V2PoolSettings};
use crate::stratum_v2_tcp_payload_replay::replay_deadline_ms;

const OWNER_THREAD_NAME: &str = "stratum-v2-tcp-diag";
const OWNER_STACK_BYTES: usize = 8 * 1_024;
const WIFI_TIMEOUT: Duration = Duration::from_secs(60);
const WIFI_POLL: Duration = Duration::from_millis(100);
const MONITOR_ARM_DELAY: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);
const ADDRESS_CAPACITY: usize = 8;
const RECEIPT_ACK: u8 = 0xa5;
const PAYLOAD: [u8; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
];

pub(crate) fn start(admission: TcpPayloadDiagnosticAdmission) -> anyhow::Result<()> {
    thread::Builder::new()
        .name(OWNER_THREAD_NAME.to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(move || run(admission))?;
    Ok(())
}

fn run(admission: TcpPayloadDiagnosticAdmission) {
    let mut transcript = DiagnosticTranscript::default();
    if admission.lease() == 0 {
        complete(&mut transcript, "state", false);
        return;
    }
    if !wait_for_wifi() {
        complete(&mut transcript, "wifi", false);
        return;
    }
    thread::sleep(MONITOR_ARM_DELAY);
    transcript.record_stage("monitor_armed");
    let settings = match exact_primary_settings() {
        Ok(settings) => settings,
        Err(category) => {
            complete(&mut transcript, category, false);
            return;
        }
    };
    match connect_and_send(settings, &mut transcript) {
        Ok(()) => complete(&mut transcript, "accepted", true),
        Err(category) => complete(&mut transcript, category, false),
    }
}

fn connect_and_send(
    settings: V2PoolSettings,
    transcript: &mut DiagnosticTranscript,
) -> Result<(), &'static str> {
    let addresses = (
        settings.session.endpoint_host.as_str(),
        settings.session.endpoint_port,
    )
        .to_socket_addrs()
        .map_err(|_| "resolve")?
        .take(ADDRESS_CAPACITY + 1)
        .collect::<Vec<_>>();
    if addresses.is_empty() || addresses.len() > ADDRESS_CAPACITY {
        return Err("resolve");
    }
    transcript.record_stage("resolved");
    let started = Instant::now();
    let mut stream = addresses
        .iter()
        .find_map(|address| TcpStream::connect_timeout(address, CONNECT_TIMEOUT).ok())
        .ok_or("connect")?;
    transcript.record_timing("connect", started.elapsed());
    transcript.record_stage("tcp_connected");
    stream.set_nodelay(true).map_err(|_| "configure")?;
    stream
        .set_write_timeout(Some(WRITE_TIMEOUT))
        .map_err(|_| "configure")?;
    let started = Instant::now();
    stream.write_all(&PAYLOAD).map_err(|_| "write")?;
    stream.flush().map_err(|_| "flush")?;
    transcript.record_timing("write", started.elapsed());
    transcript.record_stage("payload_sent");
    stream
        .shutdown(Shutdown::Write)
        .map_err(|error| shutdown_error_category(error.kind()))?;
    transcript.record_stage("write_half_closed");
    stream
        .set_read_timeout(Some(WRITE_TIMEOUT))
        .map_err(|_| "configure")?;
    let mut receipt = [0_u8; 1];
    stream.read_exact(&mut receipt).map_err(|_| "receipt")?;
    if receipt[0] != RECEIPT_ACK {
        return Err("receipt");
    }
    transcript.record_stage("receipt_acknowledged");
    Ok(())
}

#[derive(Default)]
struct DiagnosticTranscript {
    stages: Vec<&'static str>,
    timings: Vec<(&'static str, u32)>,
    terminal: Option<(&'static str, bool)>,
}

impl DiagnosticTranscript {
    fn record_stage(&mut self, stage: &'static str) {
        self.stages.push(stage);
        publish_stage(stage);
    }

    fn record_timing(&mut self, phase: &'static str, duration: Duration) {
        let duration_ms = u32::try_from(duration.as_millis()).unwrap_or(u32::MAX);
        self.timings.push((phase, duration_ms));
        publish_timing(phase, duration_ms);
    }

    fn record_terminal(&mut self, category: &'static str, accepted: bool) {
        self.terminal = Some((category, accepted));
        publish_terminal(category, accepted);
    }

    fn replay(&self) {
        for stage in &self.stages {
            publish_stage(stage);
        }
        for (phase, duration_ms) in &self.timings {
            publish_timing(phase, *duration_ms);
        }
        if let Some((category, accepted)) = self.terminal {
            publish_terminal(category, accepted);
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

fn shutdown_error_category(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::WouldBlock => "shutdown_would_block",
        ErrorKind::NotConnected => "shutdown_not_connected",
        ErrorKind::OutOfMemory => "shutdown_out_of_memory",
        ErrorKind::InvalidInput => "shutdown_invalid_input",
        ErrorKind::Unsupported => "shutdown_unsupported",
        _ => "shutdown_other",
    }
}

fn exact_primary_settings() -> Result<V2PoolSettings, &'static str> {
    let pool_set = crate::settings_adapter::read_stratum_v2_pool_set()
        .map_err(|_| "configure")?
        .ok_or("configure")?;
    if pool_set.prefer_fallback || pool_set.fallback.is_some() {
        return Err("configure");
    }
    pool_set.primary.ok_or("configure")
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

fn publish_stage(stage: &'static str) {
    crate::info_retained(&format!(
        "stratum_v2_tcp_payload={{\"schema\":\"bitaxe-stratum-v2-tcp-payload-v1\",\"stage\":\"{stage}\"}}"
    ));
}

fn publish_timing(phase: &'static str, duration_ms: u32) {
    crate::info_retained(&format!(
        "stratum_v2_tcp_payload_timing={{\"schema\":\"bitaxe-stratum-v2-tcp-payload-timing-v1\",\"phase\":\"{phase}\",\"duration_ms\":{duration_ms}}}"
    ));
}

fn publish_terminal(category: &'static str, accepted: bool) {
    let bytes_written = if accepted { PAYLOAD.len() } else { 0 };
    crate::info_retained(&format!(
        "stratum_v2_tcp_payload_terminal={{\"schema\":\"bitaxe-stratum-v2-tcp-payload-terminal-v1\",\"category\":\"{category}\",\"accepted\":{accepted},\"bytes_written\":{bytes_written},\"noise_started\":false,\"mining_started\":false,\"asic_touched\":false,\"fan_touched\":false,\"voltage_touched\":false}}"
    ));
}
