#!/usr/bin/env bash
# Phase 28.1.1.6 version-rolling diagnostic: baseline compare + forced work-field A/B.
# Forced A/B label: negotiated_version_mask_work_field_parity (Wave 0 / D-04 locked).
# Does not recommend or enable falsified levers (job-byte / poll / long-block /
# post_max_baud / ticket_mask_asic_difficulty / count_asic_chips_rx_loop_parity).
# Does NOT implement ASIC mask reload (D-05/D-14).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
phase_dir=".planning/phases/28.1.1.6-bm1366-version-rolling-negotiation-nonce-production-diagnosi"
source_work_root="scratch/phase28.1.1.1-source-work"
# Forced single A/B (D-11 / RESEARCH PRIMARY): negotiated_version_mask_work_field_parity.
forced_ab_label="negotiated_version_mask_work_field_parity"
port=""
wifi_credentials=""
duration_seconds="360"
capture_dir=""
dry_run="false"

usage() {
	printf 'usage: %s --port PATH --wifi-credentials PATH --duration-seconds N --capture-dir PATH [--dry-run]\n' "$(basename "$0")"
	printf '  Baseline version-rolling compare from Phase 28.1.1.1 source-work logs, then one\n'
	printf '  forced Rust A/B for negotiated_version_mask_work_field_parity (mask-on-MiningWork).\n'
}

die() {
	printf 'version_rolling_diagnostic_error: %s\n' "$*" >&2
	exit 1
}

