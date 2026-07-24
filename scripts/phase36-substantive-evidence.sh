#!/usr/bin/env bash
set -euo pipefail

readonly script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
readonly workspace_dir="${BUILD_WORKSPACE_DIRECTORY:-$(git -C "$script_dir/.." rev-parse --show-toplevel)}"

mode=""
board=""
private_parent=""
attempt_handle_file=""
candidate_output=""
capture_timeout_seconds=""
wifi_credentials=""
classification_output=""
package_manifest=""

fail() {
	printf 'category=%s\n' "$1" >&2
	exit 2
}

absolute_path() {
	local value="$1"
	if [[ "$value" == /* ]]; then
		printf '%s\n' "$value"
	else
		printf '%s/%s\n' "$workspace_dir" "$value"
	fi
}

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
}

assign_once() {
	local name="$1"
	local value="$2"
	local current="${!name}"
	[[ -z "$current" ]] || fail duplicate_argument
	printf -v "$name" '%s' "$value"
}

for argument in "$@"; do
	[[ "$argument" == *=* ]] || fail invalid_argument
	name="${argument%%=*}"
	value="${argument#*=}"
	[[ -n "$value" ]] || fail invalid_argument
	case "$name" in
	mode) assign_once mode "$value" ;;
	board) assign_once board "$value" ;;
	private-parent) assign_once private_parent "$value" ;;
	attempt-handle-file) assign_once attempt_handle_file "$value" ;;
	candidate-output) assign_once candidate_output "$value" ;;
	capture-timeout-seconds) assign_once capture_timeout_seconds "$value" ;;
	wifi-credentials) assign_once wifi_credentials "$value" ;;
	classification-output) assign_once classification_output "$value" ;;
	package-manifest) assign_once package_manifest "$value" ;;
	*) fail unknown_argument ;;
	esac
done

case "$mode" in
preflight | synthetic | hardware | inspect-candidate | classify-candidate) ;;
"") fail missing_mode ;;
*) fail invalid_mode ;;
esac
[[ "$board" == 205 ]] || fail wrong_board
[[ -n "$private_parent" && -n "$attempt_handle_file" && -n "$candidate_output" ]] ||
	fail missing_argument

private_parent="$(absolute_path "$private_parent")"
attempt_handle_file="$(absolute_path "$attempt_handle_file")"
candidate_output="$(absolute_path "$candidate_output")"
[[ "$private_parent" == "$workspace_dir/"* && "$private_parent" != *"/../"* ]] ||
	fail private_parent_invalid
[[ "$attempt_handle_file" == "$private_parent/"* && "$attempt_handle_file" != *"/../"* ]] ||
	fail attempt_handle_invalid
[[ "$candidate_output" == "$workspace_dir/"* && "$candidate_output" != *"/../"* ]] ||
	fail candidate_output_invalid

relative_parent="${private_parent#"$workspace_dir/"}"
git -C "$workspace_dir" check-ignore -q -- "$relative_parent" ||
	fail private_parent_not_ignored

resolve_report() {
	local candidate
	for candidate in \
		"${RUNFILES_DIR:-}/_main/tools/parity/report" \
		"$workspace_dir/bazel-bin/tools/parity/report"; do
		if [[ -x "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return
		fi
	done
	return 1
}

validate_parent() {
	[[ -d "$private_parent" && ! -L "$private_parent" ]] || fail private_parent_invalid
	[[ "$(file_mode "$private_parent")" == 700 ]] || fail private_parent_permissions_invalid
}

validate_handle() {
	validate_parent
	[[ -f "$attempt_handle_file" && ! -L "$attempt_handle_file" ]] ||
		fail attempt_handle_invalid
	[[ "$(file_mode "$attempt_handle_file")" == 600 ]] ||
		fail attempt_handle_permissions_invalid
}

handle_field() {
	jq -er --arg field "$1" '.[$field] | select(type == "string" and length > 0)' \
		"$attempt_handle_file" 2>/dev/null || fail attempt_handle_invalid
}

resolve_attempt_child() {
	local child_name
	child_name="$(handle_field child_name)"
	[[ "$child_name" =~ ^attempt-[0-9a-f]{16}$ ]] || fail attempt_handle_invalid
	printf '%s/%s\n' "$private_parent" "$child_name"
}

case "$mode" in
preflight)
	[[ -z "$wifi_credentials" && -z "$classification_output" ]] ||
		fail invalid_argument
	if [[ -n "$capture_timeout_seconds" ]]; then
		[[ "$capture_timeout_seconds" =~ ^[0-9]+$ ]] || fail capture_timeout_invalid
		((capture_timeout_seconds >= 360)) || fail capture_timeout_invalid
	fi
	umask 077
	if [[ ! -e "$private_parent" ]]; then
		mkdir -p "$private_parent" || fail private_parent_create_failed
		chmod 700 "$private_parent" || fail private_parent_permissions_invalid
	fi
	validate_parent
	[[ ! -e "$attempt_handle_file" && ! -e "$candidate_output" ]] ||
		fail preflight_output_exists
	if [[ -z "$package_manifest" ]]; then
		package_manifest="$workspace_dir/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
	else
		package_manifest="$(absolute_path "$package_manifest")"
	fi
	[[ "$package_manifest" == "$workspace_dir/"* && "$package_manifest" != *"/../"* &&
		-f "$package_manifest" && ! -L "$package_manifest" ]] ||
		fail package_manifest_invalid
	[[ -z "$(git -C "$workspace_dir" status --porcelain --untracked-files=all)" ]] ||
		fail source_tree_not_clean
	[[ -z "$(git -C "$workspace_dir/reference/esp-miner" status --porcelain --untracked-files=all)" ]] ||
		fail reference_tree_not_clean
	source_commit="$(git -C "$workspace_dir" rev-parse HEAD)"
	reference_commit="$(git -C "$workspace_dir/reference/esp-miner" rev-parse HEAD)"
	[[ "$(jq -er '.schema_version' "$package_manifest")" == 3 &&
		"$(jq -er '.source_commit' "$package_manifest")" == "$source_commit" &&
		"$(jq -er '.reference_commit' "$package_manifest")" == "$reference_commit" &&
		"$(jq -er '.image_metadata.board' "$package_manifest")" == 205 &&
		"$(jq -er '.image_metadata.device_model' "$package_manifest")" == "Ultra 205" &&
		"$(jq -er '.image_metadata.asic' "$package_manifest")" == BM1366 &&
		"$(jq -er '.image_metadata.rust_target' "$package_manifest")" == xtensa-esp32s3-espidf ]] ||
		fail package_identity_invalid
	resolve_artifact() {
		local kind="$1"
		local count path expected_digest resolved actual_digest
		count="$(jq -er --arg kind "$kind" '[.artifacts[] | select(.kind == $kind)] | length' "$package_manifest")"
		[[ "$count" == 1 ]] || fail package_identity_invalid
		path="$(jq -er --arg kind "$kind" '.artifacts[] | select(.kind == $kind) | .path' "$package_manifest")"
		expected_digest="$(jq -er --arg kind "$kind" '.artifacts[] | select(.kind == $kind) | .sha256' "$package_manifest")"
		[[ "$expected_digest" =~ ^[0-9a-f]{64}$ ]] || fail package_identity_invalid
		resolved="$(cd "$(dirname "$package_manifest")" && perl -MCwd=realpath -e 'print realpath($ARGV[0]) // q{}' "$path")"
		[[ -n "$resolved" && -f "$resolved" && ! -L "$resolved" ]] ||
			fail package_identity_invalid
		actual_digest="$(shasum -a 256 "$resolved" | awk '{print $1}')"
		[[ "$actual_digest" == "$expected_digest" ]] || fail package_identity_invalid
		printf '%s\t%s\n' "$resolved" "$actual_digest"
	}
	IFS=$'\t' read -r firmware_elf_path firmware_elf_digest < <(resolve_artifact firmware_elf)
	IFS=$'\t' read -r executable_image_path executable_image_digest < <(resolve_artifact firmware_ota_image)
	IFS=$'\t' read -r factory_image_path factory_image_digest < <(resolve_artifact factory_merged_image)
	[[ "$(jq -er '.app_elf_sha256' "$package_manifest")" == "$firmware_elf_digest" ]] ||
		fail package_identity_invalid
	manifest_digest="$(shasum -a 256 "$package_manifest" | awk '{print $1}')"
	package_identity_digest="$(printf '%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s' \
		phase36-package-identity-v1 "$source_commit" "$reference_commit" "$manifest_digest" \
		"$firmware_elf_digest" "$executable_image_digest" "$factory_image_digest" |
		shasum -a 256 | awk '{print $1}')"
	report="$(resolve_report)" || fail broker_unavailable
	evaluator_identity_digest="$("$report" phase36-evaluator-identity |
		awk -F= '$1 == "evaluator_identity_digest" {print $2}')"
	[[ "$evaluator_identity_digest" =~ ^[0-9a-f]{64}$ ]] ||
		fail evaluator_identity_invalid
	nonce="$(printf '%s\\0%s\\0%s' "$source_commit" "$reference_commit" "$$-$RANDOM" |
		shasum -a 256 | awk '{print $1}')"
	child_name="attempt-${nonce:0:16}"
	child_path="$private_parent/$child_name"
	[[ ! -e "$child_path" ]] || fail attempt_child_exists
	capability_digest="$(printf '%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s\\0%s' \
		phase36-preflight-v2 "$source_commit" "$reference_commit" "$evaluator_identity_digest" \
		"$manifest_digest" "$firmware_elf_digest" "$executable_image_digest" \
		"$factory_image_digest" "$package_identity_digest" xtensa-esp32s3-espidf 205 BM1366 "$nonce" |
		shasum -a 256 | awk '{print $1}')"
	jq -cn \
		--arg child_name "$child_name" \
		--arg capability_digest "$capability_digest" \
		--arg source_commit "$source_commit" \
		--arg reference_commit "$reference_commit" \
		--arg evaluator_identity_digest "$evaluator_identity_digest" \
		--arg manifest_path "$package_manifest" \
		--arg manifest_digest "$manifest_digest" \
		--arg firmware_elf_path "$firmware_elf_path" \
		--arg firmware_elf_digest "$firmware_elf_digest" \
		--arg executable_image_path "$executable_image_path" \
		--arg executable_image_digest "$executable_image_digest" \
		--arg factory_image_path "$factory_image_path" \
		--arg factory_image_digest "$factory_image_digest" \
		--arg package_identity_digest "$package_identity_digest" \
		'{schema_version:"phase36-attempt-handle-v2",child_name:$child_name,
		capability_digest:$capability_digest,source_commit:$source_commit,
		reference_commit:$reference_commit,evaluator_identity_digest:$evaluator_identity_digest,
		target:"xtensa-esp32s3-espidf",board:"205",asic:"BM1366",
		manifest_path:$manifest_path,manifest_digest:$manifest_digest,
		firmware_elf_path:$firmware_elf_path,firmware_elf_digest:$firmware_elf_digest,
		executable_image_path:$executable_image_path,executable_image_digest:$executable_image_digest,
		factory_image_path:$factory_image_path,factory_image_digest:$factory_image_digest,
		package_identity_digest:$package_identity_digest}' \
		>"$attempt_handle_file"
	chmod 600 "$attempt_handle_file"
	: >"${attempt_handle_file}.stdout"
	: >"${attempt_handle_file}.stderr"
	chmod 600 "${attempt_handle_file}.stdout" "${attempt_handle_file}.stderr"
	printf 'category=preflight_ready\ncapability_digest=%s\n' "$capability_digest"
	;;
synthetic)
	[[ -z "$wifi_credentials" && -z "$classification_output" && -z "$package_manifest" &&
		-z "$capture_timeout_seconds" ]] || fail invalid_argument
	validate_handle
	attempt_child="$(resolve_attempt_child)"
	[[ ! -e "$attempt_child" && ! -e "$candidate_output" ]] ||
		fail synthetic_output_exists
	umask 077
	mkdir "$attempt_child"
	chmod 700 "$attempt_child"
	report="$(resolve_report)" || fail broker_unavailable
	private_capture="$attempt_child/private-capture.json"
	capability_digest="$(handle_field capability_digest)"
	if ! "$report" phase36-synthetic-capture \
		--private-output "$private_capture" \
		--candidate-output "$candidate_output" \
		--capability-digest "$capability_digest" \
		>"${attempt_handle_file}.stdout" 2>"${attempt_handle_file}.stderr"; then
		fail synthetic_broker_failed
	fi
	chmod 600 "${attempt_handle_file}.stdout" "${attempt_handle_file}.stderr"
	private_digest="$(shasum -a 256 "$private_capture" | awk '{print $1}')"
	candidate_digest="$(jq -er '.candidate_digest' "$candidate_output")"
	jq -cn \
		--arg private_capture_digest "$private_digest" \
		--arg candidate_digest "$candidate_digest" \
		'{schema_version:"phase36-attempt-seal-v1",status:"sealed_eligible",private_capture_digest:$private_capture_digest,candidate_digest:$candidate_digest}' \
		>"$attempt_child/seal.json"
	chmod 600 "$attempt_child/seal.json"
	printf 'category=synthetic_complete\ncandidate_digest=%s\nprivate_capture_digest=%s\n' \
		"$candidate_digest" "$private_digest"
	;;
hardware)
	# Cross-plan contract: the delegated broker owns exactly one `just detect-ultra205`.
	[[ -z "$classification_output" && -z "$package_manifest" ]] || fail invalid_argument
	[[ "$capture_timeout_seconds" =~ ^[0-9]+$ ]] || fail capture_timeout_invalid
	((capture_timeout_seconds >= 360)) || fail capture_timeout_invalid
	[[ -n "$wifi_credentials" ]] || fail wifi_credentials_missing
	wifi_credentials="$(absolute_path "$wifi_credentials")"
	validate_handle
	attempt_child="$(resolve_attempt_child)"
	[[ ! -e "$attempt_child" && ! -e "$candidate_output" ]] ||
		fail hardware_output_exists
	report="$(resolve_report)" || fail broker_unavailable
	if ! "$report" phase36-hardware-capture \
		--board 205 \
		--private-parent "$private_parent" \
		--attempt-handle-file "$attempt_handle_file" \
		--candidate-output "$candidate_output" \
		--capture-timeout-seconds "$capture_timeout_seconds" \
		--wifi-credentials "$wifi_credentials" \
		>"${attempt_handle_file}.stdout" 2>"${attempt_handle_file}.stderr"; then
		broker_category="$(awk -F= '$1 == "category" && $2 ~ /^[a-z0-9_]+$/ {print $2; exit}' \
			"${attempt_handle_file}.stderr")"
		[[ -n "$broker_category" ]] || broker_category=phase36_hardware_broker_output_invalid
		fail "$broker_category"
	fi
	chmod 600 "${attempt_handle_file}.stdout" "${attempt_handle_file}.stderr"
	broker_category="$(awk -F= '$1 == "category" && $2 ~ /^sealed_(eligible|non_promotion)$/ {print $2; exit}' \
		"${attempt_handle_file}.stdout")"
	[[ -n "$broker_category" ]] || fail phase36_hardware_broker_output_invalid
	printf 'category=%s\n' "$broker_category"
	;;
inspect-candidate)
	[[ -z "$wifi_credentials" && -z "$classification_output" && -z "$package_manifest" &&
		-z "$capture_timeout_seconds" ]] || fail invalid_argument
	validate_handle
	attempt_child="$(resolve_attempt_child)"
	[[ -d "$attempt_child" && ! -L "$attempt_child" &&
		"$(file_mode "$attempt_child")" == 700 ]] || fail attempt_child_invalid
	[[ -f "$attempt_child/seal.json" && "$(file_mode "$attempt_child/seal.json")" == 600 ]] ||
		fail attempt_not_sealed
	report="$(resolve_report)" || fail classifier_unavailable
	exec "$report" inspect-phase36-candidate --candidate-input "$candidate_output"
	;;
classify-candidate)
	[[ -z "$wifi_credentials" && -z "$capture_timeout_seconds" && -z "$package_manifest" &&
		-n "$classification_output" ]] || fail invalid_argument
	classification_output="$(absolute_path "$classification_output")"
	[[ "$classification_output" == "$private_parent/"* &&
		"$classification_output" != *"/../"* ]] || fail classification_output_invalid
	[[ ! -e "$classification_output" ]] || fail classification_output_exists
	validate_handle
	attempt_child="$(resolve_attempt_child)"
	[[ -d "$attempt_child" && ! -L "$attempt_child" &&
		"$(file_mode "$attempt_child")" == 700 ]] || fail attempt_child_invalid
	[[ -f "$attempt_child/seal.json" && "$(file_mode "$attempt_child/seal.json")" == 600 ]] ||
		fail attempt_not_sealed
	report="$(resolve_report)" || fail classifier_unavailable
	exec "$report" classify-phase36-candidate \
		--private-input "$attempt_child/private-capture.json" \
		--candidate-input "$candidate_output" \
		--classification-output "$classification_output"
	;;
esac
