#!/usr/bin/env bash
# shellcheck disable=SC2034 # This sourced library exposes state to its callers.
# Shared, local-only serial-session tracing and readiness checks.

SERIAL_SESSION_TRACE_FILE=""
SERIAL_SESSION_TRACE_DIR=""
SERIAL_SESSION_TRACE_STATUS="uninitialized"
SERIAL_SESSION_READINESS_CATEGORY="unavailable"
SERIAL_SESSION_READY_IDENTITY=""
SERIAL_SESSION_READY_PHYSICAL_IDENTITY=""
SERIAL_SESSION_READY_ENUMERATION_IDENTITY=""

serial_session_monotonic_ms() {
	if [[ -n "${SERIAL_SESSION_MONOTONIC_MS_BIN:-}" ]]; then
		"$SERIAL_SESSION_MONOTONIC_MS_BIN"
		return
	fi
	perl -MTime::HiRes=clock_gettime,CLOCK_MONOTONIC -e 'printf "%.0f\n", clock_gettime(CLOCK_MONOTONIC) * 1000'
}

serial_session_utc_timestamp() {
	date -u '+%Y-%m-%dT%H:%M:%SZ'
}

serial_session_trace_init() {
	local label="$1"
	local root="${SERIAL_SESSION_TRACE_ROOT:-scratch/serial-session-traces}"
	local timestamp

	command -v jq >/dev/null 2>&1 || {
		printf 'serial-session trace requires jq\n' >&2
		return 1
	}
	[[ "$label" =~ ^[A-Za-z0-9._-]+$ ]] || {
		printf 'invalid serial-session trace label\n' >&2
		return 1
	}

	umask 077
	mkdir -p "$root"
	chmod 700 "$root"
	timestamp="$(date -u '+%Y%m%dT%H%M%SZ')"
	SERIAL_SESSION_TRACE_DIR="${root}/${timestamp}-${label}-$$"
	mkdir "$SERIAL_SESSION_TRACE_DIR"
	chmod 700 "$SERIAL_SESSION_TRACE_DIR"
	SERIAL_SESSION_TRACE_FILE="${SERIAL_SESSION_TRACE_DIR}/session.jsonl"
	: >"$SERIAL_SESSION_TRACE_FILE"
	chmod 600 "$SERIAL_SESSION_TRACE_FILE"
	SERIAL_SESSION_TRACE_STATUS="complete"
}

serial_session_trace_event() {
	local event="$1"
	local maybe_details="${2:-}"
	local monotonic_ms
	local utc_timestamp

	[[ -n "$SERIAL_SESSION_TRACE_FILE" ]] || return 0
	[[ -n "$maybe_details" ]] || maybe_details='{}'
	monotonic_ms="$(serial_session_monotonic_ms)" || {
		SERIAL_SESSION_TRACE_STATUS="incomplete"
		return 1
	}
	[[ "$monotonic_ms" =~ ^[0-9]+$ ]] || {
		SERIAL_SESSION_TRACE_STATUS="incomplete"
		return 1
	}
	utc_timestamp="$(serial_session_utc_timestamp)"
	if ! jq -cn \
		--arg event "$event" \
		--arg utc_timestamp "$utc_timestamp" \
		--argjson monotonic_ms "$monotonic_ms" \
		--argjson details "$maybe_details" \
		'{event:$event,utc_timestamp:$utc_timestamp,monotonic_ms:$monotonic_ms} + $details' \
		>>"$SERIAL_SESSION_TRACE_FILE"; then
		SERIAL_SESSION_TRACE_STATUS="incomplete"
		return 1
	fi
}

serial_session_hash_text() {
	shasum -a 256 | awk '{print $1}'
}

serial_session_node_identity() {
	local port="$1"

	if [[ -n "${SERIAL_SESSION_NODE_IDENTITY_BIN:-}" ]]; then
		"$SERIAL_SESSION_NODE_IDENTITY_BIN" "$port"
		return
	fi
	if [[ "$(uname -s)" == "Darwin" ]]; then
		stat -f '%d:%i:%p:%z' "$port"
	else
		stat -Lc '%d:%i:%a:%s' "$port"
	fi
}

