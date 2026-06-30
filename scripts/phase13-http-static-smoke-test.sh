#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly smoke_script="${PHASE13_HTTP_STATIC_SMOKE_SCRIPT:-${script_dir}/phase13-http-static-smoke.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase13-http-static-smoke-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

write_executable() {
	local path="$1"
	local body="$2"

	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_manifest() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "source_commit": "190849539700b8f9a7909fd2b6ebd84142557968",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
}
JSON
}

create_no_curl_stub() {
	local path="$1"

	write_executable "$path" 'printf "curl should not have been called\n" >&2
exit 97
'
}

create_fake_curl() {
	local path="$1"

	write_executable "$path" 'header_file=""
body_file=""
method="GET"
url=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dump-header)
      header_file="$2"
      shift 2
      ;;
    --output)
      body_file="$2"
      shift 2
      ;;
    --request)
      method="$2"
      shift 2
      ;;
    --data-binary)
      shift 2
      ;;
    --silent | --show-error)
      shift
      ;;
    --max-time | --write-out)
      shift 2
      ;;
    http://* | https://*)
      url="$1"
      shift
      ;;
    *)
      printf "unexpected curl arg: %s\n" "$1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$header_file" || -z "$body_file" || -z "$url" ]]; then
  printf "missing curl fixture inputs\n" >&2
  exit 2
fi

path="/${url#*://*/}"
if [[ "$url" == "http://device.local/" ]]; then
  path="/"
fi

status=200
headers="Content-Type: text/plain"
body=""

case "${method} ${path}" in
  "GET /")
    headers="Content-Type: text/html"
    body="<title>AxeOS unavailable</title><a>Open recovery</a><a>Release metadata</a>"
    ;;
  "GET /assets/app.css.gz")
    headers=$'"'"'Content-Type: text/css\nCache-Control: max-age=2592000\nContent-Encoding: gzip'"'"'
    body="body { color: #00ff00; }"
    ;;
  "GET /phase13-missing-static")
    status=302
    headers=$'"'"'Content-Type: text/plain\nLocation: /'"'"'
    body="Redirect to the captive portal"
    ;;
  "GET /recovery")
    headers="Content-Type: text/html"
    body="<h1>AxeOS Recovery</h1><span>Response:</span>"
    ;;
  "GET /api/system/info")
    headers="Content-Type: application/json"
    body="{\"firmware_commit\":\"190849539700\",\"ssid\":\"HomeNetwork\",\"wifiPass\":\"secret\",\"stratumUser\":\"worker\",\"stratumCert\":\"PHASE13_LONG_CERT_SECRET_PREFIX_$(printf "x%.0s" {1..260})\",\"ip\":\"192.168.1.5\"}"
    ;;
  "GET /api/phase13-unknown")
    status=404
    headers="Content-Type: application/json"
    body="{\"error\":\"unknown route\"}"
    ;;
  "GET /api/ws")
    status="${PHASE13_FAKE_WS_STATUS:-400}"
    headers="Content-Type: text/plain"
    body="WebSocket upgrade required"
    ;;
  "GET /api/ws/live")
    status="${PHASE13_FAKE_WS_STATUS:-400}"
    headers="Content-Type: text/plain"
    body="WebSocket upgrade required"
    ;;
  "POST /api/system/OTAWWW")
    status=400
    headers="Content-Type: text/plain"
    body="Wrong API input"
    ;;
  *)
    printf "unhandled fake curl route: %s %s\n" "$method" "$path" >&2
    exit 3
    ;;
esac

printf "%s\n" "$headers" >"$header_file"
printf "%s\n" "$body" >"$body_file"
printf "%s" "$status"
'
}

