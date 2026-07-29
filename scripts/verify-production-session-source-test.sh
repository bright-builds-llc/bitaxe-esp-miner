#!/usr/bin/env bash
set -euo pipefail

readonly guard_script="$1"
readonly canonical_owner_source="$2"
readonly canonical_deterministic_adapter_source="$3"
temp_root="$(mktemp -d "${TMPDIR:-/tmp}/production-session-source-test.XXXXXX")"
readonly temp_root

cleanup() {
	rm -rf "$temp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_guard_fails_with() {
	local expected_message="$1"
	local owner_source="$2"
	local deterministic_adapter_source="$3"
	local output

	if output="$(
		"$guard_script" \
			--verify-adapter-contract \
			"$owner_source" \
			"$deterministic_adapter_source" 2>&1
	)"; then
		fail "mutated adapter contract unexpectedly passed"
	fi

	if [[ "$output" != *"$expected_message"* ]]; then
		printf 'Expected output to contain: %s\n' "$expected_message" >&2
		printf 'Actual output:\n%s\n' "$output" >&2
		exit 1
	fi
}

assert_engine_guard_fails_with() {
	local expected_message="$1"
	local source="$2"
	local output

	if output="$("$guard_script" --verify-deep-engine-contract "$source" 2>&1)"; then
		fail "mutated deep engine source unexpectedly passed"
	fi

	if [[ "$output" != *"$expected_message"* ]]; then
		printf 'Actual output:\n%s\n' "$output" >&2
		fail "deep engine boundary failure category was not preserved"
	fi
}

copy_canonical_sources() {
	local case_name="$1"
	local case_root="${temp_root}/${case_name}"

	mkdir -p "$case_root"
	cp "$canonical_owner_source" "${case_root}/owner.rs"
	cp "$canonical_deterministic_adapter_source" "${case_root}/deterministic.rs"
	printf '%s\n' "$case_root"
}

test_current_adapter_contract_passes() {
	# Arrange
	local case_root
	case_root="$(copy_canonical_sources current)"

	# Act
	local output
	output="$(
		"$guard_script" \
			--verify-adapter-contract \
			"${case_root}/owner.rs" \
			"${case_root}/deterministic.rs"
	)"

	# Assert
	[[ "$output" == "production_session_adapter_contract=passed" ]] ||
		fail "current adapter contract did not pass"
}

test_interpreter_seam_drift_fails() {
	# Arrange
	local case_root
	case_root="$(copy_canonical_sources interpreter-seam)"
	sed 's/adapter\.maybe_execute(effect, now_ms)/adapter.execute(effect, now_ms)/' \
		"${case_root}/owner.rs" >"${case_root}/mutated-owner.rs"

	# Act and Assert
	assert_guard_fails_with \
		"owner contract missing: adapter.maybe_execute(effect, now_ms)" \
		"${case_root}/mutated-owner.rs" \
		"${case_root}/deterministic.rs"
}

test_adapter_count_drift_fails() {
	# Arrange
	local case_root
	case_root="$(copy_canonical_sources adapter-count)"
	printf '\nstruct OrdinaryEspProductionSessionAdapter;\n' >>"${case_root}/owner.rs"

	# Act and Assert
	assert_guard_fails_with \
		"expected exactly two adapters, found 3" \
		"${case_root}/owner.rs" \
		"${case_root}/deterministic.rs"
}

test_raw_socket_drift_fails() {
	# Arrange
	local case_root
	case_root="$(copy_canonical_sources forbidden-io)"
	printf '\nuse std::net::TcpStream;\n' >>"${case_root}/owner.rs"

	# Act and Assert
	assert_guard_fails_with \
		"production owner contains raw socket I/O" \
		"${case_root}/owner.rs" \
		"${case_root}/deterministic.rs"
}

test_raw_secret_drift_fails() {
	# Arrange
	local case_root
	case_root="$(copy_canonical_sources forbidden-secret)"
	printf '\nstruct EspNvs { poolPassword: String }\n' >>"${case_root}/owner.rs"

	# Act and Assert
	assert_guard_fails_with \
		"production owner contains raw pool secrets or NVS access" \
		"${case_root}/owner.rs" \
		"${case_root}/deterministic.rs"
}

test_raw_device_primitive_drift_fails() {
	# Arrange
	local case_root
	case_root="$(copy_canonical_sources forbidden-device)"
	printf '\nstruct ProductionAsicExecutor;\n' >>"${case_root}/owner.rs"

	# Act and Assert
	assert_guard_fails_with \
		"production owner contains raw device primitives" \
		"${case_root}/owner.rs" \
		"${case_root}/deterministic.rs"
}

test_engine_clock_contract_passes_for_caller_supplied_time() {
	# Arrange
	local source="${temp_root}/caller-supplied-clock.rs"
	printf 'fn next_step(now_ms: u64) -> u64 { now_ms.saturating_sub(1) }\n' >"$source"

	# Act
	local output
	output="$("$guard_script" --verify-deep-engine-contract "$source")"

	# Assert
	[[ "$output" == "production_session_deep_engine_contract=passed" ]] ||
		fail "caller-supplied engine clock contract did not pass"
}

test_engine_clock_contract_rejects_real_clock_ownership() {
	# Arrange
	local source="${temp_root}/real-clock.rs"
	printf 'fn now() { let _now = std::time::Instant::now(); }\n' >"$source"

	# Act and Assert
	assert_engine_guard_fails_with "reusable engine owns a real clock" "$source"
}

test_engine_contract_rejects_raw_transport_or_secrets() {
	# Arrange
	local source="${temp_root}/raw-transport.rs"
	printf 'use std::net::TcpStream;\nstruct EspNvs;\n' >"$source"

	# Act and Assert
	assert_engine_guard_fails_with "reusable engine owns raw transport or secret I/O" "$source"
}

test_engine_contract_rejects_device_primitives() {
	# Arrange
	local source="${temp_root}/device-primitive.rs"
	printf 'struct ProductionAsicExecutor;\n' >"$source"

	# Act and Assert
	assert_engine_guard_fails_with "reusable engine owns device primitives" "$source"
}

test_current_adapter_contract_passes
test_interpreter_seam_drift_fails
test_adapter_count_drift_fails
test_raw_socket_drift_fails
test_raw_secret_drift_fails
test_raw_device_primitive_drift_fails
test_engine_clock_contract_passes_for_caller_supplied_time
test_engine_clock_contract_rejects_real_clock_ownership
test_engine_contract_rejects_raw_transport_or_secrets
test_engine_contract_rejects_device_primitives

printf 'production_session_source_guard_tests=passed\n'
