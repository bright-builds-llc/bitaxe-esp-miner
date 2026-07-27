#!/usr/bin/env bash
# Fake curl implementations for Phase 17 shell contract tests.

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
if [[ "${PHASE17_FAKE_CURL_STDERR_HOST:-0}" == "1" ]]; then
  printf "curl: (6) Could not resolve host: private-bitaxe.local\ncurl: (7) Failed to connect to 192.168.1.5 port 80\ncurl: (22) URL rejected: http://private-bitaxe.local/api/system/info?token=abc123\n" >&2
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
  "GET /phase17-missing-static")
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
    body="{\"boardVersion\":\"205\",\"asicModel\":\"BM1366\",\"ssid\":\"HomeNetwork\",\"wifiPass\":\"secret\",\"stratumUser\":\"worker\",\"stratumPassword\":\"pool-pass\",\"poolUrl\":\"stratum+tcp://pool.example:3333\",\"fallbackPoolUrl\":\"stratum+tcp://backup.example:3333\",\"hostname\":\"bitaxe-private\",\"ip\":\"192.168.1.5\",\"gateway\":\"192.168.1.1\",\"netmask\":\"255.255.255.0\",\"dns\":\"1.1.1.1\",\"token\":\"abc123\",\"apiKey\":\"key123\",\"password\":\"admin\",\"mac\":\"aa:bb:cc:dd:ee:ff\"}"
    ;;
  "GET /api/phase17-unknown")
    status=404
    headers="Content-Type: application/json"
    body="{\"error\":\"unknown route\"}"
    ;;
  "GET /api/ws")
    status="${PHASE17_FAKE_WS_STATUS:-426}"
    headers="Content-Type: text/plain"
    body="WebSocket upgrade required"
    ;;
  "GET /api/ws/live")
    status="${PHASE17_FAKE_WS_STATUS:-426}"
    headers="Content-Type: text/plain"
    body="WebSocket upgrade required"
    ;;
  "POST /api/system/OTA")
    status="${PHASE17_FAKE_OTA_STATUS:-500}"
    headers="Content-Type: text/plain"
    body="${PHASE17_FAKE_OTA_BODY:-Protocol Error}"
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
