#!/usr/bin/env bash
# Detector, admission, readiness, and flash-probe helpers for Phase 35.

detector_failure() {
	local log="$1"
	local maybe_category
	maybe_category="$(sed -n 's/^failure_category=\([a-z0-9_]*\)$/\1/p' "$log" | tail -1)"
	printf '%s\n' "${maybe_category:-detector_failed}"
}

run_detector_gate() {
	local detector_log="$local_root/raw/detector.log"
	local detector_status
	set +e
	if [[ -n "$fixture_command" ]]; then
		fixture detector >"$detector_log" 2>&1
		detector_status=$?
	else
		just detect-ultra205 >"$detector_log" 2>&1
		detector_status=$?
	fi
	set -e
	chmod 600 "$detector_log"
	if ((detector_status != 0)); then
		failure_category="$(detector_failure "$detector_log")"
		return 1
	fi

	local port_count board_count
	port_count="$(sed -n 's/^port=//p' "$detector_log" | wc -l | tr -d ' ')"
	board_count="$(sed -n 's/^board=//p' "$detector_log" | wc -l | tr -d ' ')"
	[[ "$port_count" == "1" ]] || {
		failure_category="detector_candidate_count_invalid"
		return 1
	}
	if [[ "$board_count" != "0" ]]; then
		[[ "$board_count" == "1" && "$(sed -n 's/^board=//p' "$detector_log")" == "205" ]] || {
			failure_category="wrong_board"
			return 1
		}
	fi
	port="$(sed -n 's/^port=//p' "$detector_log")"

	if [[ -n "$fixture_command" ]]; then
		physical_identity_digest="$(fixture physical_identity "$port")" || {
			failure_category="physical_identity_unavailable"
			return 1
		}
	else
		physical_identity_digest="$(serial_session_usb_physical_identity "$port")" || {
			failure_category="physical_identity_unavailable"
			return 1
		}
	fi
	[[ "$physical_identity_digest" =~ ^[0-9a-f]{64}$ ]] || {
		failure_category="physical_identity_invalid"
		return 1
	}
	if [[ -n "$fixture_command" ]]; then
		stage_identity_digest="$(fixture stage_readiness detector)" || {
			failure_category="detector_stage_unstable"
			return 1
		}
	else
		local enumeration_identity
		enumeration_identity="$(serial_session_usb_enumeration_identity "$port")" || {
			failure_category="enumeration_identity_unavailable"
			return 1
		}
		stage_identity_digest="$(printf '%s\n%s\n' "$physical_identity_digest" "$enumeration_identity" | serial_session_hash_text)"
	fi

	jq -cn \
		--arg board_category 205 \
		--arg physical_identity_digest "$physical_identity_digest" \
		--arg run_id_digest "$run_id_digest" \
		'{board_category:$board_category,physical_identity_digest:$physical_identity_digest,run_id_digest:$run_id_digest,board_info_verified:true,single_candidate_verified:true}' \
		>"$local_root/artifacts/detector-capability.json"
	chmod 600 "$local_root/artifacts/detector-capability.json"
	local detector_artifact_digest
	detector_artifact_digest="$(sha256_file "$local_root/artifacts/detector-capability.json")"
	detector_capability_digest="$(hash_fields phase35-detector-run-v1 \
		205 "$detector_artifact_digest" "$physical_identity_digest" true true "$run_id_digest")"
	jq \
		--arg detector_capability_digest "$detector_artifact_digest" \
		--arg capability_digest "$detector_capability_digest" \
		'. + {detector_capability_digest:$detector_capability_digest,capability_digest:$capability_digest}' \
		"$local_root/artifacts/detector-capability.json" >"$local_root/raw/detector-run-capability.json"
	chmod 600 "$local_root/raw/detector-run-capability.json"

	jq -cn \
		--arg board_category 205 \
		--arg physical_identity_digest "$physical_identity_digest" \
		--arg run_id_digest "$run_id_digest" \
		'{board_category:$board_category,physical_identity_digest:$physical_identity_digest,run_id_digest:$run_id_digest,target_locked:true}' \
		>"$local_root/artifacts/target-lock.json"
	chmod 600 "$local_root/artifacts/target-lock.json"
	target_lock_digest="$(sha256_file "$local_root/artifacts/target-lock.json")"
}

