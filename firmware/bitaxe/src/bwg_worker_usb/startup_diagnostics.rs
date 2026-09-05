//! Allocation-free startup progress retained independently of startup completion.
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy)]
#[repr(u32)]
pub(crate) enum Stage {
    EarlyIdentity = 1,
    UsbInstall,
    Nvs,
    Hardware,
    WorkerRecovery,
    RuntimeServices,
    StorageHttp,
    Network,
    WorkerControl,
    Statistics,
    RuntimeReady,
}
impl Stage {
    fn label(raw: u32) -> &'static str {
        match raw {
            1 => "early_identity",
            2 => "usb_install",
            3 => "nvs",
            4 => "hardware",
            5 => "worker_recovery",
            6 => "runtime_services",
            7 => "storage_http",
            8 => "network",
            9 => "worker_control",
            10 => "statistics",
            11 => "runtime_ready",
            _ => "none",
        }
    }
}

pub(crate) static PROGRESS: StartupProgress = StartupProgress::new();

pub(crate) struct StartupProgress(AtomicU32);
impl StartupProgress {
    pub(crate) const fn new() -> Self {
        Self(AtomicU32::new(Stage::EarlyIdentity as u32))
    }
    pub(crate) fn enter(&self, stage: Stage) {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                Some((value & 0xff0000) | stage as u32)
            })
            .expect("startup updates always retain a state");
    }
    pub(crate) fn fail(&self, stage: Stage) {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                let first = if value >> 16 == 0 {
                    (stage as u32) << 16
                } else {
                    value & 0xff0000
                };
                Some(first | (value & 0xff) | 0x200)
            })
            .expect("startup updates always retain a state");
    }
    pub(crate) fn complete(&self) {
        self.0.fetch_or(0x100, Ordering::AcqRel);
    }
    pub(crate) fn guard<T, E>(&self, operation: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
        let result = operation();
        if result.is_err() {
            self.0
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                    let first = if value >> 16 == 0 {
                        (value & 0xff) << 16
                    } else {
                        value & 0xff0000
                    };
                    Some(first | (value & 0xff) | 0x200)
                })
                .expect("startup updates always retain a state");
        }
        result
    }
    pub(crate) fn marker(&self, now_ms: u64) -> String {
        let value = self.0.load(Ordering::Acquire);
        let state = if value & 0x200 != 0 {
            "failed"
        } else if value & 0x100 != 0 {
            "complete"
        } else {
            "entered"
        };
        format!("usb_startup schema=v1 stage={} state={state} first_failure={} uptime_ms={now_ms} redacted=true",
            Stage::label(value & 0xff), Stage::label(value >> 16))
    }
}
