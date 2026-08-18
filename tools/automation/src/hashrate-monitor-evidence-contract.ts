export const expectedPrivateRoot = "scratch/stat001-hashrate-monitor/attempt-019";
export const expectedWrapperRoot = "scratch/stat001-hashrate-monitor/wrapper-019";
export const expectedProjection =
  "docs/parity/evidence/stat001-hashrate-monitor/hashrate-monitor-projection.json";
export const expectedPlan = "docs/parity/work-plans/20260818T050654Z-STAT-001/PLAN.md";
export const expectedPlanSha256 = "b9bc554eb3e49c685bcbd7a852a754febf015228df4ae89efe6e6b951eb65e24";
export const expectedReferenceCommit = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
export const activeTask = "task-parity-stat001-hashrate-monitor";
export const runtimeAttestationParseFailures = [
  "none",
  "not_observed",
  "invalid_utf8",
  "missing_marker",
  "malformed_token",
  "duplicate_field",
  "unknown_field",
  "missing_field",
  "invalid_field",
  "incomplete_readiness",
] as const;
export const watchdogFailures = [
  "none",
  "supervisor_unavailable",
  "checkpoint_unhealthy",
  "checkpoint_sequence_missing",
  "watchdog_reason_missing",
  "watchdog_unproved",
  "watchdog_snapshot_retry_exhausted",
  "watchdog_snapshot_history_poisoned",
  "watchdog_read_outcome_unknown",
  "watchdog_invalid_observation",
  "watchdog_subscription_failed",
  "watchdog_feed_failed",
  "watchdog_unsubscription_failed",
  "watchdog_unsubscribed",
  "watchdog_reason_unknown",
  "watchdog_participation_inconsistent",
  "watchdog_feed_sequence_missing",
  "watchdog_feed_age_missing",
  "watchdog_feed_stale",
  "watchdog_owner_phase_unknown",
  "watchdog_owner_subphase_unknown",
  "watchdog_wait_state_unknown",
  "http_checkpoint_not_advanced",
  "http_feed_not_advanced",
  "websocket_checkpoint_not_advanced",
  "websocket_feed_not_advanced",
] as const;
export const watchdogReadOutcomes = [
  "stable",
  "uninitialized",
  "retry_exhausted",
  "history_poisoned",
] as const;
export const watchdogWaitStates = [
  "not_waiting",
  "within_deadline",
  "deadline_overrun",
  "invalid_observation",
] as const;
export const watchdogOwnerPhases = [
  "unavailable",
  "subscribing",
  "loop_start",
  "waiting_inbox",
  "handling_inbox",
  "handling_observation",
  "handling_readiness",
  "publishing_campaign_status",
  "servicing_hashrate",
  "shutdown",
] as const;
export const watchdogOwnerSubphases = [
  "unavailable", "inbox_mapping", "session_evaluation",
  "effect_prepare_hardware", "effect_read_pool_configuration", "effect_connect_pool",
  "effect_write_pool_line", "effect_apply_version_mask", "effect_dispatch_chip",
  "effect_poll_chip", "effect_block_submissions", "effect_invalidate_work_and_submissions",
  "effect_stop_chip_interaction", "effect_close_pool_connection", "effect_safe_stop_hardware",
  "effect_record_scoreboard", "effect_record_block_found", "effect_publish",
  "safe_stop_stop_dispatch", "safe_stop_reduce_frequency_and_nonce_state",
  "safe_stop_assert_control_line_low", "safe_stop_disable_core_rail", "safe_stop_disable_chip",
  "safe_stop_set_cooling_maximum", "safe_stop_wait_for_cooling_proof", "safe_stop_set_cooling_paused",
] as const;
export const expectedAttemptFiles = [
  "campaign-diagnostics.private.json",
  "campaign-flash.private.json",
  "campaign-mining-diagnostics.private.json",
  "campaign-network.private.json",
  "campaign-observations.private.json",
  "campaign-result.json",
  "campaign-result.sha256",
] as const;
