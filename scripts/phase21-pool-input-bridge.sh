#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --device-url ORIGIN --pool-credentials PATH --out-dir PATH [--curl-bin PATH] [--node-bin PATH] [--credentials-helper PATH] [--max-attempts N] [--poll-interval-seconds N]\n' "$(basename "$0")" >&2
}

device_url=""
pool_credentials=""
out_dir=""
curl_bin="${CURL_BIN:-curl}"
node_bin="${NODE_BIN:-node}"
credentials_helper="${PHASE21_POOL_CREDENTIALS_HELPER:-scripts/phase21-pool-credentials-json.mjs}"
max_attempts=5
poll_interval_seconds=1

while [[ $# -gt 0 ]]; do
	case "$1" in
	--device-url)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		device_url="$2"
		shift 2
		;;
	--pool-credentials)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		pool_credentials="$2"
		shift 2
		;;
	--out-dir)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		out_dir="$2"
		shift 2
		;;
	--curl-bin)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		curl_bin="$2"
		shift 2
		;;
	--node-bin)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		node_bin="$2"
		shift 2
		;;
	--credentials-helper)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		credentials_helper="$2"
		shift 2
		;;
	--max-attempts)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		max_attempts="$2"
		shift 2
		;;
	--poll-interval-seconds)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		poll_interval_seconds="$2"
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'unknown argument: %s\n' "$1" >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "$device_url" || -z "$pool_credentials" || -z "$out_dir" ]]; then
	usage
	exit 2
fi

if [[ ! "$max_attempts" =~ ^[0-9]+$ || "$max_attempts" -lt 1 ]]; then
	printf 'max attempts must be a positive integer\n' >&2
	exit 2
fi

if [[ ! "$poll_interval_seconds" =~ ^[0-9]+$ ]]; then
	printf 'poll interval seconds must be a non-negative integer\n' >&2
	exit 2
fi

mkdir -p "$out_dir"
readonly log_file="${out_dir}/pool-input-bridge.log"
readonly patch_response_file="${out_dir}/patch-response.redacted.txt"
readonly patch_error_file="${out_dir}/patch-error.redacted.txt"
readonly logs_file="${out_dir}/logs.redacted.txt"
: >"$log_file"
: >"$patch_response_file"
: >"$patch_error_file"
: >"$logs_file"

redact_text() {
	LC_ALL=C tr -d '\000\r' |
		sed -E 's#https?://[^[:space:]"<>]+#[redacted-url]#g; s#wss?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.:-]+/\1[redacted-host]/g; s/(Failed to connect to )[[:alnum:]_.:-]+/\1[redacted-host]/g; s/"(ssid|wifiPass|wifiPassword|stratumURL|stratumUser|stratumPassword|stratumCert|poolUrl|poolURL|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|apiToken|apiKey|token|password|user|worker|poolUser|poolPassword)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort|poolPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g; s#(BITAXE_POOL_PASSWORD=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_USER=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_URL=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_PORT=)[^[:space:]]+#\1[redacted]#g; s#(DEVICE_URL=)[^[:space:]]+#\1[redacted]#g'
}

log() {
	printf '%s\n' "$*" | redact_text | tee -a "$log_file" >/dev/null
}

append_redacted() {
	redact_text | tee -a "$log_file" >/dev/null
}

clear_pool_credentials_env() {
	unset BITAXE_POOL_URL BITAXE_POOL_PORT BITAXE_POOL_USER BITAXE_POOL_PASSWORD
}

record_blocked() {
	local reason="$1"

	log "pool_input_bridge_status=blocked"
	log "pool_settings_consumed_by_runtime=false"
	log "pool_input_bridge_blocker=${reason}"
	log "pool_config=local-owner-supplied"
	log "raw_pool_values_committed=no"
	log "network_scan=disabled"
	log "redaction_status=passed"
}

