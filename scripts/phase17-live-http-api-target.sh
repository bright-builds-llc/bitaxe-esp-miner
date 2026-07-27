#!/usr/bin/env bash
# Target admission, identity, and redacted lock helpers for Phase 17.

json_field() {
	local target_path="$1"
	local field="$2"

	if [[ ! -f "$target_path" ]] || ! command -v python3 >/dev/null 2>&1; then
		printf ''
		return
	fi
	python3 "$phase17_json_helper" field "$target_path" "$field"
}

device_urls_from_monitor_log() {
	local target_path="$1"
	python3 "$phase17_json_helper" monitor-urls "$target_path"
}

load_device_url_from_flash_evidence() {
	device_url_lookup_reason=""

	if [[ -z "$flash_evidence_json" || ! -f "$flash_evidence_json" ]]; then
		device_url_lookup_reason="missing --flash-evidence-json"
		return 1
	fi

	local command_kind
	local board
	local trusted_output
	command_kind="$(json_field "$flash_evidence_json" command_kind)"
	board="$(json_field "$flash_evidence_json" board)"
	trusted_output="$(json_field "$flash_evidence_json" trusted_output)"

	if [[ "$command_kind" != *"flash-monitor"* ]]; then
		device_url_lookup_reason="flash command_kind is not flash-monitor"
		return 1
	fi
	if [[ "$board" != "205" ]]; then
		device_url_lookup_reason="flash board is not 205"
		return 1
	fi
	if [[ "$trusted_output" != "true" ]]; then
		device_url_lookup_reason="flash trusted_output is not true"
		return 1
	fi

	local monitor_log
	monitor_log="$(json_field "$flash_evidence_json" monitor_log_path)"
	if [[ -z "$monitor_log" ]]; then
		monitor_log="$(json_field "$flash_evidence_json" log_path)"
	fi
	if [[ -z "$monitor_log" || ! -f "$monitor_log" ]]; then
		device_url_lookup_reason="monitor log path is missing or unreadable"
		return 1
	fi

	local urls
	urls="$(device_urls_from_monitor_log "$monitor_log" || true)"
	local url_count
	url_count="$(printf '%s\n' "$urls" | LC_ALL=C sed '/^$/d' | wc -l | tr -d ' ')"
	if [[ "$url_count" != "1" ]]; then
		device_url_lookup_reason="monitor log must contain exactly one device_url"
		return 1
	fi

	device_url="$(printf '%s\n' "$urls" | LC_ALL=C sed -n '1p')"
	if ! validate_origin_device_url "$device_url"; then
		device_url=""
		device_url_lookup_reason="monitor log device_url is not origin-only"
		return 1
	fi

	device_url_source="usb_flash_monitor_log"
	return 0
}

redacted_origin() {
	local url="$1"

	case "$url" in
	http://*) printf 'http://[redacted]' ;;
	https://*) printf 'https://[redacted]' ;;
	*) printf '[invalid-url]' ;;
	esac
}

redacted_target() {
	local url="$1"
	local scheme
	local rest
	local path="/"

	case "$url" in
	http://*)
		scheme="http"
		rest="${url#http://}"
		;;
	https://*)
		scheme="https"
		rest="${url#https://}"
		;;
	*)
		printf '[invalid-url]'
		return
		;;
	esac

	if [[ "$rest" == */* ]]; then
		path="/${rest#*/}"
	fi
	printf '%s://[redacted]%s' "$scheme" "$path"
}

validate_origin_device_url() {
	local value="$1"
	local rest

	case "$value" in
	http://*)
		rest="${value#http://}"
		;;
	https://*)
		rest="${value#https://}"
		;;
	*)
		return 1
		;;
	esac

	if [[ -z "$rest" || "$rest" == *"@"* || "$rest" == *"?"* || "$rest" == *"#"* ]]; then
		return 1
	fi
	if [[ "$rest" == */* && "$rest" != */ ]]; then
		return 1
	fi
	if [[ "$rest" == "/" ]]; then
		return 1
	fi
	return 0
}

write_target_lock() {
	local target_status="$1"
	local lock_selected_port="$2"

	mkdir -p "$(dirname "$target_lock_out")"
	python3 "$phase17_json_helper" write-target-lock \
		"$target_lock_out" \
		"$target_status" \
		"$device_url_source" \
		"$(redacted_origin "$device_url")" \
		"205" \
		"$lock_selected_port" \
		"$manifest_source_commit" \
		"$manifest_reference_commit" \
		"$manifest" \
		"$flash_evidence_json"
}

identity_preflight_passes() {
	if [[ ! -f "$manifest" ]]; then
		identity_block_reason="missing package manifest"
		return 1
	fi
	if [[ -z "$manifest_source_commit" || -z "$manifest_reference_commit" ]]; then
		identity_block_reason="manifest missing source_commit or reference_commit"
		return 1
	fi
	if [[ -z "$flash_evidence_json" || ! -f "$flash_evidence_json" ]]; then
		identity_block_reason="missing --flash-evidence-json"
		return 1
	fi

	local command_kind
	local board
	local trusted_output
	local firmware_commit
	local reference_commit
	local observed_firmware_commit
	local observed_reference_commit

	command_kind="$(json_field "$flash_evidence_json" command_kind)"
	board="$(json_field "$flash_evidence_json" board)"
	trusted_output="$(json_field "$flash_evidence_json" trusted_output)"
	firmware_commit="$(json_field "$flash_evidence_json" firmware_commit)"
	reference_commit="$(json_field "$flash_evidence_json" reference_commit)"
	observed_firmware_commit="$(json_field "$flash_evidence_json" observed_firmware_commit)"
	observed_reference_commit="$(json_field "$flash_evidence_json" observed_reference_commit)"

	if [[ "$command_kind" != *"flash-monitor"* ]]; then
		identity_block_reason="flash command_kind is not flash-monitor"
		return 1
	fi
	if [[ "$board" != "205" ]]; then
		identity_block_reason="flash board is not 205"
		return 1
	fi
	if [[ "$trusted_output" != "true" ]]; then
		identity_block_reason="flash trusted_output is not true"
		return 1
	fi
	if [[ "$firmware_commit" != "$manifest_source_commit" ]]; then
		identity_block_reason="flash firmware_commit does not match manifest source_commit"
		return 1
	fi
	if [[ "$reference_commit" != "$manifest_reference_commit" ]]; then
		identity_block_reason="flash reference_commit does not match manifest reference_commit"
		return 1
	fi
	if [[ "$observed_reference_commit" != "$manifest_reference_commit" ]]; then
		identity_block_reason="observed_reference_commit does not match manifest reference_commit"
		return 1
	fi
	if [[ "$observed_firmware_commit" != "$manifest_source_commit" ]]; then
		if [[ ${#observed_firmware_commit} -lt 12 || "$manifest_source_commit" != "$observed_firmware_commit"* ]]; then
			identity_block_reason="observed_firmware_commit is not the manifest source_commit or a 12+ character prefix"
			return 1
		fi
	fi

	return 0
}
