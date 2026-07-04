# Phase 21 Live Mining Enablement

controlled_live_mining_package_status: ready
controlled_runtime_harness_status: ready
controlled_runtime_contract_tests: passed
runtime_required_log_markers: phase21_controlled_runtime_status, stratum_subscribe_status, stratum_authorize_status, stratum_notify_status, bm1366_work_dispatch_status, result_receive_status, share_submission_status, runtime_snapshot_status, api_websocket_telemetry_update_status, safe_stop_status
readiness_status: blocked_by_default
enablement_mode: live-mining-runtime
hardware_evidence_ack: ultra205-live-mining-runtime-safe-bench
package_manifest: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement/package/bitaxe-ultra205-package.json
source_commit: b5502a6306377183134afa223256997bfab9f6ae
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
recovery_steps: stop wrapper, power-cycle board if serial stalls, reflash default safe package if live mode does not safe-stop
prerequisite_artifacts: readiness-audit.md
evidence_dir: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement
redaction_reviewer: required-before-citation
post_action_safe_state_marker: safe_state: mining=disabled
hardware_control=disabled
work_submission=disabled
