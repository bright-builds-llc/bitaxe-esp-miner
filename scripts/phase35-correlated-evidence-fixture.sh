#!/usr/bin/env bash
set -euo pipefail

readonly fixture_state="${PHASE35_FIXTURE_STATE:?PHASE35_FIXTURE_STATE is required}"
readonly scenario="${PHASE35_FIXTURE_SCENARIO:-success}"
readonly calls="${fixture_state}/calls.log"
readonly current_setting="${fixture_state}/current-setting.txt"
readonly source_commit="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
readonly other_commit="cccccccccccccccccccccccccccccccccccccccc"
readonly identity_digest="dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
readonly other_identity_digest="eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"

mkdir -p "$fixture_state"

record() {
	printf '%s\n' "$1" >>"$calls"
}

sha256_text() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

hash_fields() {
	local domain="$1"
	shift
	{
		printf '%s' "$domain"
		local field
		for field in "$@"; do
			printf '\0%s' "$field"
		done
	} | shasum -a 256 | awk '{print $1}'
}

emit_snapshot() {
	local label="$1"
	local value
	value="$(<"$current_setting")"
	local ordinal=40
	local session="fixture-session-a"
	local reset_category="setup"
	local revision=10

	case "$label" in
	boot-a)
		revision=11
		;;
	boot-b)
		ordinal=41
		session="fixture-session-b"
		reset_category="software_cpu"
		revision=12
		;;
	esac

	[[ "$scenario" != "wrong_reset_category" || "$label" != "boot-b" ]] ||
		reset_category="unexpected"
	[[ "$scenario" != "boot_ordinal_mismatch" || "$label" != "boot-b" ]] ||
		ordinal=43
	if [[ "$scenario" == "pre_patch_mismatch" && "$label" == "boot-a-pre" ]] ||
		[[ "$scenario" == "boot_b_value_mismatch" && "$label" == "boot-b" ]]; then
		value="fixture-other-setting"
	fi

	jq -cn \
		--argjson boot_ordinal "$ordinal" \
		--arg boot_session "$session" \
		--argjson storage_revision "$revision" \
		--arg reset_category "$reset_category" \
		--arg setting_digest "$(sha256_text "$value")" \
		--arg system_info_document "fixture-system-document" \
		--arg websocket_document "fixture-websocket-document" \
		--arg retained_log_document "fixture-retained-document" \
		--argjson started_millis "$((ordinal * 100))" \
		--argjson ended_millis "$((ordinal * 100 + 20))" \
		'{boot_ordinal:$boot_ordinal,boot_session:$boot_session,storage_revision:$storage_revision,reset_category:$reset_category,setting_digest:$setting_digest,system_info_document:$system_info_document,websocket_document:$websocket_document,retained_log_document:$retained_log_document,started_millis:$started_millis,ended_millis:$ended_millis}'
}

mutate_evidence_for_scenario() {
	local evidence="$1"
	local temporary="${evidence}.fixture"
	case "$scenario" in
	package_capability_drift)
		jq '.boot_a.package_capability_digest = ("0" * 64)' "$evidence" >"$temporary"
		;;
	detector_capability_drift)
		jq '.boot_b.detector_capability_digest = ("0" * 64)' "$evidence" >"$temporary"
		;;
	root_contract_drift)
		jq '.boot_a.root_contract_digest = ("0" * 64)' "$evidence" >"$temporary"
		;;
	target_lock_drift)
		jq '.boot_b.target_lock_digest = ("0" * 64)' "$evidence" >"$temporary"
		;;
	broken_event_predecessor)
		jq '.events[3].predecessor_event_digest = ("0" * 64)' "$evidence" >"$temporary"
		;;
	*)
		return
		;;
	esac
	mv "$temporary" "$evidence"
}

validate_event_chain() {
	local evidence="$1"
	local predecessor
	predecessor="$(jq -er '.admission_facts.root_contract_digest' "$evidence")"
	local sequence=0
	local prior_millis=0
	while IFS= read -r event; do
		((sequence += 1))
		[[ "$(jq -r '.sequence' <<<"$event")" == "$sequence" ]] || return 1
		[[ "$(jq -r '.predecessor_event_digest' <<<"$event")" == "$predecessor" ]] ||
			return 1
		local event_millis
		event_millis="$(jq -er '.monotonic_millis' <<<"$event")" || return 1
		((event_millis > prior_millis)) || return 1
		prior_millis="$event_millis"
		predecessor="$(hash_fields phase35-event-v1 \
			"$sequence" \
			"$(jq -r '.category' <<<"$event")" \
			"$event_millis" \
			"$(jq -r '.payload_digest' <<<"$event")" \
			"$predecessor")"
	done < <(jq -c '.events[]' "$evidence")
	[[ "$sequence" == 9 ]]
}