serial_session_usb_physical_identity() {
	local port="$1"
	local tty_name="${port##*/}"
	local sysfs_root="${SERIAL_SESSION_SYSFS_ROOT:-/sys}"

	if [[ -n "${SERIAL_SESSION_USB_PHYSICAL_IDENTITY_BIN:-}" ]]; then
		"$SERIAL_SESSION_USB_PHYSICAL_IDENTITY_BIN" "$port"
		return
	fi
	if [[ -n "${SERIAL_SESSION_USB_IDENTITY_BIN:-}" ]]; then
		"$SERIAL_SESSION_USB_IDENTITY_BIN" "$port"
		return
	fi
	if [[ "$(uname -s)" == "Darwin" ]] && command -v ioreg >/dev/null 2>&1; then
		local identity_block
		local stable_identity_fields
		identity_block="$(ioreg -r -c IOSerialBSDClient -l -w 0 |
			awk -v port="$port" 'BEGIN { RS="\\n[+]?-o " } index($0, port) { print }')"
		[[ -n "$identity_block" ]] || return 1
		stable_identity_fields="$(printf '%s\n' "$identity_block" | awk '
			/"(USB Serial Number|idVendor|idProduct|locationID)" =/ { print }
		')"
		printf '%s\n' "$stable_identity_fields" | grep -q '"idVendor" =' || return 1
		printf '%s\n' "$stable_identity_fields" | grep -q '"idProduct" =' || return 1
		printf '%s\n' "$stable_identity_fields" | grep -Eq '"(USB Serial Number|locationID)" =' || return 1
		printf '%s\n' "$stable_identity_fields" | serial_session_hash_text
		return
	fi
	if [[ -e "${sysfs_root}/class/tty/${tty_name}/device" ]]; then
		local device_path
		device_path="$(readlink -f "${sysfs_root}/class/tty/${tty_name}/device")" || return 1
		while [[ "$device_path" != "/" ]]; do
			if [[ -f "${device_path}/idVendor" && -f "${device_path}/idProduct" ]]; then
				{
					printf 'topology=%s\n' "${device_path##*/}"
					printf 'vendor=' && sed -n '1p' "${device_path}/idVendor"
					printf 'product=' && sed -n '1p' "${device_path}/idProduct"
					if [[ -f "${device_path}/serial" ]]; then
						printf 'serial=' && sed -n '1p' "${device_path}/serial"
					fi
				} | serial_session_hash_text
				return
			fi
			device_path="${device_path%/*}"
		done
	fi
	return 1
}

serial_session_usb_enumeration_identity() {
	local port="$1"
	local tty_name="${port##*/}"
	local sysfs_root="${SERIAL_SESSION_SYSFS_ROOT:-/sys}"

	if [[ -n "${SERIAL_SESSION_USB_ENUMERATION_IDENTITY_BIN:-}" ]]; then
		"$SERIAL_SESSION_USB_ENUMERATION_IDENTITY_BIN" "$port"
		return
	fi
	if [[ -n "${SERIAL_SESSION_USB_IDENTITY_BIN:-}" ]]; then
		serial_session_node_identity "$port"
		return
	fi
	if [[ "$(uname -s)" == "Darwin" ]] && command -v ioreg >/dev/null 2>&1; then
		local identity_block
		local enumeration_fields
		identity_block="$(ioreg -r -c IOSerialBSDClient -l -w 0 |
			awk -v port="$port" 'BEGIN { RS="\\n[+]?-o " } index($0, port) { print }')"
		[[ -n "$identity_block" ]] || return 1
		enumeration_fields="$(printf '%s\n' "$identity_block" | awk '
			match($0, /id 0x[0-9a-fA-F]+/) { print substr($0, RSTART, RLENGTH) }
			/"(IOCalloutDevice|IODialinDevice|IOTTYDevice|IOTTYBaseName)" =/ { print }
		')"
		[[ -n "$enumeration_fields" ]] || return 1
		{
			serial_session_node_identity "$port"
			printf '%s\n' "$enumeration_fields"
		} | serial_session_hash_text
		return
	fi
	if [[ -e "${sysfs_root}/class/tty/${tty_name}/device" ]]; then
		{
			serial_session_node_identity "$port"
			readlink -f "${sysfs_root}/class/tty/${tty_name}/device"
			find "${sysfs_root}/class/tty/${tty_name}/device" -maxdepth 1 -name uevent -type f -exec sed -n '1,80p' {} \;
		} | serial_session_hash_text
		return
	fi
	serial_session_node_identity "$port"
}

