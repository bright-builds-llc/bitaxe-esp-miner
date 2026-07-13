#!/usr/bin/env bash
# shellcheck source=scripts/phase28.1.1-terminal-closure-guard.sh
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# Diagnose Phase 28.1.1.1 below-job-byte BM1366 sequencing without committing raw logs.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
phase_dir=".planning/phases/28.1.1.1-bm1366-upstream-golden-comparator-and-nonce-production-gap-r"
source_work_root="scratch/phase28.1.1.1-source-work"
port=""
wifi_credentials=""
duration_seconds="360"
capture_dir=""
dry_run="false"

usage() {
	printf 'usage: %s --port PATH --wifi-credentials PATH --duration-seconds N --capture-dir PATH [--dry-run]\n' "$(basename "$0")"
	printf '  Reuses deterministic source-work logs when valid and runs one targeted Rust A/B capture when recommended.\n'
}

die() {
	printf 'below_job_byte_diagnostic_error: %s\n' "$*" >&2
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

write_evidence() {
	local status="$1"
	local blocker="$2"
	local sequence_report="$3"
	local recommendation="$4"
	local ab_status="$5"
	local ab_submit="$6"
	local evidence_path="$phase_dir/28.1.1.1-below-job-byte-diagnostic-redacted.md"

	mkdir -p "$phase_dir"
	{
		printf '# Phase 28.1.1.1 Below-Job-Byte Diagnostic Evidence\n\n'
		printf 'status: %s\n' "$status"
		printf 'blocker: %s\n' "$blocker"
		printf 'sequence_report: %s\n' "$(basename "$sequence_report")"
		printf 'recommended_investigation: %s\n' "$recommendation"
		printf 'ab_capture_status: %s\n' "$ab_status"
		printf 'ab_fake_pool_submit_observed: %s\n' "$ab_submit"
		printf 'raw_logs_committed: false\n'
		printf 'credential_contents_read: false\n'
		printf 'wifi_credentials_used_for_network_join_only: true\n'
		printf 'real_pool_credentials_used: false\n'
		printf '\n## Conclusion\n\n'
		if [[ "$status" == "captured_gaps_found" ]]; then
			printf 'The deterministic source-work job-byte comparison remains the controlling evidence: BM1366 job fields match, and this diagnostic did not justify a job-construction patch.\n\n'
			printf 'The remaining blocker is below job bytes. Continue with the recorded recommended investigation only if it produced a narrower A/B signal.\n'
		else
			printf 'The diagnostic stopped before a conclusive below-job-byte capture. No Rust patch is justified from this evidence.\n'
		fi
	} >"$evidence_path"
}

copy_report_to_phase() {
	local report="$1"
	local basename_out="$2"
	local phase_report="$phase_dir/$basename_out"
	mkdir -p "$phase_dir"
	cp "$report" "$phase_report"
	printf '%s\n' "$phase_report"
}

run_sequence_compare() {
	local upstream_log="$1"
	local rust_log="$2"
	local out_report="$3"
	node scripts/phase28.1.1.1-below-job-byte-sequence-compare.mjs \
		--upstream "$upstream_log" \
		--rust "$rust_log" \
		--out "$out_report"
}

run_rust_ab_capture() {
	local recommendation="$1"
	local upstream_log="$2"
	local ab_dir="$capture_dir/ab-${recommendation}"
	local fake_pool_dir="$ab_dir/rust-fake-pool"
	local fake_pool_host="$3"

	start_fake_pool "rust" "$fake_pool_dir"
	local ready_json="$last_fake_pool_ready_json"
	local fake_pool_pid="$last_fake_pool_pid"
	local pool_credentials="$fake_pool_dir/synthetic-pool-credentials.json"
	node scripts/phase28.1.1.1-synthetic-pool-credentials.mjs \
		--ready-json "$ready_json" \
		--host "$fake_pool_host" \
		--out "$pool_credentials"

	local rust_capture_dir="$ab_dir/rust-capture"
	if ! bash scripts/phase28.1.1-wire-parity-capture.sh \
		"$port" \
		"$rust_capture_dir" \
		"$duration_seconds" \
		--wifi-credentials "$wifi_credentials" \
		--pool-credentials "$pool_credentials" \
		--investigation "$recommendation" \
		>"$ab_dir/rust-helper.stdout.log" 2>"$ab_dir/rust-helper.stderr.log"; then
		stop_fake_pool_pid "$fake_pool_pid"
		printf 'ab_capture_failed\n'
		return 0
	fi
	stop_fake_pool_pid "$fake_pool_pid"

	local ab_report="$ab_dir/below-job-byte-sequence-redacted.md"
	run_sequence_compare "$upstream_log" "$rust_capture_dir/flash-monitor.log" "$ab_report"
	copy_report_to_phase "$ab_report" "28.1.1.1-below-job-byte-ab-${recommendation}-redacted.md" >/dev/null
	fake_pool_submit_observed "$fake_pool_dir/report.json"
}

just verify-reference >"$capture_dir/verify-reference.raw.log" 2>&1
just detect-ultra205 >"$capture_dir/detect-ultra205.raw.log" 2>&1
detected_port="$(sed -n 's/^port=//p' "$capture_dir/detect-ultra205.raw.log" | tail -1)"
[[ "$detected_port" == "$port" ]] ||
	die "detected Ultra 205 port did not match requested port"

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
rust_log="$aligned_capture_dir/rust-capture/flash-monitor.log"
sequence_report="$capture_dir/below-job-byte-sequence-redacted.md"
run_sequence_compare "$upstream_log" "$rust_log" "$sequence_report"
phase_sequence_report="$(copy_report_to_phase "$sequence_report" "28.1.1.1-below-job-byte-sequence-redacted.md")"

comparison_status="$(field_from_report comparison_status "$sequence_report")"
recommendation="$(field_from_report recommended_investigation "$sequence_report")"
ab_status="not_run"
ab_submit="false"

case "$comparison_status" in
blocked_safe_prerequisite)
	write_evidence "blocked_safe_prerequisite" "below_job_byte_sequence_incomplete" "$phase_sequence_report" "$recommendation" "$ab_status" "$ab_submit"
	die "below-job-byte sequence comparator blocked"
	;;
