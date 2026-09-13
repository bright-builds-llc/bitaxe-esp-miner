//! Nonblocking measurement adapter; this module never owns a control or mining lease.
use bitaxe_worker_control::cadence::{CadenceEndpoint, CadenceRecorder, CadenceSnapshot};
use esp_idf_svc::sys;
use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) static RECORDER: CadenceRecorder = CadenceRecorder::new();
static HTTP_PORT: AtomicU32 = AtomicU32::new(0);

pub(crate) fn now_us() -> u64 {
    u64::try_from(unsafe { sys::esp_timer_get_time() }).unwrap_or(0)
}

pub(crate) fn http_prepared(port: u16) {
    HTTP_PORT.store(u32::from(port), Ordering::Release);
}

pub(crate) fn http_activated() {
    HTTP_PORT.fetch_or(1 << 16, Ordering::AcqRel);
}

pub(crate) fn snapshot() -> CadenceSnapshot {
    let mut snapshot = RECORDER.snapshot();
    snapshot.storage_bytes += std::mem::size_of::<AtomicU32>();
    snapshot
}

pub(crate) fn maybe_endpoint(generation: u32) -> Option<CadenceEndpoint> {
    let port = HTTP_PORT.load(Ordering::Acquire);
    if port & (1 << 16) == 0 || port & 0xffff == 0 || generation == 0 {
        return None;
    }
    let ipv4 = crate::wifi_adapter::maybe_connected_station_ipv4()?;
    let observed_at_us = now_us();
    if observed_at_us == 0 {
        return None;
    }
    Some(CadenceEndpoint {
        schema: "worker-telemetry-endpoint-v1",
        ipv4: ipv4.to_string(),
        http_port: u16::try_from(port & 0xffff).ok()?,
        observed_at_us,
        boot_ordinal: crate::boot_evidence::boot_ordinal(),
        generation,
    })
}

const _: () =
    assert!(std::mem::size_of::<CadenceRecorder>() + std::mem::size_of::<AtomicU32>() <= 2048);
