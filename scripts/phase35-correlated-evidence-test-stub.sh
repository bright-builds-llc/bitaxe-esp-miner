#!/usr/bin/env bash
# Multi-call fake tool dispatcher for Phase 35 process-boundary tests.

case "${0##*/}" in
flash)
	subcommand="${1:-}"
	all_args="$*"
	case "$subcommand" in
	flash-monitor)
		printf 'CALL\n' >>"${PHASE35_DIRECT_FLASH_CALLS:?}"
		printf 'arg=%s\n' "$@" >>"${PHASE35_DIRECT_FLASH_CALLS:?}"
		printf 'direct_flash\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
		;;
	finalize-evidence)
		printf 'CALL\n' >>"${PHASE35_FINALIZER_CALLS:?}"
		printf 'arg=%s\n' "$@" >>"${PHASE35_FINALIZER_CALLS:?}"
		printf 'finalize_evidence\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
		;;
	phase35-probe)
		printf 'CALL\n' >>"${PHASE35_DIRECT_FLASH_CALLS:?}"
		printf 'arg=%s\n' "$@" >>"${PHASE35_DIRECT_FLASH_CALLS:?}"
		printf 'flash_probe\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
		;;
	*) exit 98 ;;
	esac
	evidence_dir=""
	stage_root=""
	expected_private_sha256=""
	while (($#)); do
		if [[ "$1" == "--evidence-dir" ]]; then
			evidence_dir="$2"
			shift 2
			continue
		fi
		if [[ "$1" == "--expected-private-sha256" ]]; then
			expected_private_sha256="$2"
			shift 2
			continue
		fi
		if [[ "$1" == "--stage-root" ]]; then
			stage_root="$2"
			shift 2
			continue
		fi
		shift
	done
	if [[ "$subcommand" == phase35-probe ]]; then
		[[ -n "$stage_root" ]]
		[[ " $all_args " != *" wifi-credentials "* ]]
		[[ " $all_args " == *" --board 205 "* ]]
		[[ " $all_args " == *" --timeout-seconds 30 "* ]]
		mkdir -p "$stage_root"
		chmod 700 "$stage_root"
		if [[ "${PHASE35_TEST_PROBE_BOUNDARY_SCENARIO:-ready}" == pre_connect ]]; then
			printf 'Connecting...\n' >"${stage_root}/probe.private.log"
			jq -cn '{schema_version:"phase35-flash-boundary-v1",stage:"probe",tool_version_valid:true,launched:true,connected:false,device_info_complete:false,transfer_started:false,completed:false,duration_millis:11}' \
				>"${stage_root}/probe.metrics.json"
			chmod 600 "${stage_root}/probe.private.log" "${stage_root}/probe.metrics.json"
			exit 9
		fi
		printf 'Connecting...\n0x0123456789abcdef0123456789abcdef\n' \
			>"${stage_root}/probe.private.log"
		jq -cn '{schema_version:"phase35-flash-boundary-v1",stage:"probe",tool_version_valid:true,launched:true,connected:true,device_info_complete:true,transfer_started:true,completed:true,duration_millis:19}' \
			>"${stage_root}/probe.metrics.json"
		chmod 600 "${stage_root}/probe.private.log" "${stage_root}/probe.metrics.json"
		exit 0
	fi
	[[ -n "$evidence_dir" ]]
	if [[ "$subcommand" == flash-monitor ]]; then
		mkdir -p "$evidence_dir"
		chmod 700 "$evidence_dir"
		if [[ "${PHASE35_TEST_FLASH_BOUNDARY_SCENARIO:-ready}" == post_info_pre_transfer ]]; then
			mkdir -p "${evidence_dir}/private-stages"
			chmod 700 "${evidence_dir}/private-stages"
			printf '%s\n' \
				'Connecting...' \
				'Connected to device' \
				'Chip type: esp32s3' \
				'Flash size: 16 MB' \
				'Error: target connection failed before transfer' \
				>"${evidence_dir}/private-stages/factory.private.log"
			jq -cn \
				'{schema_version:"phase35-flash-boundary-v1",stage:"factory",tool_version_valid:true,launched:true,connected:true,device_info_complete:true,transfer_started:false,completed:false,duration_millis:37}' \
				>"${evidence_dir}/private-stages/factory.metrics.json"
			chmod 600 \
				"${evidence_dir}/private-stages/factory.private.log" \
				"${evidence_dir}/private-stages/factory.metrics.json"
			exit 9
		fi
		mkdir -p "${evidence_dir}/private-stages"
		chmod 700 "${evidence_dir}/private-stages"
		for stage in factory nvs monitor; do
			printf 'Writing finished\n' >"${evidence_dir}/private-stages/${stage}.private.log"
			jq -cn --arg stage "$stage" \
				'{schema_version:"phase35-flash-boundary-v1",stage:$stage,tool_version_valid:true,launched:true,connected:true,device_info_complete:true,transfer_started:true,completed:true,duration_millis:25}' \
				>"${evidence_dir}/private-stages/${stage}.metrics.json"
			chmod 600 \
				"${evidence_dir}/private-stages/${stage}.private.log" \
				"${evidence_dir}/private-stages/${stage}.metrics.json"
		done
		if [[ "${PHASE35_TEST_PRIVATE_INPUT:?}" == valid ]]; then
			printf 'device_%s=%s%s password=[redacted]\n' url http '://fixture-target' \
				>"${evidence_dir}/flash-monitor.classifier-input.log"
		else
			printf 'fixture-monitor-without-origin\n' \
				>"${evidence_dir}/flash-monitor.classifier-input.log"
		fi
		private_digest="$(shasum -a 256 \
			"${evidence_dir}/flash-monitor.classifier-input.log" | awk '{print $1}')"
		jq -cn \
			--arg private_monitor_log_sha256 "$private_digest" \
			'{redaction_mode:"dual",commit_ready:false,private_monitor_log_sha256:$private_monitor_log_sha256}' \
			>"${evidence_dir}/flash-command-evidence.private.json"
		chmod 600 \
			"${evidence_dir}/flash-monitor.classifier-input.log" \
			"${evidence_dir}/flash-command-evidence.private.json"
		exit 0
	fi
	[[ -n "$expected_private_sha256" ]]
	[[ ! -e "${evidence_dir}/flash-monitor.log" ]]
	[[ ! -e "${evidence_dir}/flash-command-evidence.json" ]]
	actual_private_sha256="$(shasum -a 256 \
		"${evidence_dir}/flash-monitor.classifier-input.log" | awk '{print $1}')"
	[[ "$actual_private_sha256" == "$expected_private_sha256" ]]
	printf 'device_url=[redacted-url] password=[redacted]\n' \
		>"${evidence_dir}/flash-monitor.log"
	jq -cn \
		--arg monitor_log_sha256 "$(shasum -a 256 \
			"${evidence_dir}/flash-monitor.log" | awk '{print $1}')" \
		'{redaction_mode:"dual",commit_ready:true,monitor_log_path:"flash-monitor.log",monitor_log_sha256:$monitor_log_sha256}' \
		>"${evidence_dir}/flash-command-evidence.json"
	chmod 600 \
		"${evidence_dir}/flash-monitor.log" \
		"${evidence_dir}/flash-command-evidence.json"
	;;
device-session)
	[[ "${1:-}" == reboot ]]
	printf 'CALL\n' >>"${PHASE35_DEVICE_SESSION_CALLS:?}"
	printf 'arg=%s\n' "$@" >>"${PHASE35_DEVICE_SESSION_CALLS:?}"
	printf 'device_session_reboot\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
	shift
	private_root=""
	request_input=""
	projection_output=""
	timeout_seconds=""
	while (($#)); do
		case "$1" in
		--private-root)
			private_root="$2"
			shift 2
			;;
		--request-input)
			request_input="$2"
			shift 2
			;;
		--projection-output)
			projection_output="$2"
			shift 2
			;;
		--timeout-seconds)
			timeout_seconds="$2"
			shift 2
			;;
		*) exit 98 ;;
		esac
	done
	[[ -d "$private_root" && ! -L "$private_root" ]]
	[[ -f "$request_input" && ! -L "$request_input" ]]
	[[ -n "$projection_output" && ! -e "$projection_output" ]]
	[[ "$timeout_seconds" == 360 ]]
	jq -e '
			.schema_version == "esp-device-session-reboot-request-v1" and
			.board_category == "205" and
			(.admitted_port | type == "string" and length > 0) and
			(.physical_identity_digest | test("^[0-9a-f]{64}$")) and
			(.trusted_origin | type == "string" and length > 0) and
			(.baseline.boot_session | type == "string" and length > 0) and
			(.baseline.boot_ordinal | type == "number") and
			(.expected_postcondition.hostname_sha256 | test("^[0-9a-f]{64}$"))
		' "$request_input" >/dev/null
	boot_ordinal="$(jq -er '.baseline.boot_ordinal + 1' "$request_input")"
	printf '{"event":"pre_reboot_application_bytes","count":24}\n' \
		>"${private_root}/events.private.jsonl"
	: >"${private_root}/serial.private.bin"
	printf '{"request_sent":true,"response_received":false}\n' \
		>"${private_root}/http.private.jsonl"
	jq -cn \
		--arg trusted_origin "$(jq -er '.trusted_origin' "$request_input")" \
		--arg source_commit "$(jq -er '.baseline.source_commit' "$request_input")" \
		--arg reference_commit "$(jq -er '.baseline.reference_commit' "$request_input")" \
		--arg app_elf_sha256 "$(jq -er '.baseline.app_elf_sha256' "$request_input")" \
		--arg hostname_sha256 "$(jq -er '.expected_postcondition.hostname_sha256' "$request_input")" \
		--argjson boot_ordinal "$boot_ordinal" \
		'{schema_version:"esp-device-session-private-result-v1",terminal_category:"ready",request_outcome:"response_missing",maybe_secondary_cleanup_failure:false,boot_b:{boot_session:"cccccccccccccccccccccccccccccccc",boot_ordinal:$boot_ordinal,reset_reason_category:"software_cpu",trusted_origin:$trusted_origin,source_commit:$source_commit,reference_commit:$reference_commit,app_elf_sha256:$app_elf_sha256,hostname_sha256:$hostname_sha256}}' \
		>"${private_root}/result.private.json"
	jq -cn \
		'{schema_version:"esp-device-session-v1",terminal_category:"ready",platform_category:"macos",board_category:"205",same_physical_device:true,stable_enumeration:true,reenumerated:false,reader_armed:true,pre_restart_serial_delivery:true,post_restart_serial_delivery:false,serial_delivery:"silent",request_outcome:"response_missing",request_attempt_count:1,service_loss_observed:false,trusted_origin_preserved:true,application_recovered:true,build_identity_matches:true,boot_session_changed:true,boot_ordinal_advanced_by_one:true,software_reset_observed:true,postcondition_matches:true,cleanup_complete:true,usb_disappearance_count:0,enumeration_change_count:0,serial_byte_count:64,http_observation_count:2,duration_millis:125}' \
		>"$projection_output"
	chmod 600 \
		"${private_root}/events.private.jsonl" \
		"${private_root}/serial.private.bin" \
		"${private_root}/http.private.jsonl" \
		"${private_root}/result.private.json" \
		"$projection_output"
	;;
