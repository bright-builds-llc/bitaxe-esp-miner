#!/usr/bin/env bash
# Runtime capture, HTTP, and device-session tests for Phase 35.

run_production_capture_case() {
	local scenario="$1"
	local capture_root="${test_root}/production-capture-${scenario}"
	rm -rf "$capture_root"
	mkdir -p "$capture_root/raw"
	chmod 700 "$capture_root" "$capture_root/raw"
	printf '%s\n' 1000 >"$capture_root/raw/monotonic-state"
	chmod 600 "$capture_root/raw/monotonic-state"
	jq -cn \
		--arg session 0123456789abcdef0011223344556677 \
		'{status:"passed",category:"none",session:$session,boot_ordinal:51}' \
		>"$capture_root/raw/boot-a-setup.json"
	chmod 600 "$capture_root/raw/boot-a-setup.json"

	(
		local fixture_command=""
		local workspace_dir="$workspace"
		local local_root="$capture_root"
		local target_token=synthetic-origin
		local failure_category=""
		local capture_scenario="$scenario"

		# shellcheck source=scripts/phase35-correlated-evidence-root.sh
		source "${script_dir}/phase35-correlated-evidence-root.sh"
		# shellcheck source=scripts/phase35-correlated-evidence-effects.sh
		source "${script_dir}/phase35-correlated-evidence-effects.sh"

		monotonic_millis() {
			local state_file="$local_root/raw/monotonic-state"
			local current
			current="$(<"$state_file")"
			printf '%s\n' "$((current + 100))" >"$state_file"
			printf '%s\n' "$current"
		}

		curl() {
			local output=""
			local endpoint=""
			local http1=false
			local no_proxy=""
			local proto=""
			local connect_timeout=""
			local max_time=""
			local max_filesize=""
			while (($#)); do
				case "$1" in
				--output)
					output="$2"
					shift 2
					;;
				--http1.1)
					http1=true
					shift
					;;
				--noproxy)
					no_proxy="$2"
					shift 2
					;;
				--proto)
					proto="$2"
					shift 2
					;;
				--connect-timeout)
					connect_timeout="$2"
					shift 2
					;;
				--max-time)
					max_time="$2"
					shift 2
					;;
				--max-filesize)
					max_filesize="$2"
					shift 2
					;;
				--silent | --show-error | --fail)
					shift
					;;
				*)
					endpoint="$1"
					shift
					;;
				esac
			done
			[[ "$http1" == true && "$no_proxy" == '*' && "$proto" == '=http,https' ]] ||
				return 92
			[[ "$connect_timeout" == 5 && "$max_time" == 10 ]] || return 92
			case "$endpoint" in
			*/api/system/info)
				[[ "$max_filesize" == 65536 ]] || return 92
				if [[ "$capture_scenario" == missing_hostname ]]; then
					jq -cn \
						--arg bootSession 0123456789abcdef0011223344556677 \
						'{bootSession:$bootSession,operatorSnapshotRevision:20}' >"$output"
				else
					jq -cn \
						--arg bootSession 0123456789abcdef0011223344556677 \
						'{bootSession:$bootSession,operatorSnapshotRevision:20,hostname:"synthetic-host"}' >"$output"
				fi
				;;
			*/api/system/logs)
				[[ "$max_filesize" == 524288 ]] || return 92
				printf '%s\n' \
					'operator_snapshot session=0123456789abcdef0011223344556677 revision=20 redacted=true' \
					>"$output"
				if [[ "$capture_scenario" != missing_websocket_marker ]]; then
					printf '%s\n' \
						'operator_snapshot session=0123456789abcdef0011223344556677 revision=21 redacted=true' \
						>>"$output"
				fi
				;;
			*) return 91 ;;
			esac
		}

		node() {
			local output=""
			while (($#)); do
				if [[ "$1" == --out ]]; then
					output="$2"
					shift 2
					continue
				fi
				shift
			done
			local revision=21
			[[ "$capture_scenario" != same_revision ]] || revision=20
			jq -cn \
				--arg session 0123456789abcdef0011223344556677 \
				--argjson revision "$revision" \
				'{event:"system_info",data:{bootSession:$session,operatorSnapshotRevision:$revision}}' |
				sed 's/^/websocket_frame_1=/' >"$output"
			printf 'websocket_close_status=closed\n' >>"$output"
		}

		capture_epoch boot-a-pre
	)
}

