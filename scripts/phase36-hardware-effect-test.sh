#!/usr/bin/env bash
set -euo pipefail

readonly subject="$1"
readonly fake_flash_source="$2"
readonly test_root="$(mktemp -d "${TMPDIR:-/tmp}/phase36-hardware-effect-test.XXXXXX")"
trap 'rm -rf "$test_root"' EXIT

readonly workspace="$test_root/workspace"
readonly runfiles="$test_root/runfiles"
readonly attempt_child="$test_root/attempt"
readonly manifest="$workspace/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
readonly factory_image="$test_root/execroot/bitaxe-ultra205-factory.bin"
readonly firmware_elf="$test_root/execroot/bitaxe-ultra205.elf"
readonly executable_image="$test_root/execroot/bitaxe-ultra205.bin"
readonly wifi_credentials="$test_root/wifi-credentials.json"
readonly result="$attempt_child/effect-result-exact-package-flash.json"
readonly passive_result="$attempt_child/effect-result-passive-serial-observation.json"
readonly digest="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

# Arrange
mkdir -p \
	"$(dirname "$manifest")" \
	"$(dirname "$factory_image")" \
	"$runfiles/_main/tools/flash" \
	"$runfiles/_main/tools/parity" \
	"$runfiles/_main/scripts" \
	"$attempt_child"
chmod 700 "$attempt_child"
printf '{}\n' >"$manifest"
printf 'factory\n' >"$factory_image"
printf 'elf\n' >"$firmware_elf"
printf 'executable\n' >"$executable_image"
printf '{}\n' >"$wifi_credentials"
chmod 600 "$wifi_credentials"

cp "$fake_flash_source" "$runfiles/_main/tools/flash/flash"
chmod 700 "$runfiles/_main/tools/flash/flash"
touch \
	"$runfiles/_main/tools/parity/report" \
	"$runfiles/_main/scripts/phase35_http_boundary_read" \
	"$runfiles/_main/scripts/phase17-websocket-capture.mjs"

# Act
RUNFILES_DIR="$runfiles" \
	BUILD_WORKSPACE_DIRECTORY="$workspace" \
	bash "$subject" \
	operation=exact-package-flash \
	board=205 \
	port=/dev/null \
	attempt-child="$attempt_child" \
	package-identity-digest="$digest" \
	manifest-path="$manifest" \
	manifest-digest="$digest" \
	firmware-elf-path="$firmware_elf" \
	firmware-elf-digest="$digest" \
	executable-image-path="$executable_image" \
	executable-image-digest="$digest" \
	factory-image-path="$factory_image" \
	factory-image-digest="$digest" \
	capture-timeout-seconds=360 \
	wall-clock-timeout-seconds=420 \
	wifi-credentials="$wifi_credentials" \
	trusted-origin=unavailable \
	result-path="$result"

# Assert
readonly flash_args="$attempt_child/fake-flash/flash.args"
[[ -f "$flash_args" ]] || {
	printf 'missing fake-flash argument capture\n' >&2
	exit 1
}
awk -v expected="$manifest" '
	$0 == "--manifest" { count += 1; getline; if ($0 == expected) matched += 1 }
	END { exit !(count == 1 && matched == 1) }
' "$flash_args" || {
	printf 'manifest selector was not forwarded exactly once\n' >&2
	exit 1
}
if rg -x -- '--image' "$flash_args" >/dev/null; then
	printf 'redundant image override crossed the Phase 36 boundary\n' >&2
	exit 1
fi
jq -e '
	.schema_version == "phase36-effect-result-v1" and
	.operation == "exact_package_flash" and
	.status == "completed" and
	.failure == null
' "$result" >/dev/null
[[ "$(stat -f '%Lp' "$result" 2>/dev/null || stat -c '%a' "$result")" == 600 ]] || {
	printf 'effect result is not mode 0600\n' >&2
	exit 1
}

# Act
RUNFILES_DIR="$runfiles" \
	BUILD_WORKSPACE_DIRECTORY="$workspace" \
	bash "$subject" \
	operation=passive-serial-observation \
	board=205 \
	port=/dev/null \
	attempt-child="$attempt_child" \
	package-identity-digest="$digest" \
	manifest-path="$manifest" \
	manifest-digest="$digest" \
	firmware-elf-path="$firmware_elf" \
	firmware-elf-digest="$digest" \
	executable-image-path="$executable_image" \
	executable-image-digest="$digest" \
	factory-image-path="$factory_image" \
	factory-image-digest="$digest" \
	capture-timeout-seconds=360 \
	wall-clock-timeout-seconds=420 \
	wifi-credentials=unavailable \
	trusted-origin=unavailable \
	result-path="$passive_result" \
	>/dev/null

# Assert
readonly classifier_log="$attempt_child/passive-serial-observation/monitor.classifier-input.log"
readonly diagnostic_log="$attempt_child/passive-serial-observation/monitor.stderr.log"
for private_file in "$classifier_log" "$diagnostic_log" "$passive_result"; do
	[[ -f "$private_file" && ! -L "$private_file" ]] || {
		printf 'passive monitor private artifact is missing or unsafe\n' >&2
		exit 1
	}
	[[ "$(stat -f '%Lp' "$private_file" 2>/dev/null || stat -c '%a' "$private_file")" == 600 ]] || {
		printf 'passive monitor private artifact is not mode 0600\n' >&2
		exit 1
	}
done
rg -x 'device_url=http://192\.0\.2\.1' "$classifier_log" >/dev/null || {
	printf 'passive monitor classifier input was not captured\n' >&2
	exit 1
}
if rg -x -- '--evidence-mode|--evidence-dir|--redact-evidence' "$diagnostic_log" >/dev/null; then
	printf 'unsupported monitor evidence flag was forwarded\n' >&2
	exit 1
fi
jq -e '
	.schema_version == "phase36-effect-result-v1" and
	.operation == "passive_serial_observation" and
	.status == "completed" and
	.failure == null
' "$passive_result" >/dev/null

printf 'phase36-hardware-effect-test: passed\n'