while (($#)); do
	case "$1" in
	--port)
		[[ $# -ge 2 ]] || die "missing value for --port"
		port="${2:-}"
		shift 2
		;;
	--wifi-credentials)
		[[ $# -ge 2 ]] || die "missing value for --wifi-credentials"
		wifi_credentials="${2:-}"
		shift 2
		;;
	--duration-seconds)
		[[ $# -ge 2 ]] || die "missing value for --duration-seconds"
		duration_seconds="${2:-}"
		shift 2
		;;
	--capture-dir)
		[[ $# -ge 2 ]] || die "missing value for --capture-dir"
		capture_dir="${2:-}"
		shift 2
		;;
	--dry-run)
		dry_run="true"
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		die "unknown argument: $1"
		;;
	esac
done

[[ -n "$port" ]] || die "--port is required"
[[ -n "$wifi_credentials" ]] || die "--wifi-credentials is required"
[[ -n "$capture_dir" ]] || die "--capture-dir is required"
[[ "$duration_seconds" =~ ^[0-9]+$ ]] || die "--duration-seconds must be an integer"
((duration_seconds >= 360)) || die "--duration-seconds must be at least 360"

cd "$repo_root"
[[ -f "$wifi_credentials" ]] || die "Wi-Fi credential file is missing"
mkdir -p "$capture_dir"
git check-ignore -q "$capture_dir/probe.raw.log" ||
	die "capture directory must be ignored by git: $capture_dir"

fake_pool_pids=()
last_fake_pool_pid=""
last_fake_pool_ready_json=""

cleanup() {
	local pid
	for pid in "${fake_pool_pids[@]:-}"; do
		kill "$pid" 2>/dev/null || true
		wait "$pid" 2>/dev/null || true
	done
}
trap cleanup EXIT

if [[ "$dry_run" == "true" ]]; then
	printf 'dry_run_status=passed\n'
	printf 'capture_dir_ignored=true\n'
	printf 'credential_values_printed=false\n'
	printf 'forced_ab_label=%s\n' "$forced_ab_label"
	printf 'falsified_levers_recommended=false\n'
	printf 'asic_mask_reload_applied=false\n'
	printf 'job_byte_patch_applied=false\n'
	exit 0
fi

detect_fake_pool_host() {
	if [[ -n "${PHASE28_FAKE_POOL_HOST:-}" ]]; then
		printf '%s\n' "$PHASE28_FAKE_POOL_HOST"
		return
	fi

	if command -v ipconfig >/dev/null 2>&1 && command -v route >/dev/null 2>&1; then
		local iface
		iface="$(route -n get default 2>/dev/null | awk '/interface:/{print $2; exit}')"
		if [[ -n "$iface" ]]; then
			ipconfig getifaddr "$iface" 2>/dev/null && return
		fi
	fi

	if command -v ip >/dev/null 2>&1; then
		local maybe_ip
		maybe_ip="$(ip route get 1.1.1.1 2>/dev/null |
			awk '{for (i = 1; i <= NF; i++) if ($i == "src") {print $(i + 1); exit}}')"
		if [[ -n "$maybe_ip" ]]; then
			printf '%s\n' "$maybe_ip"
			return
		fi
	fi

	return 1
}

wait_for_ready_json() {
	local ready_json="$1"
	local deadline=$((SECONDS + 30))
	while ((SECONDS < deadline)); do
		if [[ -s "$ready_json" ]]; then
			return 0
		fi
		sleep 1
	done
	return 1
}

start_fake_pool() {
	local session_label="$1"
	local session_dir="$2"
	local ready_json="$session_dir/ready.json"
	local report_json="$session_dir/report.json"
	local stdout_log="$session_dir/fake-pool.stdout.log"
	local stderr_log="$session_dir/fake-pool.stderr.log"
	local fake_pool_duration="${PHASE28_FAKE_POOL_DURATION_SECONDS:-2400}"

	mkdir -p "$session_dir"
	node scripts/phase28.1.1.1-fake-stratum-pool.mjs \
		--host 0.0.0.0 \
		--port 0 \
		--fixture phase28-source-work-v1 \
		--session-label "$session_label" \
		--ready-json "$ready_json" \
		--report-json "$report_json" \
		--duration-seconds "$fake_pool_duration" \
		>"$stdout_log" 2>"$stderr_log" &
	local pid=$!
	fake_pool_pids+=("$pid")

	wait_for_ready_json "$ready_json" || die "$session_label fake pool did not become ready"
	last_fake_pool_pid="$pid"
	last_fake_pool_ready_json="$ready_json"
}

stop_fake_pool_pid() {
	local pid="$1"
	kill "$pid" 2>/dev/null || true
	wait "$pid" 2>/dev/null || true
}

find_latest_aligned_capture() {
	local candidate
	while IFS= read -r candidate; do
		if [[ -f "$candidate/upstream-capture/upstream-monitor.raw.log" &&
			-f "$candidate/rust-capture/flash-monitor.log" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done < <(ls -td "$source_work_root"/source-work-aligned-* 2>/dev/null || true)
	return 1
}

field_from_report() {
	local field="$1"
	local report="$2"
	sed -n "s/^${field}: //p" "$report" | tail -1
}

fake_pool_submit_observed() {
	local report_json="$1"
	node - "$report_json" <<'NODE'
const fs = require("node:fs");
const reportPath = process.argv[2];
const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
process.stdout.write(report.submit_observed ? "true" : "false");
NODE
}

copy_report_to_phase() {
	local report="$1"
	local basename_out="$2"
	local phase_report="$phase_dir/$basename_out"
	mkdir -p "$phase_dir"
	cp "$report" "$phase_report"
	printf '%s\n' "$phase_report"
}

run_version_rolling_compare() {
	local upstream_log="$1"
	local rust_log="$2"
	local out_report="$3"
	node scripts/phase28.1.1.6-version-rolling-compare.mjs \
		--upstream "$upstream_log" \
		--rust "$rust_log" \
		--out "$out_report"
}

write_ab_report() {
	local ab_dir="$1"
	local upstream_log="$2"
	local rust_log="$3"
	local ab_submit="$4"
	local ab_status="$5"
	local source_commit="$6"
	local reference_commit="$7"
	local compare_report="$ab_dir/version-rolling-ab-compare-redacted.md"
	local ab_report="$ab_dir/28.1.1.6-ab-${forced_ab_label}-redacted.md"

	run_version_rolling_compare "$upstream_log" "$rust_log" "$compare_report"

	local configure_observed
	configure_observed="$(field_from_report configure_observed "$compare_report")"
	local mask_stored_class
	mask_stored_class="$(field_from_report mask_stored_class "$compare_report")"
	local mask_applied_to_work
	mask_applied_to_work="$(field_from_report mask_applied_to_work "$compare_report")"
	local job_version_field_class
	job_version_field_class="$(field_from_report job_version_field_class "$compare_report")"
	local version_mask_tx_class
	version_mask_tx_class="$(field_from_report version_mask_tx_class "$compare_report")"
	local version_bits_on_submit_path
	version_bits_on_submit_path="$(field_from_report version_bits_on_submit_path "$compare_report")"
	local power_delta_class
	power_delta_class="$(field_from_report power_delta_class "$compare_report")"
	local result_correlated
	result_correlated="$(field_from_report result_correlated "$compare_report")"
	local wire_parity_ticket_mask_retained
	wire_parity_ticket_mask_retained="$(field_from_report wire_parity_ticket_mask_retained "$compare_report")"
	local wire_parity_rx_loop_retained
	wire_parity_rx_loop_retained="$(field_from_report wire_parity_rx_loop_retained "$compare_report")"
	local recommended
	recommended="$(field_from_report recommended_investigation "$compare_report")"

	[[ -n "$configure_observed" ]] || configure_observed="false"
	[[ -n "$mask_stored_class" ]] || mask_stored_class="unavailable"
	[[ -n "$mask_applied_to_work" ]] || mask_applied_to_work="false"
	[[ -n "$job_version_field_class" ]] || job_version_field_class="unavailable"
	[[ -n "$version_mask_tx_class" ]] || version_mask_tx_class="unavailable"
	[[ -n "$version_bits_on_submit_path" ]] || version_bits_on_submit_path="unavailable"
	[[ -n "$power_delta_class" ]] || power_delta_class="unavailable"
	[[ -n "$result_correlated" ]] || result_correlated="false"
	[[ -n "$wire_parity_ticket_mask_retained" ]] || wire_parity_ticket_mask_retained="true"
	[[ -n "$wire_parity_rx_loop_retained" ]] || wire_parity_rx_loop_retained="true"
	[[ -n "$recommended" ]] || recommended="pool_negotiated_mask_asic_reload"

	local ab_outcome="unchanged"
	if [[ "$ab_status" == "failed" || "$ab_status" == "blocked_safe_prerequisite" ]]; then
		ab_outcome="blocked_safe_prerequisite"
	elif [[ "$ab_submit" == "true" || "$result_correlated" == "true" ]]; then
		ab_outcome="improved"
	fi

	{
		printf '# Phase 28.1.1.6 A/B: negotiated_version_mask_work_field_parity\n\n'
		printf 'ab_label: %s\n' "$forced_ab_label"
		printf 'board: 205\n'
		printf 'port_selected: detector_gated\n'
		printf 'source_commit: %s\n' "$source_commit"
		printf 'reference_commit: %s\n' "$reference_commit"
		printf 'capture_timeout_seconds: %s\n' "$duration_seconds"
		printf 'pool_config: synthetic_fake_pool\n'
		printf 'wifi_credentials_used_for_network_join_only: true\n'
		printf 'real_pool_credentials_used: false\n'
		printf 'raw_bytes_committed: false\n'
		printf 'raw_logs_committed: false\n'
		printf 'credential_contents_read: false\n'
		printf 'ab_capture_status: %s\n' "$ab_status"
		printf 'configure_observed: %s\n' "$configure_observed"
		printf 'mask_stored_class: %s\n' "$mask_stored_class"
		printf 'mask_applied_to_work: %s\n' "$mask_applied_to_work"
		printf 'job_version_field_class: %s\n' "$job_version_field_class"
		printf 'version_mask_tx_class: %s\n' "$version_mask_tx_class"
		printf 'version_bits_on_submit_path: %s\n' "$version_bits_on_submit_path"
		printf 'power_delta_class: %s\n' "$power_delta_class"
		printf 'fake_pool_submit_observed: %s\n' "$ab_submit"
		printf 'result_correlated: %s\n' "$result_correlated"
		printf 'ab_outcome: %s\n' "$ab_outcome"
		printf 'wire_parity_ticket_mask_retained: %s\n' "$wire_parity_ticket_mask_retained"
		printf 'wire_parity_rx_loop_retained: %s\n' "$wire_parity_rx_loop_retained"
		printf 'recommended_investigation: %s\n' "$recommended"
		printf 'asic_mask_reload_applied: false\n'
		printf 'job_byte_patch_applied: false\n'
		printf 'post_max_baud_delay_2000_applied: false\n'
		printf 'match_upstream_register_read_poll_applied: false\n'
		printf 'upstream_like_long_block_receive_applied: false\n'
		printf 'ticket_mask_asic_difficulty_sole_blocker_applied: false\n'
		printf 'count_asic_chips_rx_loop_parity_applied: false\n'
		printf 'phase30_promotion_input: pending\n'
		printf '\n## Notes\n\n'
		printf 'Forced A/B for negotiated_version_mask_work_field_parity: mask attached to MiningWork;\n'
		printf 'BM1366 UART job version remains base notify LE bytes; no ASIC reload; no job-byte patch.\n'
		printf 'improved requires result_correlated and/or fake_pool_submit_observed (D-12).\n'
		printf 'power_delta_class is corroboration only. Fake-pool mask equals init default (Pitfall 5).\n'
		printf 'ASIC-256 ticket mask + RX-loop wire parity retained (D-14).\n'
	} >"$ab_report"

	copy_report_to_phase "$ab_report" "28.1.1.6-ab-${forced_ab_label}-redacted.md" >/dev/null
	printf '%s\n' "$ab_outcome"
}

run_rust_ab_capture() {
	local upstream_log="$1"
	local fake_pool_host="$2"
	local source_commit="$3"
	local reference_commit="$4"
	local ab_dir="$capture_dir/ab-${forced_ab_label}"
	local fake_pool_dir="$ab_dir/rust-fake-pool"

	start_fake_pool "rust" "$fake_pool_dir"
	local ready_json="$last_fake_pool_ready_json"
	local fake_pool_pid="$last_fake_pool_pid"
	local pool_credentials="$fake_pool_dir/synthetic-pool-credentials.json"
	node scripts/phase28.1.1.1-synthetic-pool-credentials.mjs \
		--ready-json "$ready_json" \
		--host "$fake_pool_host" \
		--out "$pool_credentials"

	local rust_capture_dir="$ab_dir/rust-capture"
	# Candidate is the mask-on-MiningWork patch already in tree (D-04/D-11).
	# No ASIC reload, no second lever.
	if ! bash scripts/phase28.1.1-wire-parity-capture.sh \
		"$port" \
		"$rust_capture_dir" \
		"$duration_seconds" \
		--wifi-credentials "$wifi_credentials" \
		--pool-credentials "$pool_credentials" \
		>"$ab_dir/rust-helper.stdout.log" 2>"$ab_dir/rust-helper.stderr.log"; then
		stop_fake_pool_pid "$fake_pool_pid"
		write_ab_report "$ab_dir" "$upstream_log" "$rust_capture_dir/flash-monitor.log" "false" "failed" \
			"$source_commit" "$reference_commit" >/dev/null || true
		printf 'ab_capture_failed\n'
		return 0
	fi
	stop_fake_pool_pid "$fake_pool_pid"

	local ab_submit
	ab_submit="$(fake_pool_submit_observed "$fake_pool_dir/report.json")"
	write_ab_report "$ab_dir" "$upstream_log" "$rust_capture_dir/flash-monitor.log" "$ab_submit" "complete" \
		"$source_commit" "$reference_commit" >/dev/null
	printf '%s\n' "$ab_submit"
}

write_blocked_evidence() {
	local reason="$1"
	mkdir -p "$phase_dir"
	{
		printf '# Phase 28.1.1.6 Version-Rolling Diagnostic — Blocked\n\n'
		printf 'comparison_status: blocked_safe_prerequisite\n'
		printf 'blocker: %s\n' "$reason"
		printf 'recommended_investigation: negotiated_version_mask_work_field_parity\n'
		printf 'forced_ab_label: %s\n' "$forced_ab_label"
		printf 'configure_observed: false\n'
		printf 'mask_stored_class: unavailable\n'
		printf 'mask_applied_to_work: false\n'
		printf 'job_version_field_class: unavailable\n'
		printf 'version_mask_tx_class: unavailable\n'
		printf 'version_bits_on_submit_path: unavailable\n'
		printf 'power_delta_class: unavailable\n'
		printf 'fake_pool_submit_observed: false\n'
		printf 'result_correlated: false\n'
		printf 'raw_bytes_committed: false\n'
		printf 'credential_contents_read: false\n'
		printf 'asic_mask_reload_applied: false\n'
		printf 'job_byte_patch_applied: false\n'
		printf 'phase30_promotion_input: pending\n'
	} >"$phase_dir/28.1.1.6-version-rolling-redacted.md"
	{
		printf '# Phase 28.1.1.6 A/B: negotiated_version_mask_work_field_parity\n\n'
		printf 'ab_label: %s\n' "$forced_ab_label"
		printf 'board: 205\n'
		printf 'port_selected: detector_gated\n'
		printf 'ab_capture_status: blocked_safe_prerequisite\n'
		printf 'capture_timeout_seconds: %s\n' "$duration_seconds"
		printf 'configure_observed: false\n'
		printf 'mask_stored_class: unavailable\n'
		printf 'mask_applied_to_work: false\n'
		printf 'job_version_field_class: unavailable\n'
		printf 'version_mask_tx_class: unavailable\n'
		printf 'version_bits_on_submit_path: unavailable\n'
		printf 'power_delta_class: unavailable\n'
		printf 'fake_pool_submit_observed: false\n'
		printf 'result_correlated: false\n'
		printf 'ab_outcome: blocked_safe_prerequisite\n'
		printf 'wire_parity_ticket_mask_retained: true\n'
		printf 'wire_parity_rx_loop_retained: true\n'
		printf 'blocker: %s\n' "$reason"
		printf 'raw_bytes_committed: false\n'
		printf 'credential_contents_read: false\n'
		printf 'asic_mask_reload_applied: false\n'
		printf 'job_byte_patch_applied: false\n'
		printf 'post_max_baud_delay_2000_applied: false\n'
		printf 'match_upstream_register_read_poll_applied: false\n'
		printf 'upstream_like_long_block_receive_applied: false\n'
		printf 'ticket_mask_asic_difficulty_sole_blocker_applied: false\n'
		printf 'count_asic_chips_rx_loop_parity_applied: false\n'
		printf 'phase30_promotion_input: pending\n'
	} >"$phase_dir/28.1.1.6-ab-${forced_ab_label}-redacted.md"
}

just verify-reference >"$capture_dir/verify-reference.raw.log" 2>&1
just detect-ultra205 >"$capture_dir/detect-ultra205.raw.log" 2>&1
detected_port="$(sed -n 's/^port=//p' "$capture_dir/detect-ultra205.raw.log" | tail -1)"
if [[ -z "$detected_port" ]]; then
	write_blocked_evidence "detect_ultra205_failed"
	die "detect-ultra205 did not report a port"
fi
[[ "$detected_port" == "$port" ]] ||
	die "detected Ultra 205 port did not match requested port"

source_commit="$(git rev-parse HEAD)"
reference_commit="$(git -C reference/esp-miner rev-parse HEAD 2>/dev/null || printf 'unavailable')"

aligned_capture_dir="$(find_latest_aligned_capture || true)"
if [[ -z "$aligned_capture_dir" ]]; then
	aligned_capture_dir="$capture_dir/source-work-aligned-rerun"
	bash scripts/phase28.1.1.1-source-work-aligned-capture.sh \
		--port "$port" \
		--wifi-credentials "$wifi_credentials" \
		--duration-seconds "$duration_seconds" \
		--capture-dir "$aligned_capture_dir" \
		>"$capture_dir/source-work-rerun.stdout.log" 2>"$capture_dir/source-work-rerun.stderr.log"
fi

upstream_log="$aligned_capture_dir/upstream-capture/upstream-monitor.raw.log"
baseline_rust_log="$aligned_capture_dir/rust-capture/flash-monitor.log"
[[ -f "$upstream_log" && -f "$baseline_rust_log" ]] || {
	write_blocked_evidence "source_work_logs_missing"
	die "aligned source-work logs missing"
}

baseline_report="$capture_dir/version-rolling-redacted.md"
run_version_rolling_compare "$upstream_log" "$baseline_rust_log" "$baseline_report"
phase_baseline="$(copy_report_to_phase "$baseline_report" "28.1.1.6-version-rolling-redacted.md")"

fake_pool_host="$(detect_fake_pool_host)" || {
	write_blocked_evidence "fake_pool_host_unavailable"
	die "could not determine a host address reachable by the board; set PHASE28_FAKE_POOL_HOST"
}

ab_submit="$(run_rust_ab_capture "$upstream_log" "$fake_pool_host" "$source_commit" "$reference_commit")"
ab_status="complete"
if [[ "$ab_submit" == "ab_capture_failed" ]]; then
	ab_status="failed"
	ab_submit="false"
fi

printf 'version_rolling_diagnostic_status=complete\n'
printf 'baseline_report=%s\n' "$phase_baseline"
printf 'forced_ab_label=%s\n' "$forced_ab_label"
printf 'ab_capture_status=%s\n' "$ab_status"
printf 'ab_fake_pool_submit_observed=%s\n' "$ab_submit"
printf 'credential_values_printed=false\n'
printf 'asic_mask_reload_applied=false\n'
printf 'job_byte_patch_applied=false\n'
