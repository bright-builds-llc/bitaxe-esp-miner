#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PanicSignature {
    StackOverflow,
    StackSmashing,
    HeapCorruption,
    Assertion,
    Abort,
    RustPanic,
    GuruMeditation,
}

impl PanicSignature {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::StackOverflow => "stack_overflow",
            Self::StackSmashing => "stack_smashing",
            Self::HeapCorruption => "heap_corruption",
            Self::Assertion => "assertion",
            Self::Abort => "abort",
            Self::RustPanic => "rust_panic",
            Self::GuruMeditation => "guru_meditation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PanicTaskFamily {
    None,
    ProductionMiningSession,
    ProductionAsic,
    AxeosLiveWebsocket,
    DeferredEffects,
    SafetySupervisor,
    OperatorSensor,
    FanController,
    Statistics,
    WifiReconnect,
    HttpServer,
    Main,
    Other,
}

impl PanicTaskFamily {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ProductionMiningSession => "production_mining_session",
            Self::ProductionAsic => "production_asic",
            Self::AxeosLiveWebsocket => "axeos_live_websocket",
            Self::DeferredEffects => "deferred_effects",
            Self::SafetySupervisor => "safety_supervisor",
            Self::OperatorSensor => "operator_sensor",
            Self::FanController => "fan_controller",
            Self::Statistics => "statistics",
            Self::WifiReconnect => "wifi_reconnect",
            Self::HttpServer => "http_server",
            Self::Main => "main",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PanicLineDiagnostic {
    pub(super) signature: PanicSignature,
    pub(super) task_family: PanicTaskFamily,
}

pub(super) fn classify_panic_line(line: &[u8]) -> Option<PanicLineDiagnostic> {
    let text = std::str::from_utf8(line).ok()?;
    let lower = text.to_ascii_lowercase();
    let signature = if lower.contains("stack overflow in task") {
        PanicSignature::StackOverflow
    } else if lower.contains("stack smashing protect failure") {
        PanicSignature::StackSmashing
    } else if lower.contains("corrupt heap") || lower.contains("heap corruption detected") {
        PanicSignature::HeapCorruption
    } else if lower.contains("assert failed:") || lower.contains("assertion failed") {
        PanicSignature::Assertion
    } else if lower.contains("abort() was called") {
        PanicSignature::Abort
    } else if lower.contains("panicked at") {
        PanicSignature::RustPanic
    } else if lower.contains("guru meditation error") {
        PanicSignature::GuruMeditation
    } else {
        return None;
    };
    let task_family = if signature == PanicSignature::StackOverflow {
        classify_stack_overflow_task_family(&lower)
    } else {
        PanicTaskFamily::None
    };
    Some(PanicLineDiagnostic {
        signature,
        task_family,
    })
}

fn classify_stack_overflow_task_family(line: &str) -> PanicTaskFamily {
    for (fragment, family) in [
        ("production-min", PanicTaskFamily::ProductionMiningSession),
        ("production-asic", PanicTaskFamily::ProductionAsic),
        ("axeos-live-ws", PanicTaskFamily::AxeosLiveWebsocket),
        ("deferred-effect", PanicTaskFamily::DeferredEffects),
        ("bitaxe-safety", PanicTaskFamily::SafetySupervisor),
        ("operator-sensor", PanicTaskFamily::OperatorSensor),
        ("fan-controller", PanicTaskFamily::FanController),
        ("statistics", PanicTaskFamily::Statistics),
        ("wifi-reconnect", PanicTaskFamily::WifiReconnect),
        ("httpd", PanicTaskFamily::HttpServer),
        ("main", PanicTaskFamily::Main),
    ] {
        if line.contains(fragment) {
            return family;
        }
    }
    PanicTaskFamily::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_closed_panic_signature_has_one_exact_marker() {
        // Arrange
        let cases = [
            (
                "***ERROR*** A stack overflow in task other-task has been detected.",
                "stack_overflow",
            ),
            ("Stack smashing protect failure!", "stack_smashing"),
            ("CORRUPT HEAP: Bad head at 0x1234", "heap_corruption"),
            ("assert failed: queue.c:42", "assertion"),
            ("abort() was called at PC 0x1234 on core 0", "abort"),
            ("thread panicked at source.rs:42", "rust_panic"),
            ("Guru Meditation Error: Core 0 panic'ed", "guru_meditation"),
        ];

        // Act / Assert
        for (line, expected) in cases {
            assert_eq!(
                classify_panic_line(line.as_bytes()).map(|value| value.signature.label()),
                Some(expected)
            );
        }
    }

    #[test]
    fn task_names_collapse_to_closed_families() {
        // Arrange
        let cases = [
            ("production-min", "production_mining_session"),
            ("production-asic", "production_asic"),
            ("axeos-live-ws", "axeos_live_websocket"),
            ("deferred-effects", "deferred_effects"),
            ("bitaxe-safety-s", "safety_supervisor"),
            ("operator-sensor", "operator_sensor"),
            ("fan-controller", "fan_controller"),
            ("statistics", "statistics"),
            ("wifi-reconnect", "wifi_reconnect"),
            ("httpd", "http_server"),
            ("main", "main"),
            ("private-task-name", "other"),
        ];

        // Act / Assert
        for (task, expected) in cases {
            let line = format!("***ERROR*** A stack overflow in task {task} has been detected.");
            assert_eq!(
                classify_panic_line(line.as_bytes()).map(|value| value.task_family.label()),
                Some(expected)
            );
        }
    }

    #[test]
    fn ordinary_serial_lines_are_not_panic_evidence() {
        // Arrange
        let line = b"I app: runtime heartbeat healthy";

        // Act / Assert
        assert_eq!(classify_panic_line(line), None);
    }

    #[test]
    fn non_stack_panic_does_not_infer_task_from_source_path() {
        // Arrange
        let line = b"thread panicked at src/main.rs:42";

        // Act / Assert
        assert_eq!(
            classify_panic_line(line).map(|value| value.task_family),
            Some(PanicTaskFamily::None)
        );
    }
}
