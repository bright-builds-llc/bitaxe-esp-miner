#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly capture_script="${PHASE17_WEBSOCKET_CAPTURE_SCRIPT:-${script_dir}/phase17-websocket-capture.mjs}"
readonly peer_script="${PHASE17_WEBSOCKET_CLOSE_PEER_SCRIPT:-${script_dir}/phase17-websocket-close-peer.mjs}"
readonly node_bin="${NODE_BIN:-node}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase17-websocket-close-test.XXXXXX")"
readonly tmp_root
chmod 700 "$tmp_root"

peer_pid=""
cleanup() {
	if [[ -n "$peer_pid" ]] && kill -0 "$peer_pid" 2>/dev/null; then
		kill "$peer_pid" 2>/dev/null || true
		wait "$peer_pid" 2>/dev/null || true
	fi
	rm -rf "$tmp_root"
}
trap cleanup EXIT

ready_path="${tmp_root}/ready"
closed_path="${tmp_root}/closed"
peer_stderr="${tmp_root}/peer.stderr"
capture_path="${tmp_root}/capture.log"
capture_stderr="${tmp_root}/capture.stderr"

umask 077
"$node_bin" "$peer_script" --ready "$ready_path" --closed "$closed_path" \
	>/dev/null 2>"$peer_stderr" &
peer_pid="$!"

for _ in {1..100}; do
	[[ -f "$ready_path" ]] && break
	sleep 0.05
done
[[ -f "$ready_path" ]]
port="$(<"$ready_path")"
[[ "$port" =~ ^[0-9]+$ ]]

"$node_bin" "$capture_script" \
	--device-url "http://127.0.0.1:${port}" \
	--path /api/ws/live \
	--out "$capture_path" \
	--duration-ms 2000 \
	--max-frames 1 \
	>/dev/null 2>"$capture_stderr"

[[ -f "$closed_path" ]]
grep -Fqx 'websocket_open_status=opened' "$capture_path"
grep -Fqx 'websocket_frame_status=passed frames=1' "$capture_path"
grep -Fqx 'websocket_close_status=closed' "$capture_path"
[[ ! -s "$capture_stderr" ]]
[[ ! -s "$peer_stderr" ]]
[[ "$(stat -f '%Lp' "$tmp_root")" == "700" ]]
[[ "$(stat -f '%Lp' "$capture_path")" == "600" ]]
[[ "$(stat -f '%Lp' "$closed_path")" == "600" ]]

wait "$peer_pid"
peer_pid=""

printf 'phase17 websocket close-handshake test passed\n'
