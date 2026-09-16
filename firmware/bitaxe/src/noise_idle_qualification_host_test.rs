#![allow(dead_code)]
#[path = "production_mining_session/qualification.rs"]
mod qualification;
#[path = "production_mining_session/revocation.rs"]
mod revocation;
#[path = "production_mining_session/shutdown_budget.rs"]
mod shutdown_budget;
mod runtime_uptime {
    pub fn millis() -> u64 {
        1000
    }
}
mod settings_adapter {
    pub fn start_mining_on_boot() -> bool {
        false
    }
}
mod worker_acceptance_budget {
    pub fn diagnostic_snapshot() -> (u64, bool) {
        (0, false)
    }
}
mod worker_qualification_budget {
    pub fn observation(_: u32, _: u64) -> Option<serde_json::Value> {
        None
    }
}
mod owner_resources {
    pub fn observation(_: u32) -> Option<serde_json::Value> {
        None
    }
}
mod mining_progress {
    pub fn observation(_: u32) -> Option<serde_json::Value> {
        None
    }
}
mod task_watchdog_observation {
    use bitaxe_core::runtime_health::TaskWatchdogObservation;
    pub struct Snapshot {
        pub maybe_latest: Option<TaskWatchdogObservation>,
    }
    pub fn coherent_observation() -> Snapshot {
        Snapshot {
            maybe_latest: Some(TaskWatchdogObservation::Fed {
                sequence: 1,
                observed_at_millis: 999,
            }),
        }
    }
}
mod safety_adapter {
    use bitaxe_safety::observation::{
        BootSessionId, MonotonicMillis, Observation, ObservationSequence, StampedSample,
    };
    pub struct Snapshot {
        pub bus_voltage_volts: Observation<f64>,
        pub power_watts: Observation<f64>,
        pub chip_temp_celsius: Observation<f64>,
        pub fan_rpm: Observation<u16>,
    }
    fn fresh<T>(value: T) -> Observation<T> {
        Observation::Fresh {
            sample: StampedSample::new(
                value,
                BootSessionId::new(1),
                ObservationSequence::new(1),
                MonotonicMillis::new(999),
            ),
        }
    }
    pub fn observation_snapshot() -> Snapshot {
        Snapshot {
            bus_voltage_volts: fresh(5.0),
            power_watts: fresh(1.0),
            chip_temp_celsius: fresh(40.0),
            fan_rpm: fresh(1200),
        }
    }
}
#[test]
fn production_projection_has_idle_counters_and_real_provider_health_without_mutation() {
    // Arrange
    assert!(revocation::timing(1000).is_none());
    // Act
    let value = qualification::status_evidence(None).expect("idle projection");
    // Assert
    assert_eq!(value["schema"], "worker-qualification-v1");
    assert_eq!(value["generation"], 0);
    assert_eq!(value["work_dispatched"], 0);
    assert_eq!(value["submitted"], 0);
    assert_eq!(value["voltage_volts"], 5.0);
    assert_eq!(value["fan_rpm"], 1200);
    assert_eq!(value["watchdog_alive"], true);
    assert_eq!(value["voltage_fresh"], true);
    assert_eq!(value["safe_stop_complete"], false);
    assert!(value["shutdown_started_ms"].is_null());
    assert!(value["active_limit_ms"].is_null());
    assert!(value.get("attempt").is_none());
    assert!(revocation::timing(1000).is_none());
}
