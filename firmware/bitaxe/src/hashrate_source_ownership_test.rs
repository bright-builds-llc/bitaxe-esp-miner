const OWNER_SOURCE: &str = include_str!("production_mining_session.rs");
const OWNER_LOOP_SOURCE: &str = include_str!("production_mining_session/owner_loop.rs");
const WORKER_SOURCE: &str = include_str!("production_mining_session/asic_worker.rs");
const HASHRATE_SOURCE: &str = include_str!("production_mining_session/hashrate.rs");
const ASIC_SOURCE: &str = include_str!("asic_adapter/production.rs");
const SNAPSHOT_SOURCE: &str = include_str!("runtime_snapshot.rs");

#[test]
fn sole_production_owner_schedules_active_only_hashrate_reads() {
    // Arrange
    let service = "service_hashrate_monitor(&session.snapshot(), now_ms)";

    // Act
    let service_count = OWNER_LOOP_SOURCE.matches(service).count();

    // Assert
    assert_eq!(service_count, 1);
    assert!(HASHRATE_SOURCE.contains("MiningActivityStatus::Active"));
    assert!(HASHRATE_SOURCE.contains("WorkSubmissionGate::Ready"));
    assert!(OWNER_SOURCE.contains("AsicWorkerCommand::ReadHashrateRegisters"));
    assert!(!OWNER_SOURCE.contains("std::thread::Builder::new().name(\"hashrate"));
    assert!(!OWNER_LOOP_SOURCE.contains("std::thread::Builder::new().name(\"hashrate"));
}

#[test]
fn parsed_register_value_reaches_monitor_before_poll_completion() {
    // Arrange
    let event = "AsicWorkerEvent::RegisterRead";

    // Act
    let event_count = OWNER_SOURCE.matches(event).count();

    // Assert
    assert_eq!(event_count, 1);
    assert!(WORKER_SOURCE.contains("ProductionReadOutcome::RegisterReadProof(read)"));
    assert!(WORKER_SOURCE.contains("emit(AsicWorkerEvent::RegisterRead"));
    assert!(WORKER_SOURCE.contains("observed_at_us: elapsed_micros(started_at)"));
    assert!(OWNER_SOURCE.contains("self.hashrate.observe(read, observed_at_us)"));
    assert!(OWNER_SOURCE.contains("AsicPollCompletion::RegisterRead"));
}

#[test]
fn passive_read_burst_is_guarded_by_production_readiness() {
    // Arrange
    let burst = "fn send_register_read_burst(registers: &[u8]) -> bool";
    let start = ASIC_SOURCE.find(burst).expect("burst helper must exist");
    let source = &ASIC_SOURCE[start..];

    // Act
    let ready_guard = source.find("if !state.production_ready");
    let first_write = source.find("uart.write_frame");

    // Assert
    assert!(ready_guard.is_some());
    assert!(ready_guard < first_write);
    assert_eq!(
        WORKER_SOURCE
            .matches("request_hashrate_monitor_register_reads_tx()")
            .count(),
        1
    );
}

#[test]
fn stop_resets_measurements_and_runtime_publication_preserves_samples() {
    // Arrange
    let stop_branch = "if !active";

    // Act
    let stop_index = HASHRATE_SOURCE
        .find(stop_branch)
        .expect("inactive branch must exist");
    let stopped_source = &HASHRATE_SOURCE[stop_index..];

    // Assert
    assert!(stopped_source.contains("self.monitor.sample(false)"));
    assert!(SNAPSHOT_SOURCE.contains("let hashrate = state.mining.hashrate_inputs.clone()"));
    assert!(SNAPSHOT_SOURCE.contains("state.mining.record_hashrate_inputs(hashrate)"));
    assert!(SNAPSHOT_SOURCE.contains("pub fn publish_hashrate_snapshot"));
}

#[test]
fn runtime_service_requests_only_at_active_one_second_boundaries() {
    // Arrange
    let mut service = ProductionHashrateMonitor::new();
    let inactive_snapshot =
        bitaxe_stratum::v1::production_session::ProductionMiningSession::new().snapshot();

    // Act
    let initial = service
        .service_snapshot(&inactive_snapshot, 100)
        .expect("schedule should remain valid")
        .expect("initial inactive topology should publish");
    let repeated_inactive = service
        .service(false, 200)
        .expect("schedule should remain valid");
    let active = service
        .service(true, 300)
        .expect("schedule should remain valid")
        .expect("active transition is immediately due");
    let early = service
        .service(true, 1_299)
        .expect("schedule should remain valid");
    let due = service
        .service(true, 1_300)
        .expect("schedule should remain valid")
        .expect("one-second boundary should be due");

    // Assert
    assert!(!initial.request_registers);
    assert_eq!(initial.snapshot.asics.len(), 1);
    assert!(repeated_inactive.is_none());
    assert!(active.request_registers);
    assert!(early.is_none());
    assert!(due.request_registers);
}

#[test]
fn runtime_service_ignores_inactive_reads_and_resets_on_stop() {
    // Arrange
    let mut service = ProductionHashrateMonitor::new();
    let register = |value| Bm1366RegisterRead {
        register: Bm1366Register::TotalCount,
        asic_index: 0,
        asic_address: 0,
        value,
    };
    service.observe(register(1_000), 1_000_000);
    let _ = service
        .service(true, 2_000)
        .expect("schedule should remain valid");
    service.observe(register(10), 2_000_000);
    service.observe(register(20), 3_000_000);

    // Act
    let active = service
        .service(true, 3_000)
        .expect("schedule should remain valid")
        .expect("one-second boundary should be due");
    let stopped = service
        .service(false, 3_100)
        .expect("schedule should remain valid")
        .expect("stop transition should publish");
    service.observe(register(u32::MAX), 4_000_000);
    let resumed = service
        .service(true, 5_000)
        .expect("schedule should remain valid")
        .expect("resume transition should publish");

    // Assert
    assert!(active.snapshot.current_ghs > 0.0);
    assert_eq!(stopped.snapshot.current_ghs, 0.0);
    assert_eq!(resumed.snapshot.current_ghs, 0.0);
}
#[path = "production_mining_session/hashrate.rs"]
mod hashrate;

use bitaxe_asic::bm1366::{
    registers::Bm1366Register,
    result::Bm1366RegisterRead,
};
use hashrate::ProductionHashrateMonitor;
