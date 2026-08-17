const OWNER_SOURCE: &str = include_str!("production_mining_session.rs");
const OWNER_LOOP_SOURCE: &str = include_str!("production_mining_session/owner_loop.rs");
const OWNER_PROGRESS_SOURCE: &str = include_str!("production_mining_session/owner_progress.rs");
const WORKER_SOURCE: &str = include_str!("production_mining_session/asic_worker.rs");
const HASHRATE_SOURCE: &str = include_str!("production_mining_session/hashrate.rs");
const CAMPAIGN_STATUS_SOURCE: &str =
    include_str!("production_mining_session/campaign_status/publication.rs");
const TASK_WATCHDOG_OBSERVATION_SOURCE: &str = include_str!("task_watchdog_observation.rs");
const RUNTIME_HEALTH_ADAPTER_SOURCE: &str = include_str!("runtime_health_adapter.rs");
const ASIC_SOURCE: &str = include_str!("asic_adapter/production.rs");
const SNAPSHOT_SOURCE: &str = include_str!("runtime_snapshot.rs");
const SDKCONFIG_DEFAULTS: &str = include_str!("../sdkconfig.defaults");

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
fn owner_watchdog_feeds_only_after_completed_cooperative_progress() {
    // Arrange
    let execute = "let maybe_feedback = execute(effect);";
    let completed = "progress(OwnerProgressBoundary::EffectCompleted);";

    // Act
    let execute_index = OWNER_PROGRESS_SOURCE
        .find(execute)
        .expect("effect execution must be explicit");
    let completed_index = OWNER_PROGRESS_SOURCE
        .find(completed)
        .expect("completed effect progress must be explicit");

    // Assert
    assert!(execute_index < completed_index);
    assert!(OWNER_LOOP_SOURCE.contains("drive_feedback("));
    assert!(OWNER_LOOP_SOURCE.contains("|_| task_watchdog.feed("));
    assert!(!WORKER_SOURCE.contains("ProductionTaskWatchdog"));
}

#[test]
fn owner_phase_and_campaign_publication_have_single_production_ownership() {
    // Arrange
    let publication = "if let Err(error) = adapter.publish_campaign_status";
    let phase_store = "static OWNER_PHASE: AtomicU8";

    // Act / Assert
    assert_eq!(OWNER_LOOP_SOURCE.matches(publication).count(), 1);
    assert!(OWNER_LOOP_SOURCE
        .contains("record_owner_phase(TaskWatchdogOwnerPhase::PublishingCampaignStatus)"));
    assert!(OWNER_LOOP_SOURCE
        .contains("record_owner_phase(TaskWatchdogOwnerPhase::ServicingHashrate)"));
    assert_eq!(TASK_WATCHDOG_OBSERVATION_SOURCE.matches(phase_store).count(), 1);
    assert!(TASK_WATCHDOG_OBSERVATION_SOURCE.contains("Ordering::Release"));
    assert!(TASK_WATCHDOG_OBSERVATION_SOURCE.contains("Ordering::Acquire"));
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains(".with_task_watchdog_owner_phase("));
    assert!(CAMPAIGN_STATUS_SOURCE
        .contains("CAMPAIGN_STATUS_PUBLICATION_INTERVAL_MS: u64 = 1_000"));
}

#[test]
fn runtime_health_copies_producer_facts_before_sampling_evaluation_time() {
    // Arrange
    let checkpoint_read = "supervisor_checkpoint_history()";
    let watchdog_read = "task_watchdog_observation::observation_history()";
    let phase_read = "task_watchdog_observation::owner_observation()";
    let clock_read = "let current_monotonic_millis = crate::runtime_uptime::millis();";

    // Act
    let checkpoint_index = RUNTIME_HEALTH_ADAPTER_SOURCE
        .find(checkpoint_read)
        .expect("checkpoint history read must exist");
    let watchdog_index = RUNTIME_HEALTH_ADAPTER_SOURCE
        .find(watchdog_read)
        .expect("watchdog history read must exist");
    let phase_index = RUNTIME_HEALTH_ADAPTER_SOURCE
        .find(phase_read)
        .expect("owner phase read must exist");
    let clock_index = RUNTIME_HEALTH_ADAPTER_SOURCE
        .find(clock_read)
        .expect("post-observation clock read must exist");

    // Assert
    assert!(checkpoint_index < clock_index);
    assert!(watchdog_index < clock_index);
    assert!(phase_index < clock_index);
    assert!(RUNTIME_HEALTH_ADAPTER_SOURCE.contains("pub(crate) fn collect()"));
    assert!(!RUNTIME_HEALTH_ADAPTER_SOURCE.contains("collect(current_monotonic_millis"));
    assert!(SNAPSHOT_SOURCE.contains("runtime_health_adapter::collect()"));
    assert!(!SNAPSHOT_SOURCE.contains("runtime_health_adapter::collect(crate::runtime_uptime"));
}

#[test]
fn waiting_inbox_arms_deadline_before_phase_and_uses_priority_five() {
    // Arrange
    let deadline_store = "OWNER_WAIT_DEADLINE_MILLIS.store";
    let waiting_phase_store = "TaskWatchdogOwnerPhase::WaitingInbox as u8";

    // Act
    let deadline_store_index = TASK_WATCHDOG_OBSERVATION_SOURCE
        .find(deadline_store)
        .expect("wait deadline store must exist");
    let waiting_phase_index = TASK_WATCHDOG_OBSERVATION_SOURCE
        .find(waiting_phase_store)
        .expect("waiting phase publication must exist");
    let deadline_compute_index = OWNER_LOOP_SOURCE
        .find("let maybe_wait_deadline_millis")
        .expect("wait deadline computation must exist");
    let wait_publish_index = OWNER_LOOP_SOURCE
        .find("record_owner_wait(maybe_wait_deadline_millis)")
        .expect("wait observation publication must exist");
    let receive_index = OWNER_LOOP_SOURCE
        .find("receiver.recv_timeout(wait)")
        .expect("bounded receive must exist");

    // Assert
    assert!(deadline_store_index < waiting_phase_index);
    assert!(deadline_compute_index < wait_publish_index);
    assert!(wait_publish_index < receive_index);
    assert_eq!(
        SDKCONFIG_DEFAULTS
            .matches("CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5")
            .count(),
        1
    );
    assert_eq!(
        bitaxe_core::runtime_orchestration::PRODUCTION_REREAD_CADENCE_MS,
        1_000
    );
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
