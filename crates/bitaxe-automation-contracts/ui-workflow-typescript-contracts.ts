export type UiWorkflowSourceEvidence = {
  operator_snapshot_evidence_sha256: string;
  browser_attestation_sha256: string;
  theme_evidence_sha256: string;
  settings_evidence_sha256: string;
  log_evidence_sha256: string;
  partition_evidence_sha256: string;
  rollback_evidence_sha256: string;
  implementation_result_sha256: string;
  static_ui_contract_sha256: string;
  prior_plan_sha256: string;
  prior_closure_sha256: string;
  current_plan_sha256: string;
  compatibility_source_set_sha256: string;
  compatibility_path_count: number;
  all_source_evidence_valid: boolean;
  joined_source_commits_ancestral: boolean;
  attempt_source_ancestral: boolean;
  compatibility_paths_unchanged: boolean;
  compatibility_paths_clean: boolean;
};

export type UiWorkflowBrowserEvidence = {
  expected_route_count: number;
  desktop_route_count: number;
  mobile_route_count: number;
  same_origin_requests_observed: boolean;
  log_websocket_observed: boolean;
  mobile_navigation_opened: boolean;
  mobile_navigation_closed: boolean;
  write_only_secrets_blank: boolean;
  no_file_update_disabled: boolean;
  otawww_unavailable: boolean;
  console_error_count: number;
  unexpected_request_failure_count: number;
  desktop_viewport_observed: boolean;
  mobile_viewport_observed: boolean;
  browser_cleanup_complete: boolean;
};

export type UiWorkflowEvidence = {
  schema_version: "bitaxe-ui-workflow-evidence-v1";
  board: 205;
  attempt_source_commit: string;
  projector_source_commit: string;
  reference_commit: string;
  package_manifest_sha256: string;
  app_elf_sha256: string;
  www_spiffs_sha256: string;
  workflow: {
    schema_version: "bitaxe-workflow-identity-v1";
    command: "project-ui-workflow-evidence";
    request_sha256: string;
  };
  sources: UiWorkflowSourceEvidence;
  browser: UiWorkflowBrowserEvidence;
  exact_package_observed: boolean;
  normal_restart_observed: boolean;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  device_cleanup_complete: boolean;
  private_modes_valid: boolean;
  hardware_rerun_used: false;
  redaction_status: "passed";
};