test_production_capture_preserves_real_epoch_boundaries() {
	# Arrange and Act
	local snapshot
	snapshot="$(run_production_capture_case success)" ||
		fail_test "production capture fixture failed"

	# Assert
	[[ "$(jq -r '.boot_ordinal' "$snapshot")" == 51 ]] ||
		fail_test "capture did not use the serial-classified boot ordinal"
	[[ "$(jq -r '.storage_revision' "$snapshot")" == 20 ]] ||
		fail_test "capture did not retain the API storage revision"
	[[ "$(jq -r '.setting_digest' "$snapshot")" == "$(text_digest synthetic-host)" ]] ||
		fail_test "capture did not hash the validated private setting"
	assert_contains "$snapshot" 'operatorSnapshotRevision\\\":21'
	assert_absent "$snapshot" 'live_websocket_json: .*event'
	assert_line "${test_root}/production-capture-success/raw/boot-a-pre-retained.log" \
		'operator_snapshot session=0123456789abcdef0011223344556677 revision=20 redacted=true'
	assert_line "${test_root}/production-capture-success/raw/boot-a-pre-retained.log" \
		'operator_snapshot session=0123456789abcdef0011223344556677 revision=21 redacted=true'
	(("$(jq -r '.ended_millis' "$snapshot")" > "$(jq -r '.started_millis' "$snapshot")")) ||
		fail_test "capture did not preserve a positive real interval"
	local protected_file
	for protected_file in \
		"${test_root}/production-capture-success/raw/boot-a-pre-api.json" \
		"${test_root}/production-capture-success/raw/boot-a-pre-api.stderr" \
		"${test_root}/production-capture-success/raw/boot-a-pre-websocket.log" \
		"${test_root}/production-capture-success/raw/boot-a-pre-websocket.stderr" \
		"${test_root}/production-capture-success/raw/boot-a-pre-retained.log" \
		"${test_root}/production-capture-success/raw/boot-a-pre-retained.stderr"; do
		[[ "$(file_mode "$protected_file")" == 600 ]] ||
			fail_test "production capture artifact is not mode 0600"
	done
}

test_production_capture_rejects_incoherent_boundaries() {
	local scenario
	for scenario in same_revision missing_websocket_marker missing_hostname; do
		# Arrange and Act
		local output="${test_root}/${scenario}.stdout"
		local error="${test_root}/${scenario}.stderr"
		set +e
		run_production_capture_case "$scenario" >"$output" 2>"$error"
		local result_code=$?
		set -e

		# Assert
		[[ "$result_code" != 0 ]] ||
			fail_test "${scenario} production capture was accepted"
		[[ ! -s "$output" && ! -s "$error" ]] ||
			fail_test "${scenario} production failure exposed private diagnostics"
		[[ ! -e "${test_root}/production-capture-${scenario}/raw/boot-a-pre.json" ]] ||
			fail_test "${scenario} production failure created a usable snapshot"
	done
}