# Compatibility name: USB identity now represents only the stable device.
serial_session_usb_identity() {
	serial_session_usb_physical_identity "$1"
}

serial_session_holder_pids() {
	local port="$1"
	local lsof_bin="${SERIAL_SESSION_LSOF_BIN:-}"
	local output
	local status

	if [[ -z "$lsof_bin" ]]; then
		if ! lsof_bin="$(command -v lsof)"; then
			return 70
		fi
	fi
	[[ -n "$lsof_bin" && -x "$lsof_bin" ]] || return 70

	set +e
	output="$($lsof_bin -t -- "$port" 2>/dev/null)"
	status=$?
	set -e
	case "$status" in
	0)
		printf '%s\n' "$output" | awk '/^[0-9]+$/ && !seen[$0]++'
		;;
	1)
		return 0
		;;
	*)
		return 70
		;;
	esac
}

serial_session_process_snapshot() {
	local pid="$1"
	local pgid="unavailable"
	local pgid_status=0
	local descendant_count=0
	local descendant_status=0
	local group_member_count=0
	local group_member_status=0

	if [[ "$pid" =~ ^[0-9]+$ ]]; then
		set +e
		pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')"
		pgid_status=$?
		set -e
		if ((pgid_status != 0)); then
			pgid="unavailable"
		fi
		[[ -n "$pgid" ]] || pgid="unavailable"
		set +e
		descendant_count="$(pgrep -P "$pid" 2>/dev/null | wc -l | tr -d ' ')"
		descendant_status=$?
		set -e
		if ((descendant_status != 0)) || [[ ! "$descendant_count" =~ ^[0-9]+$ ]]; then
			descendant_count=0
		fi
		if [[ "$pgid" =~ ^[0-9]+$ ]]; then
			set +e
			group_member_count="$(ps -axo pgid= 2>/dev/null | awk -v pgid="$pgid" '$1 == pgid { count++ } END { print count + 0 }')"
			group_member_status=$?
			set -e
			if ((group_member_status != 0)) || [[ ! "$group_member_count" =~ ^[0-9]+$ ]]; then
				group_member_count=0
			fi
		fi
	fi
	jq -cn \
		--arg pid "$pid" \
		--arg pgid "$pgid" \
		--argjson descendant_count "$descendant_count" \
		--argjson group_member_count "$group_member_count" \
		'{pid:$pid,pgid:$pgid,descendant_count:$descendant_count,group_member_count:$group_member_count}'
}

