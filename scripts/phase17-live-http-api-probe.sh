#!/usr/bin/env bash
# HTTP route probing and response-redaction helpers for Phase 17.

redact_stream() {
	LC_ALL=C tr -d '\000\r' |
		LC_ALL=C sed -E 's/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|token|apiKey|password|nvsSecret|secret)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s#https?://[^[:space:]"<>]+#[redacted-url]#g; s#wss?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.-]+/\1[redacted-host]/g; s/(Failed to connect to )([0-9]{1,3}\.){3}[0-9]{1,3}/\1[redacted-ip]/g; s/(Failed to connect to )[[:alnum:]_.-]+/\1[redacted-host]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g'
}

write_redacted_artifact() {
	local source_file="$1"
	local artifact_file="$2"

	redact_stream <"$source_file" | head -c 4000 >"$artifact_file"
}

redacted_snippet() {
	local source_file="$1"

	redact_stream <"$source_file" |
		head -c 1000 |
		LC_ALL=C tr '\n\t' '  '
}

selected_headers() {
	local header_file="$1"

	while IFS= read -r line; do
		local clean_line="${line%$'\r'}"
		case "$clean_line" in
		[Cc][Oo][Nn][Tt][Ee][Nn][Tt]-[Tt][Yy][Pp][Ee]:* | \
			[Ll][Oo][Cc][Aa][Tt][Ii][Oo][Nn]:* | \
			[Cc][Aa][Cc][Hh][Ee]-[Cc][Oo][Nn][Tt][Rr][Oo][Ll]:* | \
			[Cc][Oo][Nn][Tt][Ee][Nn][Tt]-[Ee][Nn][Cc][Oo][Dd][Ii][Nn][Gg]:*)
			printf '%s\n' "$clean_line"
			;;
		esac
	done <"$header_file" | redact_stream
}

header_contains() {
	local header_file="$1"
	local pattern="$2"

	selected_headers "$header_file" | grep -Eiq "$pattern"
}

body_contains_marker() {
	local body_file="$1"
	local marker="$2"

	grep -Fq "$marker" "$body_file"
}

markers_match() {
	local body_file="$1"
	local markers="$2"

	if [[ -z "$markers" ]]; then
		return 0
	fi

	local marker
	local old_ifs="$IFS"
	IFS='|'
	for marker in $markers; do
		if ! body_contains_marker "$body_file" "$marker"; then
			IFS="$old_ifs"
			return 1
		fi
	done
	IFS="$old_ifs"
	return 0
}

status_matches() {
	local expected="$1"
	local actual="$2"

	if [[ "$expected" == "websocket-no-upgrade" ]]; then
		[[ "$actual" == "400" || "$actual" == "426" ]]
		return
	fi
	if [[ "$expected" == "ota-route-present" ]]; then
		[[ "$actual" == "400" || "$actual" == "409" || "$actual" == "413" || "$actual" == "415" || "$actual" == "422" || "$actual" == "500" ]]
		return
	fi
	[[ "$expected" == "$actual" ]]
}

ota_validation_path_present() {
	local body_file="$1"

	grep -Eiq 'Protocol Error|Write Error|Validation Error|Validation / Activation Error|Not allowed in AP mode|Firmware update|OTA|invalid|image|upload' "$body_file"
}

route_specific_markers_match() {
	local route_id="$1"
	local header_file="$2"
	local body_file="$3"

	case "$route_id" in
	app-css-gz)
		header_contains "$header_file" '^Content-Type:' &&
			header_contains "$header_file" '^Content-Encoding:[[:space:]]*gzip' &&
			header_contains "$header_file" '^Cache-Control:'
		;;
	missing-static)
		header_contains "$header_file" '^Location:[[:space:]]*/$'
		;;
	system-info)
		grep -Eq '^[[:space:]]*\{' "$body_file" &&
			grep -Fq '205' "$body_file" &&
			(grep -Fq 'BM1366' "$body_file" || grep -Fq 'Ultra' "$body_file")
		;;
	firmware-ota)
		ota_validation_path_present "$body_file"
		;;
	*)
		return 0
		;;
	esac
}