run_stage_readiness_gate() {
	local stage="$1"
	if [[ -n "$fixture_command" ]]; then
		local observed
		observed="$(fixture stage_readiness "$stage")" || {
			failure_category="${stage}_readiness_unavailable"
			return 1
		}
		[[ "$observed" == "$stage_identity_digest" ]] || {
			failure_category="${stage}_identity_changed"
			return 1
		}
		return
	fi
	if ! serial_session_readiness_gate "phase35-${stage}" "$port"; then
		failure_category="${stage}_${SERIAL_SESSION_READINESS_CATEGORY:-readiness_failed}"
		return 1
	fi
	[[ "$SERIAL_SESSION_READY_PHYSICAL_IDENTITY" == "$physical_identity_digest" ]] || {
		failure_category="${stage}_physical_identity_changed"
		return 1
	}
	stage_identity_digest="$SERIAL_SESSION_READY_IDENTITY"
}

validate_credential_path_after_detector() {
	[[ -n "$wifi_credentials" ]] || return 0
	local resolved_credentials
	resolved_credentials="$(absolute_path "$wifi_credentials")"
	if [[ -n "$fixture_command" ]]; then
		fixture credential_path "$resolved_credentials" >/dev/null
		wifi_credentials="$resolved_credentials"
		return
	fi
	[[ -f "$resolved_credentials" ]] || {
		failure_category="wifi_credentials_path_missing"
		return 1
	}
	[[ "$resolved_credentials" == "${workspace_dir}/"* ]] || {
		failure_category="wifi_credentials_path_not_ignored"
		return 1
	}
	local workspace_relative_credentials="${resolved_credentials#"${workspace_dir}/"}"
	git -C "$workspace_dir" check-ignore -q -- "$workspace_relative_credentials" || {
		failure_category="wifi_credentials_path_not_ignored"
		return 1
	}
	wifi_credentials="$resolved_credentials"
}

production_classify_boot() {
	local mode="$1"
	local trace="$2"
	local output="$3"
	shift 3
	local classifier_stderr="${output%.json}.stderr"
	if ! "${workspace_dir}/bazel-bin/tools/parity/report" phase33-classify \
		--trace "$trace" \
		--mode "$mode" \
		"$@" >"$output" 2>"$classifier_stderr"; then
		chmod 600 "$output" "$classifier_stderr"
		failure_category="boot_classifier_failed"
		return 1
	fi
	chmod 600 "$output" "$classifier_stderr"

	local classification_status
	classification_status="$(jq -er '.status' "$output")" || {
		failure_category="boot_classifier_output_invalid"
		return 1
	}
	if [[ "$classification_status" == "passed" ]]; then
		return 0
	fi
	if [[ "$classification_status" != "failed" ]]; then
		failure_category="boot_classifier_output_invalid"
		return 1
	fi

	local classification_category
	classification_category="$(jq -er \
		'.category | select(type == "string") | select(test("^[a-z0-9_]+$"))' \
		"$output")" || {
		failure_category="boot_classifier_rejected"
		return 1
	}
	failure_category="$classification_category"
	return 1
}

classify_flash_stage() {
	local stage="$1"
	local metrics="$2"
	local private_log="$3"
	local projection="$local_root/artifacts/flash-${stage}-boundary.json"
	local classifier_stderr="$local_root/raw/flash-${stage}-classifier.stderr"
	local classifier_status=0
	set +e
	"${workspace_dir}/bazel-bin/tools/parity/report" classify-phase35-flash \
		--metrics-input "$metrics" \
		--private-log-input "$private_log" \
		--projection-output "$projection" \
		>"$local_root/raw/flash-${stage}-classifier.stdout" 2>"$classifier_stderr"
	classifier_status=$?
	set -e
	chmod 600 "$local_root/raw/flash-${stage}-classifier.stdout" "$classifier_stderr"
	[[ -f "$projection" && ! -L "$projection" ]] || {
		failure_category="flash_boundary_invalid"
		return 1
	}
	chmod 600 "$projection"
	jq -e \
		--arg schema phase35-flash-boundary-v1 \
		--arg stage "$stage" \
		'((keys | sort) == (["completed","connected","device_info_complete","duration_millis","launched","schema_version","stage","terminal_boundary","tool_version_valid","transfer_started"] | sort)) and .schema_version == $schema and .stage == $stage and (.terminal_boundary | type == "string" and test("^(version_mismatch|spawn_failure|pre_connect_failure|device_info_failure|post_info_pre_transfer_failed|transfer_failure|post_transfer_failure|ready)$"))' \
		"$projection" >/dev/null || {
		failure_category="flash_boundary_invalid"
		return 1
	}
	flash_boundary_schema="$(jq -r '.schema_version' "$projection")"
	flash_stage="$(jq -r '.stage' "$projection")"
	flash_boundary="$(jq -r '.terminal_boundary' "$projection")"
	if [[ "$flash_boundary" == ready ]]; then
		((classifier_status == 0)) || {
			failure_category="flash_boundary_invalid"
			return 1
		}
		return 0
	fi
	((classifier_status != 0)) || {
		failure_category="flash_boundary_invalid"
		return 1
	}
	failure_category="$flash_boundary"
	return 1
}