validate_evidence() {
	local evidence="$1"
	[[ "$(jq -r '.schema_version' "$evidence")" == "phase35-evidence-v1" ]] || return 1
	[[ "$(jq -r '[.events[].category] | join(",")' "$evidence")" == "root_admitted,boot_a_observed,patch_responded,storage_confirmed,reboot_started,boot_b_observed,no_actuation_verified,restoration_confirmed,cleanup_confirmed" ]] || return 1
	[[ "$(jq -r '.exact_package.capability_digest' "$evidence")" == "$(jq -r '.boot_a.package_capability_digest' "$evidence")" ]] || return 1
	[[ "$(jq -r '.detector_run.capability_digest' "$evidence")" == "$(jq -r '.boot_b.detector_capability_digest' "$evidence")" ]] || return 1
	[[ "$(jq -r '.admission_facts.root_contract_digest' "$evidence")" == "$(jq -r '.boot_a.root_contract_digest' "$evidence")" ]] || return 1
	[[ "$(jq -r '.admission_facts.target_lock_digest' "$evidence")" == "$(jq -r '.boot_b.target_lock_digest' "$evidence")" ]] || return 1
	validate_event_chain "$evidence"
}

action="${1:?fixture action is required}"
shift

case "$action" in
package_admission)
	record package_admission
	case "$scenario" in
	package_drift | runtime_identity_drift) exit 1 ;;
	esac
	printf 'fixture_package_admitted=true\n'
	;;
reference_guard)
	record reference_guard
	[[ "$scenario" != "reference_drift" ]]
	;;
current_head)
	record current_head
	if [[ "$scenario" == "source_drift" ]]; then
		printf '%s\n' "$other_commit"
	else
		printf '%s\n' "$source_commit"
	fi
	;;
detector)
	record detector
	case "$scenario" in
	zero_candidates) exit 0 ;;
	multiple_candidates)
		printf 'port=fixture-a\nport=fixture-b\n'
		exit 0
		;;
	board_info_failure)
		printf 'failure_category=board_info_failed\n'
		exit 1
		;;
	wrong_board)
		printf 'board=204\nport=fixture-device\n'
		exit 0
		;;
	esac
	printf 'board=205\nport=fixture-device\n'
	;;
physical_identity)
	record physical_identity
	printf '%s\n' "$identity_digest"
	;;
physical_identity_after)
	record physical_identity_after
	if [[ "$scenario" == "same_board_identity_drift" ]]; then
		printf '%s\n' "$other_identity_digest"
	else
		printf '%s\n' "$identity_digest"
	fi
	;;
credential_path)
	record credential_path
	;;
flash_boot_a)
	record flash_boot_a
	jq -cn '{target_token:"fixture-target"}'
	;;
validate_target)
	record validate_target
	case "$scenario" in
	stale_origin | multiple_origins | malformed_origin) exit 1 ;;
	esac
	;;
read_setting)
	label="${1:?read label is required}"
	record "read_setting_${label}"
	if [[ "$scenario" == "immediate_storage_readback_mismatch" && "$label" == "immediate" ]]; then
		printf 'fixture-other-setting\n'
	else
		printf '%s\n' "$(<"$current_setting")"
	fi
	;;
capture_epoch)
	label="${1:?capture label is required}"
	record "capture_${label}"
	if [[ "$scenario" == "zero_byte_capture" && "$label" == "boot-a-pre" ]]; then
		exit 0
	fi
	emit_snapshot "$label"
	;;
mutated_setting)
	record mutated_setting
	printf 'fixture-setting-after\n'
	;;
patch)
	record patch
	[[ "$scenario" != "patch_not_committed" ]] || exit 1
	printf '%s\n' "${2:?new setting is required}" >"$current_setting"
	;;
reboot)
	record reboot
	[[ "$#" == 9 ]] || exit 1
	[[ "$2" == "--chip" && "$3" == "esp32s3" ]] || exit 1
	[[ "$4" == "--before" && "$5" == "no-reset-no-sync" ]] || exit 1
	[[ "$6" == "--after" && "$7" == "no-reset" ]] || exit 1
	[[ "$8" == "--no-reset" && "$9" == "--non-interactive" ]] || exit 1
	case "$scenario" in
	reboot_before_response_readback | missing_reboot) exit 1 ;;
	additional_reboot)
		record reboot_extra
		exit 1
		;;
	esac
	;;
restore)
	record restore
	[[ "$scenario" != "restoration_failure" ]] || exit 1
	printf '%s\n' "${2:?original setting is required}" >"$current_setting"
	;;
cleanup)
	record cleanup
	case "$scenario" in
	cleanup_failure | pid_leak | holder_leak) exit 1 ;;
	esac
	;;
recheck)
	check="${1:?recheck category is required}"
	record "recheck_${check}"
	case "${scenario}:${check}" in
	current_head_recheck_failure:current_head | reference_recheck_failure:reference | lifecycle_recheck_failure:lifecycle | runtime_identity_recheck_failure:runtime_identity | no_actuation_recheck_failure:no_actuation)
		exit 1
		;;
	esac
	;;
validator)
	record validator
	root="${1:?evidence root is required}"
	[[ "$scenario" != "validator_rejection" ]] || exit 1
	mutate_evidence_for_scenario "$root/eligible.json"
	validate_evidence "$root/eligible.json" || exit 1
	if [[ "$scenario" == "raw_field_redaction_failure" ]]; then
		jq -cn '{raw_capture:"fixture"}'
	else
		jq -cn --arg digest "$(shasum -a 256 "$root/eligible.json" | awk '{print $1}')" \
			'{status:"eligible",evidence_digest:$digest,event_count:9}'
	fi
	;;
*)
	printf 'unknown fixture action: %s\n' "$action" >&2
	exit 2
	;;
esac