probe_route() {
	local id="$1"
	local method="$2"
	local path="$3"
	local expected_status="$4"
	local markers="$5"
	local expectation="$6"
	local header_file="${out_dir}/${id}.headers.txt"
	local body_file="${out_dir}/${id}.body.txt"
	local error_file="${out_dir}/${id}.curl-error.txt"
	local raw_header="${tmp_dir}/${id}.headers.raw"
	local raw_body="${tmp_dir}/${id}.body.raw"
	local raw_error="${tmp_dir}/${id}.curl-error.raw"
	local url="${base_url}${path}"

	: >"$raw_header"
	: >"$raw_body"
	: >"$raw_error"

	set +e
	local actual_status
	if [[ "$method" == "POST" ]]; then
		actual_status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$raw_header" --output "$raw_body" --write-out "%{http_code}" --request POST --data-binary "" "$url" 2>"$raw_error")"
	else
		actual_status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$raw_header" --output "$raw_body" --write-out "%{http_code}" "$url" 2>"$raw_error")"
	fi
	local curl_status=$?
	set -e

	selected_headers "$raw_header" >"$header_file"
	write_redacted_artifact "$raw_body" "$body_file"
	write_redacted_artifact "$raw_error" "$error_file"

	local route_conclusion="passed"
	local actual_result="matched"
	if [[ "$curl_status" -ne 0 ]]; then
		route_conclusion="blocked"
		actual_result="curl_error"
	elif ! status_matches "$expected_status" "$actual_status"; then
		route_conclusion="blocked"
		actual_result="unexpected_status"
	elif ! markers_match "$raw_body" "$markers"; then
		route_conclusion="blocked"
		actual_result="missing_expected_marker"
	elif ! route_specific_markers_match "$id" "$raw_header" "$raw_body"; then
		route_conclusion="blocked"
		actual_result="missing_route_specific_marker"
	fi

	if [[ "$route_conclusion" != "passed" ]]; then
		any_blocked=1
	fi

	log "route: ${method} ${path}"
	log "method: ${method}"
	log "path: ${path}"
	log "sanitized_target: $(redacted_target "$url")"
	log "expected_result: ${expectation}"
	log "actual_status: ${actual_status}"
	log "curl_status: ${curl_status}"
	log "selected_headers:"
	local headers
	headers="$(selected_headers "$raw_header")"
	if [[ -n "$headers" ]]; then
		while IFS= read -r header; do
			log "  ${header}"
		done <<<"$headers"
	else
		log "  none"
	fi
	local snippet
	snippet="$(redacted_snippet "$raw_body")"
	if [[ -z "$snippet" ]]; then
		snippet="[empty-body]"
	fi
	log "redacted_body_snippet: ${snippet}"
	if [[ -s "$raw_error" ]]; then
		log "curl_error: $(redacted_snippet "$raw_error")"
	fi
	log "actual_result: ${actual_result}"
	case "$id" in
	system-info)
		if [[ "$route_conclusion" == "passed" ]]; then
			log "system_info_device_marker: passed"
		else
			log "system_info_device_marker: blocked"
		fi
		;;
	api-ws | api-ws-live)
		if [[ "$route_conclusion" == "passed" ]]; then
			log "websocket_no_upgrade_claim: route-coexistence-only"
		fi
		;;
	firmware-ota)
		if [[ "$route_conclusion" == "passed" ]]; then
			log "ota_route_presence_claim: route-presence-only"
		else
			log "ota_route_presence_claim: blocked"
		fi
		log "ota_non_claims: valid OTA upload, invalid image rejection, reboot, rollback, selected partition, boot validation not claimed"
		;;
	otawww)
		log "otawww_rel03_status: deferred"
		;;
	esac
	log "route_conclusion: ${route_conclusion}"
	log ""
}
