#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly bridge="${PHASE21_POOL_INPUT_BRIDGE_SCRIPT:-${script_dir}/phase21-pool-input-bridge.sh}"
readonly credentials_helper="${PHASE21_POOL_CREDENTIALS_HELPER:-${script_dir}/phase21-pool-credentials-json.mjs}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase21-pool-input-bridge-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

write_credentials() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "poolURL": "private-json.example",
  "poolPort": 3333,
  "poolUser": "bc1q-json-owner-address.bitaxe",
  "poolPassword": "json-secret-password"
}
JSON
}

write_invalid_credentials() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "poolURL": "private-json.example",
  "poolUser": "bc1q-json-owner-address.bitaxe",
  "poolPassword": "json-secret-password"
}
JSON
}

write_fake_curl() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE21_CURL_MUST_NOT_RUN:-0}" == "1" ]]; then
	printf 'curl should not have been called\n' >&2
	exit 99
fi

out_file=""
url=""
while [[ $# -gt 0 ]]; do
	case "$1" in
	--output)
		out_file="$2"
		shift 2
		;;
	--write-out)
		shift 2
		;;
	--data-binary | --header | --request | --max-time)
		shift 2
		;;
	--silent | --show-error)
		shift
		;;
	*)
		url="$1"
		shift
		;;
	esac
done

case "$url" in
*/api/system)
	if [[ -n "${PHASE21_FAKE_CURL_ARGS:-}" ]]; then
		printf 'PATCH %s\n' "$url" >>"$PHASE21_FAKE_CURL_ARGS"
	fi
	if [[ -n "$out_file" ]]; then
		printf '{"stratumURL":"stratum+tcp://private-json.example:3333","stratumUser":"bc1q-json-owner-address.bitaxe","stratumPassword":"json-secret-password","ip":"10.0.0.2"}\n' >"$out_file"
	fi
	printf '%s' "${PHASE21_FAKE_PATCH_HTTP_STATUS:-200}"
	exit "${PHASE21_FAKE_PATCH_CURL_STATUS:-0}"
	;;
*/api/system/logs)
	if [[ -n "${PHASE21_FAKE_CURL_ARGS:-}" ]]; then
		printf 'GET %s\n' "$url" >>"$PHASE21_FAKE_CURL_ARGS"
	fi
	if [[ -n "$out_file" ]]; then
		if [[ "${PHASE21_FAKE_LOGS_CONSUMED:-1}" == "1" ]]; then
			printf 'phase21_pool_settings_consumed=true source=settings_patch redacted=true\n"stratumUser":"bc1q-json-owner-address.bitaxe"\nip=10.0.0.2\n' >"$out_file"
		else
			printf 'axeos_settings_patch=effects_applied\n"stratumUser":"bc1q-json-owner-address.bitaxe"\n' >"$out_file"
		fi
	fi
	printf '%s' "${PHASE21_FAKE_LOGS_HTTP_STATUS:-200}"
	exit "${PHASE21_FAKE_LOGS_CURL_STATUS:-0}"
	;;
*)
	printf 'unexpected URL: %s\n' "$url" >&2
	exit 98
	;;
esac
SH
	chmod +x "$path"
}

write_fake_node() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

exec "${PHASE21_REAL_NODE_BIN:-node}" "$@"
SH
	chmod +x "$path"
}

run_bridge() {
	local out_dir="$1"
	local credentials="$2"
	shift 2

	PHASE21_POOL_CREDENTIALS_HELPER="$credentials_helper" \
		PHASE21_REAL_NODE_BIN="${PHASE21_REAL_NODE_BIN:-node}" \
		"$BASH" "$bridge" \
		--device-url "https://10.0.0.2" \
		--pool-credentials "$credentials" \
		--out-dir "$out_dir" \
		--curl-bin "$fake_curl" \
		--node-bin "$fake_node" \
		--credentials-helper "$credentials_helper" \
		--max-attempts 1 \
		--poll-interval-seconds 0 \
		"$@"
}