report)
	report_subcommand="${1:-}"
	printf 'CALL\n' >>"${PHASE35_CLASSIFIER_CALLS:?}"
	printf 'arg=%s\n' "$@" >>"${PHASE35_CLASSIFIER_CALLS:?}"
	trace=""
	mode=""
	metrics_input=""
	private_log_input=""
	projection_output=""
	while (($#)); do
		if [[ "$1" == "--trace" ]]; then
			trace="$2"
			shift 2
			continue
		fi
		if [[ "$1" == "--mode" ]]; then
			mode="$2"
			shift 2
			continue
		fi
		if [[ "$1" == "--metrics-input" ]]; then
			metrics_input="$2"
			shift 2
			continue
		fi
		if [[ "$1" == "--private-log-input" ]]; then
			private_log_input="$2"
			shift 2
			continue
		fi
		if [[ "$1" == "--projection-output" ]]; then
			projection_output="$2"
			shift 2
			continue
		fi
		shift
	done
	printf 'trace=%s\n' "$trace" >>"${PHASE35_CLASSIFIER_CALLS:?}"
	if [[ "$report_subcommand" == classify-phase35-flash ]]; then
		printf 'flash_classifier\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
		[[ -s "$metrics_input" && -s "$private_log_input" && -n "$projection_output" ]]
		stage="$(jq -r '.stage' "$metrics_input")"
		boundary=ready
		if [[ "$(jq -r '.tool_version_valid' "$metrics_input")" != true ]]; then
			boundary=version_mismatch
		elif [[ "$(jq -r '.launched' "$metrics_input")" != true ]]; then
			boundary=spawn_failure
		elif [[ "$(jq -r '.connected' "$metrics_input")" != true ]]; then
			boundary=pre_connect_failure
		elif [[ "$(jq -r '.device_info_complete' "$metrics_input")" != true ]]; then
			boundary=device_info_failure
		elif [[ "$(jq -r '.transfer_started' "$metrics_input")" != true ]]; then
			boundary=post_info_pre_transfer_failed
		elif [[ "$(jq -r '.completed' "$metrics_input")" != true ]]; then
			boundary=transfer_failure
		fi
		jq -cn --arg stage "$stage" --arg boundary "$boundary" \
			'{schema_version:"phase35-flash-boundary-v1",stage:$stage,tool_version_valid:true,launched:true,connected:true,device_info_complete:true,transfer_started:($boundary == "ready"),completed:($boundary == "ready"),duration_millis:25,terminal_boundary:$boundary}' \
			>"$projection_output"
		chmod 600 "$projection_output"
		[[ "$boundary" == ready ]]
		exit
	fi
	printf 'classifier\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
	if [[ "${trace##*/}" == flash-monitor.classifier-input.log ]]; then
		[[ ! -e "$(dirname "$trace")/flash-monitor.log" ]]
		[[ ! -e "$(dirname "$trace")/flash-command-evidence.json" ]]
	fi
	case "${PHASE35_TEST_PARITY_OUTCOME:?}" in
	passed)
		private_origin_pattern="$(printf 'device_%s=%s%s' url http '://fixture-target')"
		if [[ "$mode" == post-restart ]]; then
			jq -cn '{status:"passed",category:"none",session:"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",boot_ordinal:8,device_url:"fixture-target"}'
		elif [[ "${trace##*/}" == flash-monitor.classifier-input.log ]] &&
			rg -Fq "$private_origin_pattern" "$trace"; then
			jq -cn '{status:"passed",category:"none",session:"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",boot_ordinal:7,device_url:"fixture-target"}'
		else
			jq -cn '{status:"failed",category:"baseline_origin_missing",session:null,boot_ordinal:null,device_url:null}'
		fi
		;;
	rejected)
		jq -cn '{status:"failed",category:"baseline_multiple_sessions",session:null,boot_ordinal:null,device_url:null}'
		;;
	*)
		exit 98
		;;
	esac
	;;
just | bazel)
	printf '%s\n' "${0##*/}" >>"${PHASE35_NESTED_TOOL_CALLS:?}"
	exit 97
	;;
*)
	exit 98
	;;
esac
exit 0