match | match_with_downstream_gap | mismatch) ;;
*)
	write_evidence "blocked_safe_prerequisite" "below_job_byte_sequence_unknown_status" "$phase_sequence_report" "$recommendation" "$ab_status" "$ab_submit"
	die "unknown sequence comparison status: $comparison_status"
	;;
esac

if [[ "$recommendation" != "none" ]]; then
	case "$recommendation" in
	post_max_baud_delay_2000 | clear_rx_before_production_work | single_dispatch_bounded_read)
		fake_pool_host="$(detect_fake_pool_host)" ||
			die "could not determine a host address reachable by the board; set PHASE28_FAKE_POOL_HOST"
		ab_submit="$(run_rust_ab_capture "$recommendation" "$upstream_log" "$fake_pool_host")"
		if [[ "$ab_submit" == "ab_capture_failed" ]]; then
			ab_status="failed"
			ab_submit="false"
		else
			ab_status="complete"
		fi
		;;
	*)
		ab_status="skipped_unknown_recommendation"
		;;
	esac
fi

write_evidence "captured_gaps_found" "rust_result_share_evidence_missing" "$phase_sequence_report" "$recommendation" "$ab_status" "$ab_submit"
printf 'below_job_byte_diagnostic_status=captured_gaps_found\n'
printf 'sequence_report=%s\n' "$phase_sequence_report"
printf 'recommended_investigation=%s\n' "$recommendation"
printf 'ab_capture_status=%s\n' "$ab_status"
printf 'ab_fake_pool_submit_observed=%s\n' "$ab_submit"