base_fixture_paths() {
	local prefix="$1"

	fake_curl="${tmp_root}/${prefix}-fake-curl"
	fake_node="${tmp_root}/${prefix}-fake-node"
	credentials="${tmp_root}/${prefix}-pool-credentials.json"

	write_fake_curl "$fake_curl"
	write_fake_node "$fake_node"
	write_credentials "$credentials"
}

test_applied_bridge_redacts_values() {
	base_fixture_paths "applied"
	local out_dir="${tmp_root}/applied-out"
	local curl_args="${tmp_root}/applied-curl-args.txt"

	PHASE21_FAKE_CURL_ARGS="$curl_args" run_bridge "$out_dir" "$credentials"

	assert_contains "${out_dir}/pool-input-bridge.log" "pool_input_bridge_status=applied"
	assert_contains "${out_dir}/pool-input-bridge.log" "pool_settings_consumed_by_runtime=true"
	assert_contains "$curl_args" "PATCH https://10.0.0.2/api/system"
	assert_contains "$curl_args" "GET https://10.0.0.2/api/system/logs"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "10.0.0.2"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "private-json.example"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "bc1q-json-owner-address"
	assert_not_contains "${out_dir}/patch-response.redacted.txt" "json-secret-password"
	assert_not_contains "${out_dir}/logs.redacted.txt" "10.0.0.2"
	assert_not_contains "${out_dir}/logs.redacted.txt" "bc1q-json-owner-address"
}

test_patch_timeout_blocks_without_raw_values() {
	base_fixture_paths "timeout"
	local out_dir="${tmp_root}/timeout-out"

	PHASE21_FAKE_PATCH_HTTP_STATUS=000 \
		PHASE21_FAKE_PATCH_CURL_STATUS=28 \
		run_bridge "$out_dir" "$credentials"

	assert_contains "${out_dir}/pool-input-bridge.log" "pool_input_bridge_status=blocked"
	assert_contains "${out_dir}/pool-input-bridge.log" "pool_input_bridge_blocker=settings_patch_failed"
	assert_contains "${out_dir}/pool-input-bridge.log" "pool_patch_curl_status=28"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "10.0.0.2"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "private-json.example"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "json-secret-password"
}

test_missing_consumed_marker_blocks() {
	base_fixture_paths "missing-marker"
	local out_dir="${tmp_root}/missing-marker-out"

	PHASE21_FAKE_LOGS_CONSUMED=0 run_bridge "$out_dir" "$credentials"

	assert_contains "${out_dir}/pool-input-bridge.log" "pool_input_bridge_status=blocked"
	assert_contains "${out_dir}/pool-input-bridge.log" "pool_input_bridge_blocker=pool_settings_consumed_marker_missing"
	assert_contains "${out_dir}/logs.redacted.txt" "axeos_settings_patch=effects_applied"
	assert_not_contains "${out_dir}/logs.redacted.txt" "bc1q-json-owner-address"
}

test_invalid_json_credentials_block_without_curl() {
	base_fixture_paths "invalid-json"
	local out_dir="${tmp_root}/invalid-json-out"
	write_invalid_credentials "$credentials"

	PHASE21_CURL_MUST_NOT_RUN=1 run_bridge "$out_dir" "$credentials"

	assert_contains "${out_dir}/pool-input-bridge.log" "pool_credentials_status=blocked - invalid json"
	assert_contains "${out_dir}/pool-input-bridge.log" "pool_input_bridge_blocker=missing_or_invalid_pool_credentials"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "private-json.example"
	assert_not_contains "${out_dir}/pool-input-bridge.log" "bc1q-json-owner-address"
}

test_applied_bridge_redacts_values
test_patch_timeout_blocks_without_raw_values
test_missing_consumed_marker_blocks
test_invalid_json_credentials_block_without_curl

printf 'phase21 pool input bridge tests passed\n'
