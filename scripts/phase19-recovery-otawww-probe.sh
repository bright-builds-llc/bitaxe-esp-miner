#!/usr/bin/env bash
# Recovery delegation and OTAWWW probe helpers for Phase 19.

safe_header_summary() {
	local path="$1"
	local content_type

	if [[ ! -f "$path" ]]; then
		printf 'not captured'
		return
	fi

	content_type="$(
		LC_ALL=C tr -d '\000\r' <"$path" |
			grep -i -m 1 '^content-type:' |
			sed -E 's/^[Cc][Oo][Nn][Tt][Ee][Nn][Tt]-[Tt][Yy][Pp][Ee]:[[:space:]]*//; s/[^[:alnum:]\/.+;= _-]/_/g' |
			head -c 80 || true
	)"
	if [[ -z "$content_type" ]]; then
		printf 'headers redacted - no allowlisted headers'
		return
	fi

	printf 'content-type: %s' "$content_type"
}

safe_body_summary() {
	local path="$1"

	if [[ ! -f "$path" ]]; then
		printf 'not captured'
		return
	fi

	if grep -Fq 'Wrong API input' "$path"; then
		printf 'contains Wrong API input'
		return
	fi

	printf 'body redacted - unexpected shape'
}

safe_curl_error_summary() {
	local path="$1"

	if [[ ! -s "$path" ]]; then
		printf 'none'
		return
	fi

	printf 'curl error redacted'
}

run_phase16_recovery() {
	local args=(
		--manifest "$manifest"
		--factory-image "$factory_image"
		--ota-image "$ota_image"
		--port "$port"
		--out-dir "$recovery_dir"
	)

	if [[ -n "$device_url" ]]; then
		args+=(--device-url "$device_url")
	fi
	if [[ "$allow_failed_update" -eq 1 ]]; then
		args+=(--allow-failed-update)
	fi
	if [[ "$allow_large_erase" -eq 1 ]]; then
		args+=(--allow-large-erase)
	fi
	if [[ "$allow_interrupted_ota" -eq 1 ]]; then
		args+=(--allow-interrupted-ota)
	fi

	if [[ ! -f "$recovery_script" ]]; then
		log_main "recovery_helper_status: blocked - helper missing"
		return 1
	fi

	log_main "recovery_helper: scripts/phase16-recovery-regression.sh"
	"$BASH" "$recovery_script" "${args[@]}" >>"$log_file" 2>&1
}

write_otawww_gap_without_target() {
	: >"$otawww_gap_log"
	log_otawww "phase19_recovery_otawww_evidence"
	log_otawww "network_scan: disabled"
	log_otawww "otawww_status: blocked - missing DEVICE_URL"
	log_otawww "otawww_claim: REL-03 gap"
	log_otawww "whole_www_update_proof: absent"
	log_otawww "www_bin_proof: absent - www.bin is package evidence only"
	log_otawww "route_presence_proof: absent - route presence is not update proof"
	log_otawww "wrong_api_input_proof: absent - Wrong API input is not whole-www update proof"
}

run_otawww_gap_probe() {
	local raw_root="${PHASE19_RAW_DIR:-target/phase19-recovery-regression-and-otawww-evidence-dev-raw/otawww}"
	local raw_dir
	local headers
	local body
	local error
	local empty_payload
	local safe_headers="${otawww_dir}/otawww.headers.txt"
	local safe_body="${otawww_dir}/otawww.body.txt"
	local safe_error="${otawww_dir}/otawww.curl-error.txt"
	local url="${device_url%/}/api/system/OTAWWW"
	local status
	local curl_status
	local header_summary
	local body_summary
	local error_summary
	local wrong_api_input_proof

	ensure_allowed_write_path "OTAWWW raw out-dir" "$raw_root"
	mkdir -p "$raw_root"
	raw_dir="$(mktemp -d "${raw_root%/}/probe.XXXXXX")"
	headers="${raw_dir}/otawww.headers.txt"
	body="${raw_dir}/otawww.body.txt"
	error="${raw_dir}/otawww.curl-error.txt"
	empty_payload="${raw_dir}/empty-otawww-upload.bin"

	: >"$headers"
	: >"$body"
	: >"$error"
	: >"$empty_payload"
	: >"$safe_headers"
	: >"$safe_body"
	: >"$safe_error"
	: >"$otawww_gap_log"

	set +e
	status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$headers" --output "$body" --write-out "%{http_code}" --request POST --data-binary "@${empty_payload}" "$url" 2>"$error")"
	curl_status=$?
	set -e

	header_summary="$(safe_header_summary "$headers")"
	body_summary="$(safe_body_summary "$body")"
	error_summary="$(safe_curl_error_summary "$error")"
	printf '%s\n' "$header_summary" >"$safe_headers"
	printf '%s\n' "$body_summary" >"$safe_body"
	printf '%s\n' "$error_summary" >"$safe_error"

	log_otawww "phase19_recovery_otawww_evidence"
	log_otawww "network_scan: disabled"
	log_otawww "otawww_route: POST /api/system/OTAWWW"
	log_otawww "otawww_request: bounded empty POST"
	log_otawww "otawww_curl_status: ${curl_status}"
	log_otawww "otawww_public_status: ${status}"
	log_otawww "otawww_selected_headers: ${header_summary}"
	log_otawww "otawww_public_body: ${body_summary}"
	log_otawww "otawww_curl_error: ${error_summary}"
	if [[ "$curl_status" -ne 0 || ! "$status" =~ ^[0-9][0-9][0-9]$ ]]; then
		log_otawww "otawww_status: blocked - curl failed"
		log_otawww "otawww_claim: REL-03 gap"
		log_otawww "whole_www_update_proof: absent"
		log_otawww "www_bin_proof: absent - www.bin is package evidence only"
		log_otawww "route_presence_proof: absent - route presence is not update proof"
		log_otawww "wrong_api_input_proof: absent - HTTP response unavailable"
		return 0
	fi
	wrong_api_input_proof="absent - response did not match expected public fail-closed shape"
	if [[ "$status" == "400" ]] && grep -Fq 'Wrong API input' "$body"; then
		log_otawww "current_public_route_behavior: Wrong API input"
		wrong_api_input_proof="present - Wrong API input is not whole-www update proof"
	else
		log_otawww "current_public_route_behavior: unexpected response - Wrong API input not cited"
	fi
	log_otawww "wrong_api_input_proof: ${wrong_api_input_proof}"
	log_otawww "otawww_status: captured - gap evidence only"
	log_otawww "otawww_claim: REL-03 gap"
	log_otawww "whole_www_update_proof: absent"
	log_otawww "www_bin_proof: absent - www.bin is package evidence only"
	log_otawww "route_presence_proof: absent - route presence is not update proof"
}

log_allow_flag_status() {
	local flag_name="$1"
	local supplied="$2"

	if [[ "$supplied" -eq 1 ]]; then
		log_main "${flag_name}: supplied"
		return
	fi
	log_main "${flag_name}: omitted"
}