test_production_reboot_uses_device_session_hybrid_quorum() {
	# Arrange
	prepare_case production_reboot
	prepare_direct_flash_stubs
	local reboot_root="${case_dir}/reboot-root"
	mkdir -p "$reboot_root/raw" "$reboot_root/artifacts"
	chmod 700 "$reboot_root" "$reboot_root/raw" "$reboot_root/artifacts"
	jq -cn \
		--arg session aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
		'{status:"passed",category:"none",session:$session,boot_ordinal:7,device_url:"synthetic-origin"}' \
		>"$reboot_root/raw/boot-a-setup.json"
	chmod 600 "$reboot_root/raw/boot-a-setup.json"
	local private_origin="http://127.0.0.1"
	local private_hostname="phase35-private-hostname"
	local physical_identity="dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"

	# Act
	(
		export PHASE35_FIXTURE_STATE="$state_dir"
		export PHASE35_DEVICE_SESSION_CALLS="$device_session_calls"
		export PHASE35_TEST_STUB_DISPATCH=true
		local fixture_command=""
		local workspace_dir="$workspace"
		local local_root="$reboot_root"
		local manifest="${manifest_dir}/manifest.json"
		local target_token="$private_origin"
		local port=synthetic-port
		local physical_identity_digest="$physical_identity"
		local mutated_setting="$private_hostname"
		local capture_timeout_seconds=360
		local failure_category=""

		# shellcheck source=scripts/phase35-correlated-evidence-root.sh
		source "${script_dir}/phase35-correlated-evidence-root.sh"
		# shellcheck source=scripts/phase35-correlated-evidence-effects.sh
		source "${script_dir}/phase35-correlated-evidence-effects.sh"
		resolve_device_session_executable() {
			printf '%s\n' "${workspace}/bazel-bin/tools/device-session/device-session"
		}

		run_device_session_reboot
		[[ "$target_token" == "$private_origin" ]]
	)

	# Assert
	[[ "$(rg -c '^CALL$' "$device_session_calls")" == 1 ]] ||
		fail_test "device-session reboot did not run exactly once"
	assert_line "$device_session_calls" 'arg=reboot'
	assert_line "$device_session_calls" 'arg=--timeout-seconds'
	assert_line "$device_session_calls" 'arg=360'
	[[ ! -e "$classifier_calls" ]] ||
		fail_test "the obsolete serial-only reboot classifier still ran"
	local request_input="$reboot_root/raw/device-session-request.json"
	local session_root="$reboot_root/raw/device-session"
	local projection="$reboot_root/raw/device-session-projection.json"
	[[ "$(file_mode "$request_input")" == 600 ]] ||
		fail_test "device-session request is not mode 0600"
	[[ "$(file_mode "$session_root")" == 700 ]] ||
		fail_test "device-session private root is not mode 0700"
	[[ "$(file_mode "$session_root/result.private.json")" == 600 ]] ||
		fail_test "device-session private result is not mode 0600"
	[[ "$(file_mode "$projection")" == 600 ]] ||
		fail_test "device-session projection is not mode 0600"
	[[ "$(file_mode "$reboot_root/raw/boot-b-setup.json")" == 600 ]] ||
		fail_test "Boot B hybrid classification is not mode 0600"
	[[ "$(jq -r '.session' "$reboot_root/raw/boot-b-setup.json")" == cccccccccccccccccccccccccccccccc ]] ||
		fail_test "Boot B session did not advance"
	[[ "$(jq -r '.boot_ordinal' "$reboot_root/raw/boot-b-setup.json")" == 8 ]] ||
		fail_test "Boot B ordinal is not the exact successor"
	[[ "$(jq -r '.reset_reason' "$reboot_root/raw/boot-b-setup.json")" == software_cpu ]] ||
		fail_test "Boot B reset category is not software_cpu"
	assert_absent "$projection" "$private_origin|$private_hostname|$physical_identity|synthetic-port"
	assert_absent "$reboot_root/raw/device-session.stdout" ".+"
	assert_absent "$reboot_root/raw/device-session.stderr" ".+"
	local reboot_function
	reboot_function="$(sed -n '/^run_device_session_reboot()/,/^}/p' \
		"${script_dir}/phase35-correlated-evidence-runtime.sh")"
	[[ "$reboot_function" != *espflash* && "$reboot_function" != *phase33-classify* ]] ||
		fail_test "runtime reboot observation still depends on espflash or Phase 33 serial classification"
}

test_just_entrypoint_builds_the_current_package_before_supervisor() {
	# Arrange
	local expected_recipe
	expected_recipe=$'phase35-evidence *args:\n    bazel build //firmware/bitaxe:firmware_image\n    bazel run //scripts:phase35_correlated_evidence -- {{ args }}'

	# Act
	local actual_recipe
	actual_recipe="$(awk '
		/^phase35-evidence \*args:$/ { capture = 1 }
		capture && /^[^[:space:]]/ && $0 !~ /^phase35-evidence \*args:$/ { exit }
		capture { print }
	' "$justfile")"

	# Assert
	[[ "$actual_recipe" == "$expected_recipe" ]] ||
		fail_test "phase35-evidence did not build the exact current package first"
}
