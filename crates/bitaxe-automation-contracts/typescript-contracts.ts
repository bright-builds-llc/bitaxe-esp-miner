// Generated from bitaxe-automation-contracts. Do not hand-edit.
export type AutomationCommand =
  | "doctor"
  | "bootstrap-esp"
  | "build-firmware"
  | "package-firmware"
  | "package-rollback-probe"
  | "verify-reference"
  | "verify-redaction"
  | "verify-production-session"
  | "observe-serial"
  | "verify-flash-durability"
  | "verify-firmware-ota"
  | "verify-web-assets-ota"
  | "verify-recovery"
  | "verify-http-api"
  | "verify-hardware-surface"
  | "verify-mining"
  | "capture-operator-evidence"
  | "verify-settings-durability"
  | "api-command-effects-campaign"
  | "verify-theme-durability"
  | "capture-correlated-runtime-evidence"
  | "capture-version-evidence"
  | "capture-operator-snapshot-evidence"
  | "capture-runtime-health-evidence"
  | "capture-system-info-evidence"
  | "capture-adc-observation-evidence"
  | "capture-hashrate-monitor-evidence"
  | "capture-ultra205-defaults-evidence"
  | "capture-settings-patch-evidence"
  | "capture-statistics-history-evidence"
  | "capture-log-buffer-evidence"
  | "capture-partition-layout-evidence"
  | "capture-sdkconfig-rollback-evidence"
  | "capture-network-reconnect-evidence"
  | "capture-network-scan-evidence"
  | "project-asic-initialization-evidence"
  | "project-asic-power-initialization-evidence"
  | "project-core-voltage-control-evidence" | "project-display-behavior-evidence" | "project-screen-flow-evidence" | "project-ina260-evidence" | "capture-emc2101-thermal-evidence" | "capture-emc2101-thermal-fault-evidence"
  | "project-asic-reset-evidence"
  | "project-asic-work-send-evidence"
  | "project-asic-result-parsing-evidence" | "project-asic-serial-transport-evidence"
  | "project-asic-frequency-transition-evidence"
  | "project-stratum-socket-evidence"
  | "project-protocol-coordinator-evidence"
  | "project-mining-criteria-evidence"
  | "capture-provisioning-network-evidence" | "project-ui-workflow-evidence";
export type AutomationStatus = "succeeded" | "failed" | "blocked";
export type AutomationCategory =
  | "complete"
  | "invalid_invocation"
  | "contract_mismatch"
  | "workspace_invalid"
  | "dependency_unavailable"
  | "policy_blocked"
  | "authorization_blocked"
  | "process_failed"
  | "timeout"
  | "evidence_invalid"
  | "hardware_blocked"
  | "package_invalid"
  | "interruption_not_observed"
  | "probe_boot_failed"
  | "rollback_not_observed"
  | "recovery_failed"
  | "reconnect_not_observed"
  | "reconnect_timing_invalid"
  | "service_recovery_failed" | "browser_blocked";
export type AutomationResult = {
  schema_version: "bitaxe-automation-result-v1";
  command: AutomationCommand;
  status: AutomationStatus;
  category: AutomationCategory;
  public?: unknown;
};
export type WorkflowIdentity = {
  schema_version: "bitaxe-workflow-identity-v1";
  command: AutomationCommand;
  request_sha256: string;
};
export type VersionEvidence = {
  schema_version: "bitaxe-version-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  boot_observed: true;
  same_origin_api_observed: true;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  redaction_status: "passed";
  version_projection?: VersionProjectionEvidence;
};

export type VersionProjectionEvidence = {
  api_build_label_matches_manifest: true;
  api_static_asset_version_matches_manifest: true;
  api_extended_provenance_matches_manifest: true;
  api_esp_idf_version_matches_manifest: true;
  websocket_same_boot_revision_observed: true;
  websocket_version_projection_matches_api: true;
};

export type OperatorSnapshotEpochEvidence = {
  boot_session_sha256: string;
  http_snapshot_observed: true;
  websocket_snapshot_observed: true;
  same_boot_session: true;
  http_revision: number;
  websocket_revision: number;
  websocket_revision_not_earlier: true;
  retained_log_marker_matches_http: true;
  retained_log_marker_matches_websocket: true;
  substantive_fields_present: true;
  stable_fields_match: true;
  safe_operator_state_confirmed: true;
  substantive_projection_sha256: string;
};

export type DeviceSessionEvidence = Readonly<Record<string, unknown>>;