test_missing_url_writes_blocker_without_curl() {
	local out_dir="${tmp_root}/missing-url"
	local manifest="${tmp_root}/manifest.json"
	local curl_stub="${tmp_root}/no-curl"

	create_manifest "$manifest"
	create_no_curl_stub "$curl_stub"

	CURL_BIN="$curl_stub" "$BASH" "$smoke_script" --manifest "$manifest" --out-dir "$out_dir"

	local log_file="${out_dir}/http-static-smoke.log"
	assert_contains "$log_file" "phase13_http_static_smoke"
	assert_contains "$log_file" "DEVICE_URL status: blocked - missing DEVICE_URL"
	assert_contains "$log_file" "http_static_status: blocked"
}

test_fake_success_records_required_paths() {
	local out_dir="${tmp_root}/fake-success"
	local manifest="${tmp_root}/manifest.json"
	local curl_stub="${tmp_root}/fake-curl"

	create_manifest "$manifest"
	create_fake_curl "$curl_stub"

	CURL_BIN="$curl_stub" "$BASH" "$smoke_script" \
		--device-url "http://device.local" \
		--manifest "$manifest" \
		--out-dir "$out_dir"

	local log_file="${out_dir}/http-static-smoke.log"
	assert_contains "$log_file" "DEVICE_URL status: provided"
	assert_contains "$log_file" "network_scan: disabled - using explicit DEVICE_URL only"
	assert_contains "$log_file" "route: GET /"
	assert_contains "$log_file" "route: GET /assets/app.css.gz"
	assert_contains "$log_file" "route: GET /phase13-missing-static"
	assert_contains "$log_file" "missing static redirect"
	assert_contains "$log_file" "route: GET /recovery"
	assert_contains "$log_file" "route: GET /api/system/info"
	assert_contains "$log_file" "route: GET /api/phase13-unknown"
	assert_contains "$log_file" "route: GET /api/ws"
	assert_contains "$log_file" "route: GET /api/ws/live"
	assert_contains "$log_file" "route: POST /api/system/OTAWWW"
	assert_contains "$log_file" "redacted_body_snippet:"
	assert_contains "$log_file" "\"ssid\":\"[redacted]\""
	assert_contains "$log_file" "\"wifiPass\":\"[redacted]\""
	assert_contains "$log_file" "\"stratumUser\":\"[redacted]\""
	assert_contains "$log_file" "\"stratumCert\":\"[redacted]\""
	assert_contains "$log_file" "Redirect to the captive portal"
	assert_contains "$log_file" "{\"error\":\"unknown route\"}"
	assert_contains "$log_file" "Wrong API input"
	assert_not_contains "$log_file" "sanitized_body_snippet:"
	assert_not_contains "$log_file" "HomeNetwork"
	assert_not_contains "$log_file" "secret"
	assert_not_contains "$log_file" "worker"
	assert_not_contains "$log_file" "PHASE13_LONG_CERT_SECRET_PREFIX"
	assert_not_contains "$log_file" "192.168.1.5"
	assert_contains "$log_file" "http_static_status: passed"
}

test_websocket_server_error_blocks_static_smoke() {
	local out_dir="${tmp_root}/websocket-500"
	local manifest="${tmp_root}/manifest-ws-500.json"
	local curl_stub="${tmp_root}/fake-curl-ws-500"

	create_manifest "$manifest"
	create_fake_curl "$curl_stub"

	PHASE13_FAKE_WS_STATUS=500 CURL_BIN="$curl_stub" "$BASH" "$smoke_script" \
		--device-url "http://device.local" \
		--manifest "$manifest" \
		--out-dir "$out_dir"

	local log_file="${out_dir}/http-static-smoke.log"
	assert_contains "$log_file" "route: GET /api/ws"
	assert_contains "$log_file" "actual_status: 500"
	assert_contains "$log_file" "route_conclusion: blocked"
	assert_contains "$log_file" "http_static_status: blocked"
}

if [[ ! -f "$smoke_script" ]]; then
	fail "smoke script missing: ${smoke_script}"
fi

test_missing_url_writes_blocker_without_curl
test_fake_success_records_required_paths
test_websocket_server_error_blocks_static_smoke

printf 'phase13_http_static_smoke_test passed\n'