serial_session_active_owner_gate() {
	local port="$1"
	local expected_pgid="$2"
	local interval="${SERIAL_SESSION_ACTIVE_OWNER_INTERVAL_SECONDS:-0.25}"
	local attempts="${SERIAL_SESSION_ACTIVE_OWNER_ATTEMPTS:-12}"
	local attempt

	[[ "$expected_pgid" =~ ^[0-9]+$ ]] || return 1
	for ((attempt = 1; attempt <= attempts; attempt++)); do
		local maybe_holder_pids=""
		local holder_probe_status=0
		local holder_count=0
		local expected_owner_count=0
		local unexpected_owner_count=0
		local holder_pid
		local holder_pgid
		local holder_pgid_status=0

		set +e
		maybe_holder_pids="$(serial_session_holder_pids "$port")"
		holder_probe_status=$?
		set -e
		if [[ "$holder_probe_status" -ne 0 ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="ownership_probe_unavailable"
			return 1
		fi
		while IFS= read -r holder_pid; do
			[[ "$holder_pid" =~ ^[0-9]+$ ]] || continue
			holder_count=$((holder_count + 1))
			set +e
			holder_pgid="$(ps -o pgid= -p "$holder_pid" 2>/dev/null | tr -d ' ')"
			holder_pgid_status=$?
			set -e
			if ((holder_pgid_status != 0)); then
				holder_pgid="unavailable"
			fi
			if [[ "$holder_pgid" == "$expected_pgid" ]]; then
				expected_owner_count=$((expected_owner_count + 1))
			else
				unexpected_owner_count=$((unexpected_owner_count + 1))
			fi
		done <<<"$maybe_holder_pids"

		serial_session_trace_event "active_owner_snapshot" "$(jq -cn \
			--arg port "$port" \
			--arg expected_pgid "$expected_pgid" \
			--arg holder_pids "$maybe_holder_pids" \
			--argjson attempt "$attempt" \
			--argjson holder_count "$holder_count" \
			--argjson expected_owner_count "$expected_owner_count" \
			--argjson unexpected_owner_count "$unexpected_owner_count" \
			'{port:$port,expected_pgid:$expected_pgid,attempt:$attempt,holder_count:$holder_count,holder_pids:$holder_pids,expected_owner_count:$expected_owner_count,unexpected_owner_count:$unexpected_owner_count}')" || return 1

		if ((holder_count > 0 && unexpected_owner_count == 0 && expected_owner_count == holder_count)); then
			SERIAL_SESSION_READINESS_CATEGORY="active_owner_verified"
			return 0
		fi
		if ((unexpected_owner_count > 0)); then
			SERIAL_SESSION_READINESS_CATEGORY="unexpected_active_holder"
			return 1
		fi
		if ((attempt < attempts)); then
			sleep "$interval"
		fi
	done
	SERIAL_SESSION_READINESS_CATEGORY="active_holder_missing"
	return 1
}

serial_session_readiness_gate() {
	local phase="$1"
	local port="$2"
	local maybe_expected_identity="${3:-}"
	local interval="${SERIAL_SESSION_READINESS_INTERVAL_SECONDS:-0.25}"
	local maybe_first_identity=""
	local maybe_first_physical_identity=""
	local maybe_first_enumeration_identity=""
	local sample

	SERIAL_SESSION_READINESS_CATEGORY="ready"
	SERIAL_SESSION_READY_IDENTITY=""
	SERIAL_SESSION_READY_PHYSICAL_IDENTITY=""
	SERIAL_SESSION_READY_ENUMERATION_IDENTITY=""
	for sample in 1 2 3; do
		local present=false
		local accessible=false
		local maybe_node_identity=""
		local maybe_physical_identity=""
		local maybe_enumeration_identity=""
		local maybe_combined_identity=""
		local maybe_holder_pids=""
		local holder_count=0
		local holder_probe_status=0
		local node_identity_status=0
		local physical_identity_status=0
		local enumeration_identity_status=0
		local holder_probe_available=false

		if [[ -e "$port" ]]; then
			present=true
		fi
		if [[ -r "$port" && -w "$port" ]]; then
			accessible=true
		fi
		if [[ "$present" == true ]]; then
			set +e
			maybe_node_identity="$(serial_session_node_identity "$port" 2>/dev/null)"
			node_identity_status=$?
			maybe_physical_identity="$(serial_session_usb_physical_identity "$port" 2>/dev/null)"
			physical_identity_status=$?
			maybe_enumeration_identity="$(serial_session_usb_enumeration_identity "$port" 2>/dev/null)"
			enumeration_identity_status=$?
			set -e
			if [[ -n "$maybe_physical_identity" && -n "$maybe_enumeration_identity" ]]; then
				maybe_combined_identity="$(printf '%s\n%s\n' "$maybe_physical_identity" "$maybe_enumeration_identity" | serial_session_hash_text)"
			fi
		fi

		set +e
		maybe_holder_pids="$(serial_session_holder_pids "$port")"
		holder_probe_status=$?
		set -e
		if [[ "$holder_probe_status" -eq 0 && -n "$maybe_holder_pids" ]]; then
			holder_count="$(printf '%s\n' "$maybe_holder_pids" | awk 'NF { count++ } END { print count + 0 }')"
		fi
		if [[ "$holder_probe_status" -eq 0 ]]; then
			holder_probe_available=true
		fi

		serial_session_trace_event "readiness_snapshot" "$(jq -cn \
			--arg phase "$phase" \
			--arg port "$port" \
			--arg node_identity "$maybe_node_identity" \
			--arg physical_identity "$maybe_physical_identity" \
			--arg enumeration_identity "$maybe_enumeration_identity" \
			--arg combined_identity "$maybe_combined_identity" \
			--arg holder_pids "$maybe_holder_pids" \
			--argjson sample "$sample" \
			--argjson present "$present" \
			--argjson accessible "$accessible" \
			--argjson holder_count "$holder_count" \
			--argjson holder_probe_available "$holder_probe_available" \
			'{phase:$phase,sample:$sample,port:$port,present:$present,accessible:$accessible,node_identity:$node_identity,physical_identity:$physical_identity,enumeration_identity:$enumeration_identity,combined_identity:$combined_identity,holder_count:$holder_count,holder_pids:$holder_pids,holder_probe_available:$holder_probe_available}')" || return 1

		if [[ "$present" != true ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="missing_node"
		elif [[ "$accessible" != true ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="inaccessible_node"
		elif [[ "$holder_probe_status" -ne 0 ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="ownership_probe_unavailable"
		elif ((holder_count > 0)); then
			SERIAL_SESSION_READINESS_CATEGORY="holders_present"
		elif ((node_identity_status != 0 || physical_identity_status != 0 || enumeration_identity_status != 0)) || [[ -z "$maybe_combined_identity" ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="identity_unavailable"
		elif [[ -n "$maybe_first_physical_identity" && "$maybe_physical_identity" != "$maybe_first_physical_identity" ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="physical_identity_changed"
		elif [[ -n "$maybe_first_enumeration_identity" && "$maybe_enumeration_identity" != "$maybe_first_enumeration_identity" ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="enumeration_identity_changed"
		elif [[ -n "$maybe_expected_identity" && "$maybe_combined_identity" != "$maybe_expected_identity" ]]; then
			SERIAL_SESSION_READINESS_CATEGORY="identity_changed"
		fi

		if [[ "$SERIAL_SESSION_READINESS_CATEGORY" != "ready" ]]; then
			serial_session_trace_event "readiness_result" "$(jq -cn --arg phase "$phase" --arg category "$SERIAL_SESSION_READINESS_CATEGORY" '{phase:$phase,category:$category,ready:false}')" || return 1
			return 1
		fi
		[[ -n "$maybe_first_identity" ]] || maybe_first_identity="$maybe_combined_identity"
		[[ -n "$maybe_first_physical_identity" ]] || maybe_first_physical_identity="$maybe_physical_identity"
		[[ -n "$maybe_first_enumeration_identity" ]] || maybe_first_enumeration_identity="$maybe_enumeration_identity"
		if ((sample < 3)); then
			sleep "$interval"
		fi
	done

	SERIAL_SESSION_READY_IDENTITY="$maybe_first_identity"
	SERIAL_SESSION_READY_PHYSICAL_IDENTITY="$maybe_first_physical_identity"
	SERIAL_SESSION_READY_ENUMERATION_IDENTITY="$maybe_first_enumeration_identity"
	serial_session_trace_event "readiness_result" "$(jq -cn --arg phase "$phase" --arg category ready --arg identity "$SERIAL_SESSION_READY_IDENTITY" --arg physical_identity "$SERIAL_SESSION_READY_PHYSICAL_IDENTITY" --arg enumeration_identity "$SERIAL_SESSION_READY_ENUMERATION_IDENTITY" '{phase:$phase,category:$category,identity:$identity,physical_identity:$physical_identity,enumeration_identity:$enumeration_identity,ready:true}')"
}

serial_session_trace_digest() {
	[[ -f "$SERIAL_SESSION_TRACE_FILE" ]] || return 1
	shasum -a 256 "$SERIAL_SESSION_TRACE_FILE" | awk '{print $1}'
}