export type ThemeDurabilityEvidence = {
  schema_version: "bitaxe-theme-durability-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  restart_session: DeviceSessionEvidence;
  theme_get_observed: true;
  theme_post_readback: true;
  normal_restart_observed: true;
  post_restart_persistence: true;
  restoration_complete: true;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type OperatorSnapshotEvidence = {
  schema_version: "bitaxe-operator-snapshot-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  baseline_epoch: OperatorSnapshotEpochEvidence;
  post_restart_epoch: OperatorSnapshotEpochEvidence;
  distinct_boot_sessions: true;
  restart_session: DeviceSessionEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type RuntimeHealthEvidence = {
  schema_version: "bitaxe-runtime-health-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  runtime_health: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};
export type SystemInfoEvidence = {
  schema_version: "bitaxe-system-info-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  system_info: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};
export type AdcObservationSourceEvidence = { system_info_projection_sha256: string; api_snapshot_sha256: string; websocket_snapshot_sha256: string; plan_sha256: string; system_info_projection_valid: true; protected_modes_valid: true; production_source_current: true; source_semantics_admitted: true; compatible_path_count: 7; };
export type AdcObservationQuorum = { adc_unit: 1; adc_channel: 1; gpio: 2; attenuation_db: 12; default_resolution: true; curve_calibration: true; producer_cadence_ms: 500; read_only_acquisition: true; http_fresh_sample: true; websocket_fresh_sample: true; finite_nonnegative_millivolts: true; millivolt_wire_domain_valid: true; disabled_state_bound: true; sequence_not_regressed: true; acquisition_time_not_regressed: true; same_boot_session: true; exact_public_correlation: true; exact_package_identity: true; };
export type AdcObservationEvidence = { schema_version: "bitaxe-adc-observation-evidence-v1"; board: 205; attempt_ordinal: 4; source_commit: string; reference_commit: string; package_manifest_sha256: string; workflow: WorkflowIdentity; source: AdcObservationSourceEvidence; adc: AdcObservationQuorum; detector_admitted: true; boot_observed: true; mining_state: "disabled"; hardware_control_state: "disabled"; cleanup_complete: true; recovery_used: false; redaction_status: "passed"; };
export type HashrateMonitorSourceEvidence = { plan_sha256: string; campaign_result_sha256: string; campaign_network_sha256: string; source_semantics_current: boolean; reference_semantics_current: boolean; source_path_count: number; };
export type HashrateTransportQuorum = { active_sample_count: number; positive_coherent_count: number; distinct_positive_count: number; warm_rolling_window_count: number; terminal_zero_confirmed: boolean; };
export type HashrateMonitorQuorum = { monitor_cadence_ms: number; asic_count: number; domain_count: number; required_window_count: number; covered_window_count: number; http: HashrateTransportQuorum; websocket: HashrateTransportQuorum; };
export type HashrateMonitorEvidence = { schema_version: "bitaxe-hashrate-monitor-evidence-v1"; board: 205; attempt_ordinal: 19; source_commit: string; reference_commit: string; package_manifest_sha256: string; workflow: WorkflowIdentity; source: HashrateMonitorSourceEvidence; hashrate: HashrateMonitorQuorum; detector_admitted: boolean; runtime_identity: "trusted"; campaign_profile: "conservative"; campaign_duration_seconds: 600; network_status: "accepted"; mining_state: "active_then_paused"; safe_stop_confirmed: boolean; cleanup_complete: boolean; hardware_rerun_used: false; redaction_status: "passed"; };
export type ReleaseRecoveryEvidence = { schema_version: "bitaxe-release-recovery-evidence-v1"; board: 205; attempt_ordinal: 1; source_commit: string; reference_commit: string; package_manifest_sha256: string; plan_sha256: string; detector_admitted: true; large_erase_completed: true; factory_restore_completed: true; wifi_seed_restored: true; mineonboot_disabled: true; runtime_identity_trusted: true; spiffs_ready: true; passive_safe_state_confirmed: true; cleanup_complete: true; recovery_flash_used: false; redaction_status: "passed"; };
export type Ultra205DefaultsObservationEvidence = {
  configured_default_field_count: number;
  firmware_matching_field_count: number;
  firmware_all_defaults_match: boolean;
  api_visible_default_field_count: number;
  http_defaults_match: boolean;
  websocket_defaults_match: boolean;
  retained_attestation_matches: boolean;
  mining_on_boot_disabled: boolean;
  exact_seed_fixture_sha256: string;
  system_info_evidence_sha256: string;
};
export type Ultra205DefaultsEvidence = {
  schema_version: "bitaxe-ultra205-defaults-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  system_info: SystemInfoEvidence;
  defaults: Ultra205DefaultsObservationEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  private_modes_valid: true;
  redaction_status: "passed";
};
export type SettingsPatchEvidence = { schema_version: "bitaxe-settings-patch-evidence-v1"; board: 205; source_commit: string; reference_commit: string; package_manifest_sha256: string; workflow: WorkflowIdentity; detector_admitted: true; boot_observed: true; same_origin_observed: true; settings_patch: Readonly<Record<string, unknown>>; mining_state: "disabled"; hardware_control_state: "disabled"; cleanup_complete: true; redaction_status: "passed"; };
export type StatisticsHistoryObservationEvidence = { original_setting_sha256: string; enabled_setting_sha256: string; mutation_request_field_count: 1; enabled_readback_confirmed: true; label_count: 19; row_width: 19; sample_count: number; interval_count: number; minimum_interval_ms: number; maximum_interval_ms: number; timestamps_strictly_increasing: true; finite_numeric_rows: true; immediate_repeat_unchanged: true; later_producer_growth: true; restoration_complete: true; zero_setting_clear_status: "observed" | "not_applicable"; };
export type StatisticsHistoryEvidence = { schema_version: "bitaxe-statistics-history-evidence-v1"; board: 205; source_commit: string; reference_commit: string; package_manifest_sha256: string; plan_sha256: string; workflow: WorkflowIdentity; detector_admitted: true; boot_observed: true; same_origin_observed: true; statistics_history: StatisticsHistoryObservationEvidence; mining_state: "disabled"; hardware_control_state: "disabled"; recovery_flash_used: boolean; recovery_origin_readmitted: boolean; private_modes_valid: true; cleanup_complete: true; redaction_status: "passed"; };

export type LogBufferEvidence = {
  schema_version: "bitaxe-log-buffer-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  log_buffer: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  redaction_status: "passed";
};

export type PartitionLayoutEvidence = {
  schema_version: "bitaxe-partition-layout-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  partition_layout: Readonly<Record<string, unknown>>;
  ota_session: Readonly<Record<string, unknown>>;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  private_modes_valid: true;
  redaction_status: "passed";
};

export type SdkconfigRollbackObservationEvidence = {
  sdkconfig_sha256: string;
  rollback_enabled: true;
  anti_rollback_disabled: true;
  rollback_probe_isolated: true;
  interrupted_upload_attempt_count: 1;
  interrupted_upload_prefix_bytes: number;
  interruption_protocol_abort_observed: true;
  baseline_boot_session_unchanged: true;
  baseline_boot_ordinal_unchanged: true;
  baseline_build_unchanged: true;
  probe_pending_validation_observed: true;
  probe_running_partition_ota_0: true;
  rollback_running_partition_factory: true;
  final_normal_build_restored: true;
};

export type SdkconfigRollbackEvidence = {
  schema_version: "bitaxe-sdkconfig-rollback-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  rollback_probe_image_sha256: string;
  rollback_probe_metadata_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  rollback: SdkconfigRollbackObservationEvidence;
  probe_boot_session: DeviceSessionEvidence;
  rollback_session: DeviceSessionEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  normal_package_restored: true;
  recovery_flash_used: false;
  private_modes_valid: true;
  redaction_status: "passed";
};

export type NetworkReconnectObservationEvidence = {
  disconnect_event_count: 1;
  fallback_enabled: true;
  first_retry_ordinal: 1;
  configured_retry_delay_ms: 5000;
  observed_retry_delay_ms: number;
  dhcp_recovery_observed: true;
  retry_ordinal_reset: true;
  client_only_restored: true;
  stability_window_ms: 15000;
  stability_observed: true;
  api_postcondition_matches: true;
  exact_build_identity_matches: true;
};

export type NetworkReconnectEvidence = {
  schema_version: "bitaxe-network-reconnect-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_boot_session: true;
  reconnect: NetworkReconnectObservationEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  recovery_flash_used: false;
  private_modes_valid: true;
  redaction_status: "passed";
};

export type NetworkScanObservationEvidence = {
  record_count: number;
  scan_duration_ms: number;
  records_shape_valid: true;
  signal_values_valid: true;
  auth_modes_valid: true;
  exact_build_identity_matches: true;
  same_boot_session: true;
  before_after_connected: true;
  client_only_preserved: true;
  uptime_monotonic: true;
  address_family: "v6";
  address_kind: "link_local" | "unique_local" | "global";
  address_stable: true;
};

export type NetworkScanEvidence = {
  schema_version: "bitaxe-network-scan-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  same_origin_observed: true;
  scan: NetworkScanObservationEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  cleanup_complete: true;
  recovery_flash_used: false;
  private_modes_valid: true;
  redaction_status: "passed";
};

export type AsicInitializationAttemptEvidence = { campaign_result_sha256: string; diagnostics_sha256: string; observations_sha256: string; result_seal_valid: true; private_digests_valid: true; protected_modes_valid: true; };
export type AsicInitializationObservationEvidence = { planned_step_count: 9; accepted_preparation_event_count: 18; invalid_preparation_event_count: 0; terminal_preparation_step: "retain_production_uart"; terminal_preparation_outcome: "completed"; all_preparation_steps_completed: true; exactly_one_chip_detected: true; mining_ready_initialization_completed: true; production_uart_retained: true; live_initialized_work_observed: true; initialization_paths_unchanged: true; compatible_path_count: 7; };

export type AsicInitializationEvidence = {
  schema_version: "bitaxe-asic-initialization-evidence-v1";
  board: 205;
  attempt_source_commit: string;
  current_source_commit: string;
  reference_commit: string;
  source_task_sha256: string;
  workflow: WorkflowIdentity;
  attempt: AsicInitializationAttemptEvidence;
  initialization: AsicInitializationObservationEvidence;
  package_admitted: true;
  runtime_identity: "trusted";
  runtime_attestation_status: "trusted";
  serial_outcome_detail: "clean";
  campaign_terminal_category: "submit_response_observed";
  submit_outcome: "accepted";
  safety_status: "fresh";
  mine_on_boot_disabled: true;
  safe_stop_confirmed: true;
  lease_cleanup_confirmed: true;
  usb_cleanup_ready: true;
  hardware_rerun_used: false;
  redaction_status: "passed";
};

export type AsicPowerInitializationSourceEvidence = { initialization_projection_sha256: string; initialization_projection_current_commit: string; initialization_projection_valid: true; source_task_sha256: string; plan_sha256: string; };
export type AsicPowerInitializationObservationEvidence = { profile: "conservative"; frequency_mhz: 400; core_voltage_command_mv: 1100; fan_duty_command_percent: 100; preparation_step_count: 9; accepted_preparation_event_count: 18; fresh_safety_required_before_effects: true; fan_full_commanded_before_voltage: true; post_command_nonzero_fan_rpm_required: true; core_voltage_stabilization_ms: 500; asic_enable_active_low: true; reset_and_detect_completed: true; exactly_one_chip_detected_after_reset: true; mining_ready_initialization_completed: true; production_uart_retained: true; accepted_submit_observed: true; rollback_step_count: 8; rollback_attempts_all_steps: true; initial_preparation_failure_primary: true; safe_stop_asic_disable_commanded: true; unchanged_path_count: 6; semantic_path_count: 3; source_semantics_admitted: true; };
export type AsicPowerInitializationEvidence = { schema_version: "bitaxe-asic-power-initialization-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: AsicPowerInitializationSourceEvidence; power_initialization: AsicPowerInitializationObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; };

export type CoreVoltageControlSourceEvidence = { power_initialization_projection_sha256: string; power_initialization_projection_current_commit: string; power_initialization_projection_valid: true; source_task_sha256: string; plan_sha256: string; };
export type CoreVoltageControlObservationEvidence = { target_millivolts: 1100; i2c_address: 72; output_register: 248; register_code: 225; register_write_count: 1; typed_command_routed: true; stabilization_millis: 500; stabilization_before_asic_enable: true; zero_voltage_skips_ds4432u_write: true; active_low_disable: true; successful_initialized_work_observed: true; accepted_submit_observed: true; compatible_path_count: 5; reference_semantics_admitted: true; source_semantics_admitted: true; };
export type CoreVoltageControlEvidence = { schema_version: "bitaxe-core-voltage-control-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: CoreVoltageControlSourceEvidence; voltage_control: CoreVoltageControlObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; }; export type DisplayBehaviorSourceEvidence = { display_uat_projection_sha256: string; command_effects_projection_sha256: string; source_task_sha256: string; plan_sha256: string; source_semantics_admitted: true; reference_semantics_admitted: true; }; export type DisplayBehaviorObservationEvidence = { identify_request_count: 1; machine_render_confirmed: true; machine_clear_confirmed: true; operator_render_confirmed: true; operator_clear_confirmed: true; exact_panel_admitted: true; supported_rotation_count: 4; inversion_state_count: 2; timeout_mode_count: 3; retained_display_owner: true; configuration_before_first_render: true; edge_triggered_power_commands: true; display_failure_isolated: true; compatible_path_count: 5; }; export type DisplayBehaviorEvidence = { schema_version: "bitaxe-display-behavior-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: DisplayBehaviorSourceEvidence; display: DisplayBehaviorObservationEvidence; build_identity_matches: true; usb_admission_confirmed: true; safe_stop_confirmed: true; cleanup_complete: true; mining_state: "disabled"; hardware_control_state: "disabled"; hardware_rerun_used: false; redaction_status: "passed"; }; export type ScreenFlowSourceEvidence = { display_uat_projection_sha256: string; command_effects_projection_sha256: string; source_task_sha256: string; plan_sha256: string; source_semantics_admitted: true; reference_semantics_admitted: true; }; export type ScreenFlowObservationEvidence = { identify_request_count: 1; machine_render_confirmed: true; machine_clear_confirmed: true; operator_render_confirmed: true; operator_clear_confirmed: true; priority_page_count: 6; intro_page_count: 2; carousel_page_count: 4; screen_update_ms: 500; intro_delay_ms: 3000; carousel_delay_ms: 10000; notification_mask_count: 8; paused_notification_admitted: true; identify_override_admitted: true; new_block_statistics_pin_admitted: true; bounded_private_frame_admitted: true; side_effect_free_projection_admitted: true; retained_screen_owner: true; change_only_rendering: true; priority_power_visibility_admitted: true; display_failure_isolated: true; compatible_path_count: 5; }; export type ScreenFlowEvidence = { schema_version: "bitaxe-screen-flow-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: ScreenFlowSourceEvidence; screen_flow: ScreenFlowObservationEvidence; build_identity_matches: true; usb_admission_confirmed: true; safe_stop_confirmed: true; cleanup_complete: true; mining_state: "disabled"; hardware_control_state: "disabled"; hardware_rerun_used: false; redaction_status: "passed"; };
export type Ina260SourceEvidence = { system_info_projection_sha256: string; api_snapshot_sha256: string; websocket_snapshot_sha256: string; final_evidence_sha256: string; system_info_projection_valid: true; protected_modes_valid: true; hardware_plan_sha256: string; correction_plan_sha256: string; historical_source_semantics_admitted: true; current_source_semantics_admitted: true; reference_unit_semantics_admitted: true; current_source_path_count: 11; reference_path_count: 6; };
export type Ina260ObservationEvidence = { i2c_address: 64; current_register: 1; bus_voltage_register: 2; power_register: 3; complete_register_set: true; read_only_acquisition: true; historical_http_complete_fresh_sample: true; historical_websocket_complete_fresh_sample: true; historical_si_safe_ranges: true; same_historical_values: true; same_states: true; same_acquisition_stamps: true; same_boot_session: true; exact_package_identity: true; legacy_voltage_unit: string; legacy_current_unit: string; core_voltage_unit: string; power_unit: string; nominal_voltage_unit: string; volts_to_millivolts_factor: 1000; amps_to_milliamps_factor: 1000; system_info_conversion_proved: true; statistics_conversion_proved: true; campaign_min_input_millivolts: 4500; campaign_max_input_millivolts: 5500; campaign_safety_range_preserved: true; };
export type Ina260Evidence = { schema_version: "bitaxe-ina260-evidence-v2"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; package_manifest_sha256: string; workflow: WorkflowIdentity; source: Ina260SourceEvidence; telemetry: Ina260ObservationEvidence; detector_admitted: true; boot_observed: true; mining_state: "disabled"; hardware_control_state: "disabled"; cleanup_complete: true; hardware_rerun_used: false; redaction_status: "passed"; }; export type Emc2101ThermalSourceEvidence = { system_info_projection_sha256: string; api_snapshot_sha256: string; websocket_snapshot_sha256: string; plan_sha256: string; system_info_projection_valid: true; protected_modes_valid: true; production_source_current: true; source_semantics_admitted: true; compatible_path_count: 7; }; export type Emc2101ThermalObservationEvidence = { i2c_address: 76; internal_temperature_register: 0; temperature_offset_celsius: 5; read_only_acquisition: true; http_fresh_sample: true; websocket_fresh_sample: true; finite_plausible_range: true; below_throttle_threshold: true; same_temperature: true; same_state: true; same_acquisition_stamp: true; same_boot_session: true; exact_package_identity: true; }; export type Emc2101ThermalEvidence = { schema_version: "bitaxe-emc2101-thermal-evidence-v1"; board: 205; attempt_ordinal: 3; source_commit: string; reference_commit: string; package_manifest_sha256: string; workflow: WorkflowIdentity; source: Emc2101ThermalSourceEvidence; thermal: Emc2101ThermalObservationEvidence; detector_admitted: true; boot_observed: true; mining_state: "disabled"; hardware_control_state: "disabled"; cleanup_complete: true; recovery_used: false; redaction_status: "passed"; };
export type Emc2101ThermalFaultSourceEvidence = { plan_sha256: string; prior_thermal_projection_sha256: string; restore_projection_sha256: string; intent_sha256: string; protected_modes_valid: true; production_source_current: true; }; export type Emc2101ThermalFaultStimulusEvidence = { kind: "emc2101_invalid_sample"; injected_sample_count: 5; real_healthy_baseline: true; real_reads_during_injection: true; typed_invalid_outcomes: true; thermal_reading_invalid_fault: true; baseline_marker_observed: true; fault_marker_observed: true; recovery_marker_observed: true; marker_order_exact: true; intent_consumed_before_use: true; }; export type Emc2101ThermalFaultRestorationEvidence = { ordinary_wifi_seed: true; exact_package_identity: true; http_fresh_sample: true; websocket_fresh_sample: true; below_throttle_threshold: true; fault_absent: true; stimulus_not_replayed: true; }; export type Emc2101ThermalFaultEvidence = { schema_version: "bitaxe-emc2101-thermal-fault-evidence-v1"; board: 205; attempt_ordinal: 7; source_commit: string; reference_commit: string; app_elf_sha256: string; package_manifest_sha256: string; workflow: WorkflowIdentity; source: Emc2101ThermalFaultSourceEvidence; stimulus: Emc2101ThermalFaultStimulusEvidence; restoration: Emc2101ThermalFaultRestorationEvidence; detector_admitted: true; boot_observed: true; mining_state: "disabled"; hardware_control_state: "disabled"; cleanup_complete: true; recovery_used: true; redaction_status: "passed"; };
export type AsicResetSourceEvidence = { initialization_projection_sha256: string; initialization_projection_current_commit: string; initialization_projection_valid: true; source_task_sha256: string; plan_sha256: string; };
export type AsicResetObservationEvidence = { active_low: true; low_duration_ms: 100; high_duration_ms: 100; reset_and_detect_completed: true; exactly_one_chip_detected_after_reset: true; accepted_submit_observed: true; fail_closed_hold_low: true; safe_stop_hold_low: true; reset_paths_unchanged: true; compatible_path_count: 6; adapter_semantics_admitted: true; };
export type AsicResetEvidence = { schema_version: "bitaxe-asic-reset-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: AsicResetSourceEvidence; reset: AsicResetObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; };

export type AsicWorkSendSourceEvidence = { initialization_projection_sha256: string;
  initialization_projection_current_commit: string; initialization_projection_valid: true; };
export type AsicWorkSendObservationEvidence = { payload_length_bytes: 82;
  frame_length_bytes: 88; job_id_step: 8; job_id_modulus: 128;
  typed_write_frame_action: true; production_ready_gate_required: true;
  live_work_observed: true; qualified_result_observed: true; accepted_submit_observed: true;
  production_uart_retained: true; core_paths_unchanged: true; compatible_core_path_count: 3;
  dispatch_spans_unchanged: true; uart_write_span_unchanged: true; };
export type AsicWorkSendEvidence = {
  schema_version: "bitaxe-asic-work-send-evidence-v1"; board: 205;
  attempt_source_commit: string; current_source_commit: string; reference_commit: string;
  workflow: WorkflowIdentity; source: AsicWorkSendSourceEvidence;
  work_send: AsicWorkSendObservationEvidence; package_admitted: true;
  runtime_identity: "trusted"; runtime_attestation_status: "trusted";
  campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted";
  safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true;
  lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false;
  redaction_status: "passed";
};
export type AsicResultParsingSourceEvidence = { work_send_projection_sha256: string; work_send_projection_current_commit: string; work_send_projection_valid: true; };
export type AsicResultParsingObservationEvidence = { result_frame_length_bytes: 11; strict_length_validation: true; preamble_validation: true; crc_validation: true; job_lookup_validation: true; submit_nonce_little_endian: true; core_validation: true; address_interval_validation: true; version_bits_recovered: true; known_register_classification: true; typed_soft_discard_category_count: 8; soft_discard_continuation: true; live_qualified_result_observed: true; accepted_submit_observed: true; result_transport_module_unchanged: true; parser_spans_unchanged: true; adapter_nonce_span_unchanged: true; worker_nonce_span_unchanged: true; correlation_semantics_compatible: true; };
export type AsicResultParsingEvidence = { schema_version: "bitaxe-asic-result-parsing-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: AsicResultParsingSourceEvidence; result_parsing: AsicResultParsingObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; };
export type AsicSerialTransportSourceEvidence = { work_send_projection_sha256: string; work_send_projection_current_commit: string; work_send_projection_valid: true; result_parsing_projection_sha256: string; result_parsing_projection_current_commit: string; result_parsing_projection_valid: true; };
export type AsicSerialTransportObservationEvidence = { initial_baud: 115200; tx_pin: 17; rx_pin: 18; data_bits: 8; stop_bits: 1; parity_none: true; flow_control_none: true; tx_wait_timeout_ms: 1000; rx_buffer_bytes: 2048; read_chunk_max_bytes: 64; exact_write_required: true; absolute_read_deadline: true; partial_reads_accumulated: true; empty_timeout_is_idle: true; partial_timeout_clears_rx: true; live_work_tx_observed: true; live_result_rx_observed: true; accepted_submit_observed: true; uart_module_unchanged: true; adapter_module_unchanged: true; production_tx_span_compatible: true; production_rx_span_compatible: true; };
export type AsicSerialTransportEvidence = { schema_version: "bitaxe-asic-serial-transport-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: AsicSerialTransportSourceEvidence; serial_transport: AsicSerialTransportObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; };
export type AsicFrequencyTransitionSourceEvidence = { initialization_projection_sha256: string; initialization_projection_current_commit: string; initialization_projection_valid: true; };
export type AsicFrequencyTransitionObservationEvidence = { profile: "conservative"; start_frequency_mhz: 50; target_frequency_mhz: 400; step_quarter_mhz: 25; set_frequency_command_count: 56; inter_step_delay_count: 56; inter_step_delay_ms: 100; increasing: true; production_ramp_option_enabled: true; all_frequency_actions_completed: true; live_initialized_work_observed: true; accepted_submit_observed: true; ramp_modules_unchanged: true; executor_span_compatible: true; };
export type AsicFrequencyTransitionEvidence = { schema_version: "bitaxe-asic-frequency-transition-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: AsicFrequencyTransitionSourceEvidence; frequency_transition: AsicFrequencyTransitionObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; };
export type StratumSocketSourceEvidence = { initialization_projection_sha256: string; initialization_projection_current_commit: string; initialization_projection_valid: true; };
export type StratumSocketObservationEvidence = { command_capacity: 8; connect_timeout_ms: 5000; read_timeout_ms: 50; write_timeout_ms: 2000; read_buffer_bytes: 2048; tcp_nodelay_enabled: true; typed_connect_write_close_commands: true; typed_connected_bytes_failed_closed_events: true; transport_epoch_isolation: true; authorized_session_required_before_submit: true; accepted_submit_observed: true; transport_module_unchanged: true; owner_and_lifecycle_spans_compatible: true; };
export type StratumSocketEvidence = { schema_version: "bitaxe-stratum-socket-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: StratumSocketSourceEvidence; socket: StratumSocketObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; };
export type ProtocolCoordinatorSourceEvidence = { initialization_projection_sha256: string; initialization_projection_current_commit: string; initialization_projection_valid: true; work_send_projection_sha256: string; work_send_projection_current_commit: string; work_send_projection_valid: true; result_parsing_projection_sha256: string; result_parsing_projection_current_commit: string; result_parsing_projection_valid: true; socket_projection_sha256: string; socket_projection_current_commit: string; socket_projection_valid: true; };
export type ProtocolCoordinatorObservationEvidence = { owner_inbox_capacity: 16; readiness_reread_cadence_ms: 1000; readiness_gate_count: 6; single_owner_serialization: true; hardware_prepared_before_pool_access: true; authorized_before_asic_dispatch: true; qualified_result_before_submit: true; accepted_submit_observed: true; ordered_terminal_safe_stop: true; watchdog_feed_in_owner_loop: true; coordinator_modules_unchanged: true; lifecycle_spans_compatible: true; };
export type ProtocolCoordinatorEvidence = { schema_version: "bitaxe-protocol-coordinator-evidence-v1"; board: 205; attempt_source_commit: string; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: ProtocolCoordinatorSourceEvidence; coordinator: ProtocolCoordinatorObservationEvidence; package_admitted: true; runtime_identity: "trusted"; runtime_attestation_status: "trusted"; campaign_terminal_category: "submit_response_observed"; submit_outcome: "accepted"; safety_status: "fresh"; mine_on_boot_disabled: true; safe_stop_confirmed: true; lease_cleanup_confirmed: true; usb_cleanup_ready: true; hardware_rerun_used: false; redaction_status: "passed"; };
export type MiningCriteriaSourceEvidence = { phase21_summary_sha256: string; phase21_summary_valid: true; phase21_smoke_sha256: string; phase21_smoke_valid: true; phase21_soak_sha256: string; phase21_soak_valid: true; protocol_coordinator_sha256: string; protocol_coordinator_valid: true; };
export type MiningCriteriaObservationEvidence = { historical_smoke_controlled_no_share: true; historical_soak_duration_seconds: 300; historical_watchdog_passed: true; historical_telemetry_observed: true; historical_safe_stop_confirmed: true; current_duration_seconds: 600; upstream_default_profile_required: true; active_duration_accounting: true; full_duration_required: true; accepted_share_required: true; network_correlation_required: true; safe_stop_required: true; private_evidence_required: true; redaction_required: true; source_spans_compatible: true; terminal_attempt_reopened: false; };
export type MiningCriteriaEvidence = { schema_version: "bitaxe-mining-criteria-evidence-v1"; board: 205; current_source_commit: string; reference_commit: string; workflow: WorkflowIdentity; source: MiningCriteriaSourceEvidence; criteria: MiningCriteriaObservationEvidence; hardware_rerun_used: false; redaction_status: "passed"; };

export type ProvisioningNetworkObservationEvidence = {
  host_platform_macos: true;
  single_wifi_interface: true;
  initial_wifi_powered_on: true;
  initial_wifi_unassociated: true;
  baseline_candidate_count: 0;
  configuration_candidate_count: 1;
  association_observed: true;
  dhcp_observed: true;
  dns_query_count: 1;
  wildcard_dns_answer_matches_gateway: true;
  dns_ttl_seconds: 300;
  captive_redirect_observed: true;
  captive_redirect_root: true;
  captive_redirect_body_matches: true;
  api_postcondition_matches: true;
  exact_build_identity_matches: true;
};

export type ProvisioningNetworkEvidence = {
  schema_version: "bitaxe-provisioning-network-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  workflow: WorkflowIdentity;
  detector_admitted: true;
  boot_observed: true;
  provisioning: ProvisioningNetworkObservationEvidence;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  host_network_restored: true;
  device_recovery_complete: true;
  cleanup_complete: true;
  recovery_flash_used: true;
  private_modes_valid: true;
  redaction_status: "passed";
};

const automationCommands = new Set<AutomationCommand>([
  "doctor", "bootstrap-esp", "build-firmware", "package-firmware", "package-rollback-probe", "verify-reference",
  "verify-redaction", "verify-production-session", "observe-serial", "verify-flash-durability",
  "verify-firmware-ota", "verify-web-assets-ota", "verify-recovery", "verify-http-api",
  "verify-hardware-surface", "verify-mining", "capture-operator-evidence",
  "verify-settings-durability", "api-command-effects-campaign", "verify-theme-durability", "capture-correlated-runtime-evidence", "capture-version-evidence",
  "capture-operator-snapshot-evidence", "capture-runtime-health-evidence",
  "capture-system-info-evidence", "capture-adc-observation-evidence",
  "capture-hashrate-monitor-evidence", "capture-ultra205-defaults-evidence",
  "capture-settings-patch-evidence",
  "capture-statistics-history-evidence",
  "capture-log-buffer-evidence", "capture-partition-layout-evidence", "capture-sdkconfig-rollback-evidence",
  "capture-network-reconnect-evidence", "capture-network-scan-evidence",
  "project-asic-initialization-evidence",
  "project-asic-power-initialization-evidence",
  "project-core-voltage-control-evidence", "project-display-behavior-evidence", "project-screen-flow-evidence", "project-ina260-evidence", "capture-emc2101-thermal-evidence", "capture-emc2101-thermal-fault-evidence",
  "project-asic-reset-evidence",
  "project-asic-work-send-evidence",
  "project-asic-result-parsing-evidence",
  "project-asic-serial-transport-evidence",
  "project-asic-frequency-transition-evidence",
  "project-stratum-socket-evidence",
  "project-protocol-coordinator-evidence",
  "project-mining-criteria-evidence",
  "capture-provisioning-network-evidence",
]);
const automationStatuses = new Set<AutomationStatus>(["succeeded", "failed", "blocked"]);
const automationCategories = new Set<AutomationCategory>([
  "complete", "invalid_invocation", "contract_mismatch", "workspace_invalid",
  "dependency_unavailable", "policy_blocked", "authorization_blocked", "process_failed",
  "timeout", "evidence_invalid", "hardware_blocked", "package_invalid",
  "interruption_not_observed", "probe_boot_failed", "rollback_not_observed", "recovery_failed",
  "reconnect_not_observed", "reconnect_timing_invalid", "service_recovery_failed",
]);

export function parseAutomationResult(value: unknown): AutomationResult {
  if (typeof value !== "object" || value === null) throw new Error("automation result must be an object");
  const candidate = value as Record<string, unknown>;
  if (candidate["schema_version"] !== "bitaxe-automation-result-v1") throw new Error("automation result schema mismatch");
  if (
    typeof candidate["command"] !== "string" || !automationCommands.has(candidate["command"] as AutomationCommand) ||
    typeof candidate["status"] !== "string" || !automationStatuses.has(candidate["status"] as AutomationStatus) ||
    typeof candidate["category"] !== "string" || !automationCategories.has(candidate["category"] as AutomationCategory)
  ) {
    throw new Error("automation result fields are invalid");
  }
  return candidate as AutomationResult;
}

declare const commandSpecBrand: unique symbol;

export type CommandSpec<Result> = {
  readonly program: string;
  readonly args: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
  readonly result: (value: unknown) => Result;
  readonly [commandSpecBrand]: true;
};

type CommonFlashOptions = {
  board?: 205;
  port?: string;
  dryRun?: boolean;
  redactEvidence?: boolean;
  evidenceDir?: string;
};

type PackageSelection =
  | { image?: undefined; manifest?: string }
  | { image: string; manifest: string };

export type FlashOptions = CommonFlashOptions & PackageSelection & {
  wifiCredentials?: string;
};

export type MonitorOptions = CommonFlashOptions & {
  captureTimeoutSeconds?: number;
};

export type FlashMonitorOptions = CommonFlashOptions & PackageSelection & {
  wifiCredentials?: string;
  captureTimeoutSeconds?: number;
  networkReconnectProbe?: boolean; thermalFaultStimulusIntent?: string;
} & (
    | { evidenceMode?: undefined }
    | { evidenceMode: "dual"; evidenceDir: string; redactEvidence?: false }
  );

function flag(name: string, value: string | number | boolean | undefined): string[] {
  if (value === undefined || value === false) return [];
  if (value === true) return [`--${name}`];
  return [`--${name}`, String(value)];
}

function commonOptions(options: CommonFlashOptions): string[] {
  return [
    ...flag("board", options.board),
    ...flag("port", options.port),
    ...flag("dry-run", options.dryRun),
    ...flag("redact-evidence", options.redactEvidence),
    ...flag("evidence-dir", options.evidenceDir),
  ];
}

export function internalCommandSpec<Result>(
  program: string,
  args: string[],
  result: (value: unknown) => Result,
  environment?: Readonly<Record<string, string>>,
): CommandSpec<Result> {
  const spec = environment === undefined ? { program, args, result } : { program, args, result, environment };
  return spec as unknown as CommandSpec<Result>;
}
export function flashCommand(program: string, options: FlashOptions): CommandSpec<unknown> {
  return internalCommandSpec(program, ["flash", ...commonOptions(options), ...flag("image", options.image), ...flag("manifest", options.manifest), ...flag("wifi-credentials", options.wifiCredentials)], (value) => value);
}

export function monitorCommand(program: string, options: MonitorOptions): CommandSpec<unknown> {
  return internalCommandSpec(program, ["monitor", ...commonOptions(options), ...flag("capture-timeout-seconds", options.captureTimeoutSeconds)], (value) => value);
}

export function flashMonitorCommand(program: string, options: FlashMonitorOptions): CommandSpec<unknown> {
  return internalCommandSpec(program, ["flash-monitor", ...commonOptions(options), ...flag("image", options.image), ...flag("manifest", options.manifest), ...flag("wifi-credentials", options.wifiCredentials), ...flag("network-reconnect-probe", options.networkReconnectProbe), ...flag("thermal-fault-stimulus-intent", options.thermalFaultStimulusIntent), ...flag("capture-timeout-seconds", options.captureTimeoutSeconds), ...flag("evidence-mode", options.evidenceMode)], (value) => value);
}