classify_available_flash_stages() {
	local stage
	local classified_count=0
	for stage in factory nvs monitor; do
		local metrics="$local_root/raw/flash/private-stages/${stage}.metrics.json"
		local private_log="$local_root/raw/flash/private-stages/${stage}.private.log"
		if [[ ! -e "$metrics" && ! -e "$private_log" ]]; then
			continue
		fi
		[[ -f "$metrics" && ! -L "$metrics" && -f "$private_log" && ! -L "$private_log" ]] || {
			failure_category="flash_boundary_invalid"
			return 1
		}
		((classified_count += 1))
		classify_flash_stage "$stage" "$metrics" "$private_log" || return 1
	done
	((classified_count > 0))
}

run_checksum_probe() {
	if ! run_stage_readiness_gate before_probe; then
		return 1
	fi
	if [[ -n "$fixture_command" && "$fixture_direct_flash" != true ]]; then
		fixture flash_probe "$port" >/dev/null || {
			failure_category="flash_probe_failed"
			return 1
		}
		flash_boundary_schema="phase35-flash-boundary-v1"
		flash_stage="probe"
		flash_boundary="ready"
		run_stage_readiness_gate after_probe
		return
	fi

	local flash_dir="$local_root/raw/flash"
	local stage_root="$flash_dir/private-stages"
	mkdir -p "$flash_dir" "$stage_root"
	chmod 700 "$flash_dir" "$stage_root"
	if [[ -z "$fixture_command" ]]; then
		local selected_espflash=""
		local selected_version=""
		local selected_digest=""
		local resolver_status=0
		set +e
		selected_espflash="$(espflash_resolve_bin)"
		resolver_status=$?
		set -e
		if ((resolver_status == 0)) && [[ -n "$selected_espflash" ]]; then
			local version_status=0
			local digest_status=0
			set +e
			selected_version="$(espflash_version "$selected_espflash")"
			version_status=$?
			selected_digest="$(espflash_executable_digest "$selected_espflash")"
			digest_status=$?
			set -e
			if ((version_status != 0 || digest_status != 0)); then
				selected_version=""
				selected_digest=""
			fi
		fi
		if [[ -z "$selected_version" || ! "$selected_digest" =~ ^[0-9a-f]{64}$ ]]; then
			printf 'phase35_espflash_version_contract_failed\n' >"$stage_root/probe.private.log"
			jq -cn \
				'{schema_version:"phase35-flash-boundary-v1",stage:"probe",tool_version_valid:false,launched:false,connected:false,device_info_complete:false,transfer_started:false,completed:false,duration_millis:0}' \
				>"$stage_root/probe.metrics.json"
			chmod 600 "$stage_root/probe.private.log" "$stage_root/probe.metrics.json"
			classify_flash_stage probe \
				"$stage_root/probe.metrics.json" \
				"$stage_root/probe.private.log"
			return
		fi
		jq -cn \
			--arg version "$ESPFLASH_EXPECTED_VERSION" \
			--arg executable_sha256 "$selected_digest" \
			'{version:$version,executable_sha256:$executable_sha256}' \
			>"$local_root/artifacts/espflash-provenance.json"
		chmod 600 "$local_root/artifacts/espflash-provenance.json"
	fi
	local flash_executable
	flash_executable="$(resolve_flash_executable)" || {
		failure_category="flash_executable_unavailable"
		return 1
	}
	local probe_status=0
	set +e
	"$flash_executable" phase35-probe \
		--board 205 \
		--port "$port" \
		--stage-root "$stage_root" \
		--timeout-seconds 30 \
		>"$local_root/raw/flash-probe-command.log" 2>&1
	probe_status=$?
	set -e
	chmod 600 "$local_root/raw/flash-probe-command.log"
	local metrics="$stage_root/probe.metrics.json"
	local private_log="$stage_root/probe.private.log"
	[[ -f "$metrics" && ! -L "$metrics" && -f "$private_log" && ! -L "$private_log" ]] || {
		failure_category="flash_boundary_invalid"
		return 1
	}
	if ! classify_flash_stage probe "$metrics" "$private_log"; then
		return 1
	fi
	((probe_status == 0)) || {
		failure_category="flash_boundary_invalid"
		return 1
	}
	run_stage_readiness_gate after_probe
}
