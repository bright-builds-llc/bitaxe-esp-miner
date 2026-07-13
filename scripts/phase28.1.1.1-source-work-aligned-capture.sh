#!/usr/bin/env bash
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# Capture upstream and Rust BM1366 job frames against the same synthetic Stratum work.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
phase_dir=".planning/phases/28.1.1.1-bm1366-upstream-golden-comparator-and-nonce-production-gap-r"
port=""
wifi_credentials=""
duration_seconds="360"
capture_dir=""
dry_run="false"
investigation_mode=""
chip_detect_investigation_mode=""

usage() {
	printf 'usage: %s --port PATH --wifi-credentials PATH --duration-seconds N --capture-dir PATH [--investigation MODE] [--chip-detect-investigation MODE] [--dry-run]\n' "$(basename "$0")"
	printf '  Runs upstream and Rust captures against a deterministic local fake Stratum pool.\n'
	printf '  Raw logs and generated credentials stay under ignored scratch/hardware-run paths.\n'
}

die() {
	printf 'source_work_aligned_capture_error: %s\n' "$*" >&2
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
	--investigation)
		[[ $# -ge 2 ]] || die "missing value for --investigation"
		investigation_mode="${2:-}"
		shift 2
		;;
	--chip-detect-investigation)
		[[ $# -ge 2 ]] || die "missing value for --chip-detect-investigation"
		chip_detect_investigation_mode="${2:-}"
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

assert_fake_pool_report_ready() {
	local report_json="$1"
	local label="$2"

	node - "$report_json" "$label" <<'NODE'
const fs = require("node:fs");
const [reportPath, label] = process.argv.slice(2);
const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
const failures = [];
for (const [field, expected] of [
  ["configure_observed", true],
  ["subscribe_observed", true],
  ["authorize_observed", true],
]) {
  if (report[field] !== expected) failures.push(field);
}
if (!Number.isInteger(report.notify_sent_count) || report.notify_sent_count < 1) {
  failures.push("notify_sent_count");
}
if (typeof report.source_work_fingerprint !== "string" || report.source_work_fingerprint.length !== 64) {
  failures.push("source_work_fingerprint");
}
if (failures.length > 0) {
  console.error(`${label}_fake_pool_report_failed=${failures.join(",")}`);
  process.exit(1);
}
NODE
}

write_evidence() {
	local status="$1"
	local blocker="$2"
	local comparator_report="${3:-}"
	local evidence_path="$phase_dir/28.1.1.1-source-work-alignment-evidence-redacted.md"

	mkdir -p "$phase_dir"
	{
		printf '# Phase 28.1.1.1 Source-Work Alignment Evidence\n\n'
		printf 'status: %s\n' "$status"
		printf 'blocker: %s\n' "$blocker"
		printf 'source_work_fixture: phase28-source-work-v1\n'
		printf 'raw_logs_committed: false\n'
		printf 'credential_contents_read: false\n'
		printf 'real_pool_credentials_used: false\n'
		printf 'wifi_credentials_used_for_network_join_only: true\n'
		if [[ -n "$comparator_report" ]]; then
			printf 'comparator_report: %s\n' "$(basename "$comparator_report")"
		fi
	} >"$evidence_path"
}

recover_rust_after_failure() {
	local recovery_dir="$capture_dir/rust-recovery-after-upstream-failure"
	bash scripts/phase28.1.1-wire-parity-capture.sh \
		"$port" \
		"$recovery_dir" \
		"$duration_seconds" \
		--wifi-credentials "$wifi_credentials" \
		--pool-credentials "$1" \
		>"$capture_dir/rust-recovery.stdout.log" 2>"$capture_dir/rust-recovery.stderr.log" || true
}

if [[ "$dry_run" == "true" ]]; then
	printf 'dry_run_status=passed\n'
	printf 'capture_dir_ignored=true\n'
	printf 'credential_values_printed=false\n'
	exit 0
fi

fake_pool_host="$(detect_fake_pool_host)" ||
	die "could not determine a host address reachable by the board; set PHASE28_FAKE_POOL_HOST"

just detect-ultra205 >"$capture_dir/detect-ultra205.raw.log" 2>&1
detected_port="$(sed -n 's/^port=//p' "$capture_dir/detect-ultra205.raw.log" | tail -1)"
[[ "$detected_port" == "$port" ]] ||
	die "detected Ultra 205 port did not match requested port"

upstream_fake_dir="$capture_dir/upstream-fake-pool"
start_fake_pool upstream "$upstream_fake_dir"
upstream_ready_json="$last_fake_pool_ready_json"
upstream_fake_pid="$last_fake_pool_pid"
upstream_pool_credentials="$upstream_fake_dir/synthetic-pool-credentials.json"
node scripts/phase28.1.1.1-synthetic-pool-credentials.mjs \
	--ready-json "$upstream_ready_json" \
	--host "$fake_pool_host" \
	--out "$upstream_pool_credentials"

upstream_capture_dir="$capture_dir/upstream-capture"
if ! bash scripts/phase28.1.1.1-upstream-golden-capture.sh \
	--port "$port" \
	--wifi-credentials "$wifi_credentials" \
	--pool-credentials "$upstream_pool_credentials" \
	--duration-seconds "$duration_seconds" \
	--capture-dir "$upstream_capture_dir" \
	>"$capture_dir/upstream-helper.stdout.log" 2>"$capture_dir/upstream-helper.stderr.log"; then
	stop_fake_pool_pid "$upstream_fake_pid"
	recover_rust_after_failure "$upstream_pool_credentials"
	write_evidence "blocked_safe_prerequisite" "upstream_aligned_capture_failed"
	die "upstream source-work-aligned capture failed"
fi
stop_fake_pool_pid "$upstream_fake_pid"
upstream_report_json="$upstream_fake_dir/report.json"
assert_fake_pool_report_ready "$upstream_report_json" upstream

rust_fake_dir="$capture_dir/rust-fake-pool"
start_fake_pool rust "$rust_fake_dir"
rust_ready_json="$last_fake_pool_ready_json"
rust_fake_pid="$last_fake_pool_pid"
rust_pool_credentials="$rust_fake_dir/synthetic-pool-credentials.json"
node scripts/phase28.1.1.1-synthetic-pool-credentials.mjs \
	--ready-json "$rust_ready_json" \
	--host "$fake_pool_host" \
	--out "$rust_pool_credentials"

rust_capture_dir="$capture_dir/rust-capture"
rust_capture_args=(
	"$port"
	"$rust_capture_dir"
	"$duration_seconds"
	--wifi-credentials "$wifi_credentials"
	--pool-credentials "$rust_pool_credentials"
)
if [[ -n "$investigation_mode" ]]; then
	rust_capture_args+=(--investigation "$investigation_mode")
fi
if [[ -n "$chip_detect_investigation_mode" ]]; then
	rust_capture_args+=(--chip-detect-investigation "$chip_detect_investigation_mode")
fi
if ! bash scripts/phase28.1.1-wire-parity-capture.sh \
	"${rust_capture_args[@]}" \
	>"$capture_dir/rust-helper.stdout.log" 2>"$capture_dir/rust-helper.stderr.log"; then
	stop_fake_pool_pid "$rust_fake_pid"
	write_evidence "blocked_safe_prerequisite" "rust_aligned_capture_failed"
	die "Rust source-work-aligned capture failed"
fi
stop_fake_pool_pid "$rust_fake_pid"
rust_report_json="$rust_fake_dir/report.json"
assert_fake_pool_report_ready "$rust_report_json" rust

comparator_report="$capture_dir/source-work-comparator-redacted.md"
node scripts/phase28.1.1.1-wire-field-compare.mjs \
	--upstream "$upstream_capture_dir/upstream-monitor.raw.log" \
	--rust "$rust_capture_dir/flash-monitor.log" \
	--upstream-source-work "$upstream_report_json" \
	--rust-source-work "$rust_report_json" \
	--out "$comparator_report"

write_evidence "captured" "none" "$comparator_report"
printf 'source_work_aligned_capture_status=complete\n'
printf 'comparator_report=%s\n' "$comparator_report"
