#!/usr/bin/env bash
# Target admission, identity, and redacted lock helpers for Phase 19.

redacted_origin() {
	local url="$1"

	case "$url" in
	http://*) printf 'http://[redacted]' ;;
	https://*) printf 'https://[redacted]' ;;
	"") printf 'not provided' ;;
	*) printf '[invalid-url]' ;;
	esac
}

validate_origin_device_url() {
	local value="$1"
	local rest
	local host

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
	if [[ "$rest" == *[[:space:]\"\'\<\>]* ]]; then
		return 1
	fi
	if [[ "$rest" == */* ]]; then
		if [[ "${rest#*/}" != "" ]]; then
			return 1
		fi
		host="${rest%/}"
	else
		host="$rest"
	fi

	[[ -n "$host" ]]
}

manifest_field() {
	local field="$1"

	if [[ ! -f "$manifest" ]] || ! command -v python3 >/dev/null 2>&1; then
		printf 'unavailable'
		return
	fi
	python3 "$phase19_json_helper" manifest-field "$manifest" "$field"
}

load_device_url_from_flash_evidence() {
	if [[ -z "$flash_evidence_json" || ! -f "$flash_evidence_json" ]]; then
		printf 'flash evidence JSON is missing: %s\n' "$flash_evidence_json" >&2
		return 1
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		printf 'python3 is required to parse flash evidence JSON\n' >&2
		return 1
	fi

	local extracted
	if ! extracted="$(python3 "$phase19_json_helper" extract-flash "$flash_evidence_json")"; then
		printf '%s\n' "${extracted#error=}" >&2
		return 1
	fi

	while IFS='=' read -r key value; do
		case "$key" in
		device_url) device_url="$value" ;;
		selected_port) selected_port_from_flash="$value" ;;
		esac
	done <<<"$extracted"

	if ! validate_origin_device_url "$device_url"; then
		printf 'flash evidence device_url is not origin-only\n' >&2
		device_url=""
		return 1
	fi

	device_url_source="usb_flash_monitor_log"
	return 0
}

write_target_lock() {
	local target_status="$1"
	local selected_port="$2"
	local source_commit
	local reference_commit

	source_commit="$(manifest_field source_commit)"
	reference_commit="$(manifest_field reference_commit)"
	mkdir -p "$(dirname "$target_lock_out")"

	python3 "$phase19_json_helper" write-target-lock \
		"$target_lock_out" \
		"$target_status" \
		"$device_url_source" \
		"$(redacted_origin "$device_url")" \
		"$selected_port" \
		"$source_commit" \
		"$reference_commit" \
		"$manifest" \
		"$flash_evidence_json"
}