origin_url() {
	local origin="${device_url%/}"
	local without_scheme

	case "$origin" in
	http://* | https://*) ;;
	*)
		return 1
		;;
	esac

	without_scheme="${origin#http://}"
	if [[ "$without_scheme" == "$origin" ]]; then
		without_scheme="${origin#https://}"
	fi

	if [[ -z "$without_scheme" || "$without_scheme" == */* ]]; then
		return 1
	fi

	printf '%s' "$origin"
}

load_pool_credentials() {
	clear_pool_credentials_env

	if [[ ! -f "$pool_credentials" ]]; then
		log "pool_credentials_status=blocked - missing json file"
		return 1
	fi

	if [[ ! -f "$credentials_helper" ]]; then
		log "pool_credentials_status=blocked - json helper missing"
		return 1
	fi

	local maybe_exports
	set +e
	maybe_exports="$("$node_bin" "$credentials_helper" "$pool_credentials" 2>&1)"
	local helper_status=$?
	set -e

	if [[ "$helper_status" -ne 0 ]]; then
		printf '%s\n' "$maybe_exports" | append_redacted
		log "pool_credentials_status=blocked - invalid json"
		return 1
	fi

	eval "$maybe_exports"
	export BITAXE_POOL_URL BITAXE_POOL_PORT BITAXE_POOL_USER BITAXE_POOL_PASSWORD
	log "pool_credentials_status=loaded-redacted source=json"
}

write_patch_body() {
	local patch_body="$1"

	BITAXE_POOL_URL="$BITAXE_POOL_URL" \
		BITAXE_POOL_PORT="$BITAXE_POOL_PORT" \
		BITAXE_POOL_USER="$BITAXE_POOL_USER" \
		BITAXE_POOL_PASSWORD="$BITAXE_POOL_PASSWORD" \
		"$node_bin" - "$patch_body" <<'NODE'
const fs = require("node:fs");

const [, , outPath] = process.argv;
const poolPort = Number(process.env.BITAXE_POOL_PORT);
const body = {
  stratumURL: process.env.BITAXE_POOL_URL,
  stratumPort: poolPort,
  stratumUser: process.env.BITAXE_POOL_USER,
  stratumPassword: process.env.BITAXE_POOL_PASSWORD,
};

fs.writeFileSync(outPath, `${JSON.stringify(body)}\n`);
NODE
}

patch_pool_settings() {
	local origin="$1"
	local patch_body="$2"
	local response_tmp="$3"
	local error_tmp="$4"

	set +e
	local status
	status="$("$curl_bin" --silent --show-error --max-time 10 \
		--request PATCH \
		--header 'Content-Type: application/json' \
		--data-binary "@${patch_body}" \
		--output "$response_tmp" \
		--write-out "%{http_code}" \
		"${origin}/api/system" 2>"$error_tmp")"
	local curl_status=$?
	set -e

	redact_text <"$response_tmp" >"$patch_response_file"
	redact_text <"$error_tmp" >"$patch_error_file"
	log "pool_patch_http_status=${status:-000}"
	log "pool_patch_curl_status=${curl_status}"

	if [[ "$curl_status" -ne 0 || ! "${status:-000}" =~ ^2[0-9][0-9]$ ]]; then
		return 1
	fi

	return 0
}

poll_consumed_marker() {
	local origin="$1"
	local attempt

	for ((attempt = 1; attempt <= max_attempts; attempt++)); do
		local logs_tmp
		local logs_error_tmp
		logs_tmp="$(mktemp "${TMPDIR:-/tmp}/phase21-pool-input-logs.XXXXXX")"
		logs_error_tmp="$(mktemp "${TMPDIR:-/tmp}/phase21-pool-input-logs-error.XXXXXX")"

		set +e
		local status
		status="$("$curl_bin" --silent --show-error --max-time 10 \
			--output "$logs_tmp" \
			--write-out "%{http_code}" \
			"${origin}/api/system/logs" 2>"$logs_error_tmp")"
		local curl_status=$?
		set -e

		redact_text <"$logs_tmp" >"$logs_file"
		redact_text <"$logs_error_tmp" >>"$patch_error_file"
		rm -f "$logs_tmp" "$logs_error_tmp"

		log "pool_input_logs_attempt=${attempt} http_status=${status:-000} curl_status=${curl_status}"
		if [[ "$curl_status" -eq 0 && "${status:-000}" =~ ^2[0-9][0-9]$ ]] &&
			grep -Fq "phase21_pool_settings_consumed=true source=settings_patch" "$logs_file"; then
			return 0
		fi

		if [[ "$attempt" -lt "$max_attempts" && "$poll_interval_seconds" -gt 0 ]]; then
			sleep "$poll_interval_seconds"
		fi
	done

	return 1
}

log "phase21_pool_input_bridge"
log "network_scan: disabled - DEVICE_URL must be explicit"

if ! origin="$(origin_url)"; then
	record_blocked "invalid_device_url"
	exit 0
fi

if ! load_pool_credentials; then
	record_blocked "missing_or_invalid_pool_credentials"
	exit 0
fi

patch_tmp="$(mktemp "${TMPDIR:-/tmp}/phase21-pool-input-patch.XXXXXX.json")"
response_tmp="$(mktemp "${TMPDIR:-/tmp}/phase21-pool-input-response.XXXXXX")"
error_tmp="$(mktemp "${TMPDIR:-/tmp}/phase21-pool-input-error.XXXXXX")"
cleanup() {
	rm -f "$patch_tmp" "$response_tmp" "$error_tmp"
	clear_pool_credentials_env
}
trap cleanup EXIT

if ! write_patch_body "$patch_tmp"; then
	record_blocked "patch_body_generation_failed"
	exit 0
fi

if ! patch_pool_settings "$origin" "$patch_tmp" "$response_tmp" "$error_tmp"; then
	record_blocked "settings_patch_failed"
	exit 0
fi

if ! poll_consumed_marker "$origin"; then
	record_blocked "pool_settings_consumed_marker_missing"
	exit 0
fi

log "pool_input_bridge_status=applied"
log "pool_settings_consumed_by_runtime=true"
log "pool_config=local-owner-supplied"
log "raw_pool_values_committed=no"
log "network_scan=disabled"
log "redaction_status=passed"
