#!/usr/bin/env bash
set -euo pipefail

readonly command_name="${1:-}"
shift || true

case "$command_name" in
phase36-evaluator-identity)
	[[ "$#" == 0 ]] || exit 2
	printf 'evaluator_identity_digest=%064d\n' 1
	;;
phase36-hardware-capture)
	board=""
	private_parent=""
	attempt_handle_file=""
	candidate_output=""
	capture_timeout_seconds=""
	wifi_credentials=""
	while (($# > 0)); do
		case "$1" in
		--board) board="$2" ;;
		--private-parent) private_parent="$2" ;;
		--attempt-handle-file) attempt_handle_file="$2" ;;
		--candidate-output) candidate_output="$2" ;;
		--capture-timeout-seconds) capture_timeout_seconds="$2" ;;
		--wifi-credentials) wifi_credentials="$2" ;;
		*) exit 2 ;;
		esac
		shift 2
	done
	[[ "$board" == 205 && -n "$private_parent" && -n "$attempt_handle_file" &&
		-n "$candidate_output" && "$capture_timeout_seconds" == 360 &&
		-n "$wifi_credentials" ]] || exit 2

	child_name="$(jq -er '.child_name' "$attempt_handle_file")"
	attempt_child="$private_parent/$child_name"
	mkdir "$attempt_child"
	chmod 700 "$attempt_child"

	field() {
		jq -er --arg field "$1" '.[$field]' "$attempt_handle_file"
	}

	"$RUNFILES_DIR/_main/scripts/phase36_hardware_effect" \
		operation=exact-package-flash \
		board=205 \
		port=/dev/null \
		attempt-child="$attempt_child" \
		package-identity-digest="$(field package_identity_digest)" \
		manifest-path="$(field manifest_path)" \
		manifest-digest="$(field manifest_digest)" \
		firmware-elf-path="$(field firmware_elf_path)" \
		firmware-elf-digest="$(field firmware_elf_digest)" \
		executable-image-path="$(field executable_image_path)" \
		executable-image-digest="$(field executable_image_digest)" \
		factory-image-path="$(field factory_image_path)" \
		factory-image-digest="$(field factory_image_digest)" \
		capture-timeout-seconds="$capture_timeout_seconds" \
		wall-clock-timeout-seconds=420 \
		wifi-credentials="$wifi_credentials" \
		trusted-origin=unavailable \
		result-path="$attempt_child/effect-result-exact-package-flash.json"

	jq -cn '{schema_version:"phase36-candidate-v1",test_fixture:true}' >"$candidate_output"
	chmod 600 "$candidate_output"
	jq -cn \
		'{schema_version:"phase36-attempt-seal-v1",status:"sealed_non_promotion"}' \
		>"$attempt_child/seal.json"
	chmod 600 "$attempt_child/seal.json"
	printf 'category=sealed_non_promotion\n'
	;;
*)
	exit 2
	;;
esac
