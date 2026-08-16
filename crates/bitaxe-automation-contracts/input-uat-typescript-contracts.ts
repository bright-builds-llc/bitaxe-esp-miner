// Generated from bitaxe-automation-contracts. Do not hand-edit.
export type InputUatObservationEvidence = {
  gpio: 0;
  active_low: true;
  pull_up_enabled: true;
  sampling_ms: 10;
  debounce_ms: 30;
  long_press_ms: 2000;
  checkpoint_published_before_input: true;
  physical_short_click_count: 1;
  screen_advance_observed: true;
  long_press_observed: false;
};
export type InputUatEvidence = {
  schema_version: "bitaxe-input-uat-evidence-v1";
  board: 205;
  source_commit: string;
  reference_commit: string;
  app_elf_sha256: string;
  package_manifest_sha256: string;
  plan_sha256: string;
  input: InputUatObservationEvidence;
  exact_package_flash_completed: true;
  runtime_attestation_trusted: true;
  source_semantics_admitted: true;
  reference_semantics_admitted: true;
  usb_admission_confirmed: true;
  cleanup_complete: true;
  mining_state: "disabled";
  hardware_control_state: "disabled";
  serial_transcript_retained: false;
  redaction_status: "passed";
};
