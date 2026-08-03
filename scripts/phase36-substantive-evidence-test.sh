#!/usr/bin/env bash
set -euo pipefail

readonly subject="$1"
readonly fake_report_source="$2"
readonly effect_adapter_source="$3"
readonly fake_flash_source="$4"
readonly test_root="$(cd "$(mktemp -d "${TMPDIR:-/tmp}/phase36-substantive-evidence-test.XXXXXX")" && pwd -P)"
trap 'rm -rf "$test_root"' EXIT

readonly workspace="$test_root/workspace"
readonly runfiles="$test_root/runfiles"
readonly private_parent="$workspace/scratch/private"
readonly handle="$private_parent/attempt-handle.json"
readonly candidate="$workspace/scratch/candidate.json"
readonly wifi_credentials="$workspace/scratch/wifi-credentials.json"
readonly package_dir="$workspace/bazel-bin/firmware/bitaxe"
readonly manifest="$package_dir/bitaxe-ultra205-package.json"
readonly firmware_elf="$package_dir/bitaxe-ultra205.elf"
readonly executable_image="$package_dir/bitaxe-ultra205.bin"
readonly factory_image="$package_dir/bitaxe-ultra205-factory.bin"

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
}

digest() {
	shasum -a 256 "$1" | awk '{print $1}'
}

# Arrange
mkdir -p "$workspace/reference/esp-miner" "$package_dir" \
	"$runfiles/_main/tools/parity" "$runfiles/_main/tools/flash" \
	"$runfiles/_main/scripts"
git -C "$workspace/reference/esp-miner" init -q
git -C "$workspace/reference/esp-miner" config user.email test@example.invalid
git -C "$workspace/reference/esp-miner" config user.name Test
printf 'reference\n' >"$workspace/reference/esp-miner/README"
git -C "$workspace/reference/esp-miner" add README
git -C "$workspace/reference/esp-miner" commit -qm reference

git -C "$workspace" init -q
git -C "$workspace" config user.email test@example.invalid
git -C "$workspace" config user.name Test
git -C "$workspace" config advice.addEmbeddedRepo false
printf 'scratch/\nbazel-bin/\n' >"$workspace/.gitignore"
printf 'workspace\n' >"$workspace/README"
git -C "$workspace" add .gitignore README reference/esp-miner
git -C "$workspace" commit -qm workspace

printf 'elf\n' >"$firmware_elf"
printf 'executable\n' >"$executable_image"
printf 'factory\n' >"$factory_image"
readonly source_commit="$(git -C "$workspace" rev-parse HEAD)"
readonly reference_commit="$(git -C "$workspace/reference/esp-miner" rev-parse HEAD)"
readonly firmware_elf_digest="$(digest "$firmware_elf")"
readonly executable_image_digest="$(digest "$executable_image")"
readonly factory_image_digest="$(digest "$factory_image")"
jq -cn \
	--arg source_commit "$source_commit" \
	--arg reference_commit "$reference_commit" \
	--arg firmware_elf_digest "$firmware_elf_digest" \
	--arg executable_image_digest "$executable_image_digest" \
	--arg factory_image_digest "$factory_image_digest" \
	'{schema_version:3,source_commit:$source_commit,reference_commit:$reference_commit,
	image_metadata:{board:205,device_model:"Ultra 205",asic:"BM1366",rust_target:"xtensa-esp32s3-espidf"},
	app_elf_sha256:$firmware_elf_digest,
	artifacts:[
	{kind:"firmware_elf",path:"bitaxe-ultra205.elf",sha256:$firmware_elf_digest},
	{kind:"firmware_ota_image",path:"bitaxe-ultra205.bin",sha256:$executable_image_digest},
	{kind:"factory_merged_image",path:"bitaxe-ultra205-factory.bin",sha256:$factory_image_digest}
	]}' >"$manifest"
mkdir -p "$(dirname "$wifi_credentials")"
printf '{}\n' >"$wifi_credentials"
chmod 600 "$wifi_credentials"

cp "$fake_report_source" "$runfiles/_main/tools/parity/report"
cp "$effect_adapter_source" "$runfiles/_main/scripts/phase36_hardware_effect"
cp "$fake_flash_source" "$runfiles/_main/tools/flash/flash"
chmod 700 "$runfiles/_main/tools/parity/report" \
	"$runfiles/_main/scripts/phase36_hardware_effect" \
	"$runfiles/_main/tools/flash/flash"
touch "$runfiles/_main/scripts/phase35_http_boundary_read" \
	"$runfiles/_main/scripts/phase17-websocket-capture.mjs"

# Act
readonly preflight_output="$(
	RUNFILES_DIR="$runfiles" BUILD_WORKSPACE_DIRECTORY="$workspace" \
		bash "$subject" \
		mode=preflight board=205 \
		private-parent="$private_parent" \
		attempt-handle-file="$handle" \
		candidate-output="$candidate" \
		capture-timeout-seconds=360 \
		package-manifest="$manifest"
)"
readonly hardware_output="$(
	RUNFILES_DIR="$runfiles" BUILD_WORKSPACE_DIRECTORY="$workspace" \
		bash "$subject" \
		mode=hardware board=205 \
		private-parent="$private_parent" \
		attempt-handle-file="$handle" \
		candidate-output="$candidate" \
		capture-timeout-seconds=360 \
		wifi-credentials="$wifi_credentials"
)"

# Assert
[[ "$preflight_output" == category=preflight_ready$'\n'capability_digest=* ]]
[[ "$hardware_output" == category=sealed_non_promotion ]]
jq -e \
	--arg manifest "$manifest" \
	--arg firmware_elf "$firmware_elf" \
	--arg executable_image "$executable_image" \
	--arg factory_image "$factory_image" \
	'.manifest_path == $manifest and
	.firmware_elf_path == $firmware_elf and
	.executable_image_path == $executable_image and
	.factory_image_path == $factory_image' "$handle" >/dev/null

readonly child_name="$(jq -er '.child_name' "$handle")"
readonly attempt_child="$private_parent/$child_name"
readonly flash_args="$attempt_child/fake-flash/flash.args"
awk -v expected="$manifest" '
	$0 == "--manifest" { count += 1; getline; if ($0 == expected) matched += 1 }
	END { exit !(count == 1 && matched == 1) }
' "$flash_args"
if rg -x -- '--image' "$flash_args" >/dev/null; then
	printf 'redundant image override crossed the substantive evidence boundary\n' >&2
	exit 1
fi
jq -e '
	.schema_version == "phase36-effect-result-v1" and
	.operation == "exact_package_flash" and
	.status == "completed" and
	.failure == null
' "$attempt_child/effect-result-exact-package-flash.json" >/dev/null
for private_file in \
	"$handle" "$handle.stdout" "$handle.stderr" "$candidate" \
	"$attempt_child/effect-result-exact-package-flash.json" "$attempt_child/seal.json"; do
	[[ "$(file_mode "$private_file")" == 600 ]] || {
		printf 'private output is not mode 0600: %s\n' "$private_file" >&2
		exit 1
	}
done

printf 'phase36-substantive-evidence-test: passed\n'
