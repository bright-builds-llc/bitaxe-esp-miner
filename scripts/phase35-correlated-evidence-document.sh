#!/usr/bin/env bash
# Evidence document construction and validation helpers for the Phase 35 supervisor.
# shellcheck disable=SC2034,SC2154

write_epoch_artifacts() {
	local epoch_name="$1"
	local snapshot="$2"
	jq -er '.system_info_document' "$snapshot" >"$local_root/artifacts/${epoch_name}-api.txt"
	jq -er '.websocket_document' "$snapshot" >"$local_root/artifacts/${epoch_name}-websocket.txt"
	jq -er '.retained_log_document' "$snapshot" >"$local_root/artifacts/${epoch_name}-retained-log.txt"
	chmod 600 \
		"$local_root/artifacts/${epoch_name}-api.txt" \
		"$local_root/artifacts/${epoch_name}-websocket.txt" \
		"$local_root/artifacts/${epoch_name}-retained-log.txt"
}

inventory_entry() {
	local role="$1"
	local path="$2"
	local digest
	digest="$(sha256_file "$local_root/$path")"
	jq -cn --arg role "$role" --arg path "$path" --arg sha256 "$digest" \
		'{role:$role,path:$path,sha256:$sha256}'
}

build_inventory() {
	{
		inventory_entry package_manifest artifacts/package-manifest.json
		inventory_entry executable_image artifacts/executable-image.bin
		inventory_entry factory_image artifacts/factory-image.bin
		inventory_entry package artifacts/package.json
		inventory_entry runtime_identity artifacts/runtime-identity.json
		inventory_entry target_lock artifacts/target-lock.json
		inventory_entry detector_capability artifacts/detector-capability.json
		inventory_entry no_actuation artifacts/no-actuation.txt
		inventory_entry boot_a_api artifacts/boot-a-api.txt
		inventory_entry boot_a_websocket artifacts/boot-a-websocket.txt
		inventory_entry boot_a_retained_log artifacts/boot-a-retained-log.txt
		inventory_entry boot_b_api artifacts/boot-b-api.txt
		inventory_entry boot_b_websocket artifacts/boot-b-websocket.txt
		inventory_entry boot_b_retained_log artifacts/boot-b-retained-log.txt
	} | jq -s '.' >"$local_root/raw/inventory.json"
	chmod 600 "$local_root/raw/inventory.json"
}

inventory_digest() {
	local -a fields=()
	while IFS= read -r field; do
		fields+=("$field")
	done < <(jq -r '.[] | .role, .path, .sha256' "$local_root/raw/inventory.json")
	hash_fields phase35-inventory-v1 "${fields[@]}"
}

build_epoch_input() {
	local snapshot="$1"
	local setting_digest="$2"
	local reset_category="$3"
	local session
	session="$(jq -er '.boot_session' "$snapshot")"
	jq -cn \
		--argjson boot_ordinal "$(jq -er '.boot_ordinal' "$snapshot")" \
		--arg boot_session_digest "$(sha256_text "$session")" \
		--argjson started_millis "$(jq -er '.started_millis' "$snapshot")" \
		--argjson ended_millis "$(jq -er '.ended_millis' "$snapshot")" \
		--arg system_info_document "$(jq -er '.system_info_document' "$snapshot")" \
		--arg websocket_document "$(jq -er '.websocket_document' "$snapshot")" \
		--arg retained_log_document "$(jq -er '.retained_log_document' "$snapshot")" \
		--argjson storage_revision "$(jq -er '.storage_revision' "$snapshot")" \
		--arg storage_value_digest "$setting_digest" \
		--arg reset_category "$reset_category" \
		--arg package_capability_digest "$package_capability_digest" \
		--arg detector_capability_digest "$detector_capability_digest" \
		--arg root_contract_digest "$root_contract_digest" \
		--arg target_lock_digest "$target_lock_digest" \
		--arg run_id_digest "$run_id_digest" \
		--arg runtime_identity_digest "$(sha256_file "$local_root/artifacts/runtime-identity.json")" \
		--arg physical_identity_digest "$physical_identity_digest" \
		'{boot_ordinal:$boot_ordinal,boot_session_digest:$boot_session_digest,started_millis:$started_millis,ended_millis:$ended_millis,system_info_document:$system_info_document,websocket_document:$websocket_document,retained_log_document:$retained_log_document,storage_revision:$storage_revision,storage_value_digest:$storage_value_digest,reset_category:$reset_category,package_capability_digest:$package_capability_digest,detector_capability_digest:$detector_capability_digest,root_contract_digest:$root_contract_digest,target_lock_digest:$target_lock_digest,run_id_digest:$run_id_digest,runtime_identity_digest:$runtime_identity_digest,physical_identity_digest:$physical_identity_digest}'
}

epoch_digest() {
	local epoch="$1"
	hash_fields phase35-epoch-v1 \
		"$(jq -r '.boot_ordinal' "$epoch")" \
		"$(jq -r '.boot_session_digest' "$epoch")" \
		"$(jq -r '.started_millis' "$epoch")" \
		"$(jq -r '.ended_millis' "$epoch")" \
		"$(sha256_text "$(jq -r '.system_info_document' "$epoch")")" \
		"$(sha256_text "$(jq -r '.websocket_document' "$epoch")")" \
		"$(sha256_text "$(jq -r '.retained_log_document' "$epoch")")" \
		"$(jq -r '.storage_revision' "$epoch")" \
		"$(jq -r '.storage_value_digest' "$epoch")" \
		"$(jq -r '.reset_category' "$epoch")" \
		"$(jq -r '.package_capability_digest' "$epoch")" \
		"$(jq -r '.detector_capability_digest' "$epoch")" \
		"$(jq -r '.root_contract_digest' "$epoch")" \
		"$(jq -r '.target_lock_digest' "$epoch")" \
		"$(jq -r '.run_id_digest' "$epoch")" \
		"$(jq -r '.runtime_identity_digest' "$epoch")" \
		"$(jq -r '.physical_identity_digest' "$epoch")"
}

checkpoint_time() {
	local category="$1"
	awk -F '\t' -v category="$category" '$2 == category { print $3 }' "$local_root/raw/chronology.tsv"
}

append_event() {
	local sequence="$1"
	local category="$2"
	local payload="$3"
	local timestamp
	timestamp="$(checkpoint_time "$category")"
	[[ "$timestamp" =~ ^[0-9]+$ ]] || return 1
	local event
	event="$(jq -cn \
		--argjson sequence "$sequence" \
		--arg category "$category" \
		--argjson monotonic_millis "$timestamp" \
		--arg payload_digest "$payload" \
		--arg predecessor_event_digest "$event_predecessor" \
		'{sequence:$sequence,category:$category,monotonic_millis:$monotonic_millis,payload_digest:$payload_digest,predecessor_event_digest:$predecessor_event_digest}')"
	printf '%s\n' "$event" >>"$local_root/raw/events.jsonl"
	event_predecessor="$(hash_fields phase35-event-v1 "$sequence" "$category" "$timestamp" "$payload" "$event_predecessor")"
}

build_events() {
	: >"$local_root/raw/events.jsonl"
	chmod 600 "$local_root/raw/events.jsonl"
	event_predecessor="$root_contract_digest"
	append_event 1 root_admitted "$root_contract_digest"
	append_event 2 boot_a_observed "$(epoch_digest "$local_root/raw/boot-a-input.json")"
	append_event 3 patch_responded "$(jq -r '.storage_value_digest' "$local_root/raw/boot-a-input.json")"
	append_event 4 storage_confirmed "$(hash_fields phase35-storage-confirmation-v1 \
		"$(jq -r '.storage_revision' "$local_root/raw/boot-a-input.json")" \
		"$(jq -r '.storage_value_digest' "$local_root/raw/boot-a-input.json")")"
	append_event 5 reboot_started "$(hash_fields phase35-reboot-v1 \
		"$(jq -r '.boot_ordinal' "$local_root/raw/boot-a-input.json")" \
		"$(jq -r '.boot_ordinal' "$local_root/raw/boot-b-input.json")" software_cpu)"
	append_event 6 boot_b_observed "$(epoch_digest "$local_root/raw/boot-b-input.json")"
	append_event 7 no_actuation_verified "$(sha256_file "$local_root/artifacts/no-actuation.txt")"
	append_event 8 restoration_confirmed "$(hash_fields phase35-restoration-v1 true)"
	append_event 9 cleanup_confirmed "$(hash_fields phase35-cleanup-v1 true)"
	jq -s '.' "$local_root/raw/events.jsonl" >"$local_root/raw/events.json"
	chmod 600 "$local_root/raw/events.json"
}

build_evidence_root() {
	local boot_a_snapshot="$1"
	local boot_b_snapshot="$2"
	local setting_digest="$3"
	write_epoch_artifacts boot-a "$boot_a_snapshot"
	write_epoch_artifacts boot-b "$boot_b_snapshot"
	write_private "$local_root/artifacts/no-actuation.txt" "no_actuation_verified=true"
	build_inventory

	local inventory_contract
	inventory_contract="$(inventory_digest)"
	local runtime_identity_digest
	runtime_identity_digest="$(sha256_file "$local_root/artifacts/runtime-identity.json")"
	root_contract_digest="$(hash_fields phase35-root-contract-v1 \
		"$package_capability_digest" "$detector_capability_digest" "$target_lock_digest" \
		"$PHASE35_LIFECYCLE_ID" "$run_id_digest" "$inventory_contract" \
		"$(jq -r '.boot_ordinal' "$boot_a_snapshot")" \
		"$(sha256_text "$(jq -r '.boot_session' "$boot_a_snapshot")")" \
		"$(jq -r '.boot_ordinal' "$boot_b_snapshot")" \
		"$(sha256_text "$(jq -r '.boot_session' "$boot_b_snapshot")")" \
		"$runtime_identity_digest")"

	build_epoch_input "$boot_a_snapshot" "$setting_digest" setup >"$local_root/raw/boot-a-input.json"
	build_epoch_input "$boot_b_snapshot" "$setting_digest" software_cpu >"$local_root/raw/boot-b-input.json"
	chmod 600 "$local_root/raw/boot-a-input.json" "$local_root/raw/boot-b-input.json"
	build_events

	jq -n \
		--slurpfile exact_package "$local_root/raw/exact-package-capability.json" \
		--slurpfile detector_run "$local_root/raw/detector-run-capability.json" \
		--slurpfile inventory "$local_root/raw/inventory.json" \
		--slurpfile events "$local_root/raw/events.json" \
		--slurpfile boot_a "$local_root/raw/boot-a-input.json" \
		--slurpfile boot_b "$local_root/raw/boot-b-input.json" \
		--arg schema_version "$PHASE35_SCHEMA" \
		--arg root_contract_digest "$root_contract_digest" \
		--arg target_lock_digest "$target_lock_digest" \
		--arg lifecycle_id "$PHASE35_LIFECYCLE_ID" \
		'{schema_version:$schema_version,exact_package:$exact_package[0],detector_run:$detector_run[0],admission_facts:{root_contract_digest:$root_contract_digest,target_lock_digest:$target_lock_digest,lifecycle_id:$lifecycle_id,lifecycle_verified:true,current_head_rechecked:true,reference_cleanliness_rechecked:true,runtime_identity_rechecked:true,no_actuation_verified:true,inventory_verified:true,chronology_verified:true,restoration_verified:true,cleanup_verified:true,redaction_verified:true},events:$events[0],inventory:$inventory[0],boot_a:$boot_a[0],boot_b:$boot_b[0]}' \
		>"$local_root/eligible.json"
	chmod 600 "$local_root/eligible.json"
}

validate_target_after_capture() {
	[[ -n "$target_token" ]] || return 1
	if [[ -n "$fixture_command" ]]; then
		fixture validate_target "$target_token"
		return
	fi
	[[ "$target_token" =~ ^https?://[^/?#]+/?$ ]] || return 1
	[[ "$target_token" != *"@"* ]]
}

snapshot_setting_matches() {
	local snapshot="$1"
	local expected="$2"
	local maybe_digest
	maybe_digest="$(jq -er '.setting_digest // empty' "$snapshot")"
	[[ -z "$maybe_digest" || "$maybe_digest" == "$(sha256_text "$expected")" ]]
}

run_live_rechecks() {
	if [[ -n "$fixture_command" ]]; then
		local check
		for check in current_head reference lifecycle runtime_identity no_actuation; do
			fixture recheck "$check" >/dev/null || {
				failure_category="${check}_recheck_failed"
				return 1
			}
		done
		return
	fi

	local source_commit reference_commit
	source_commit="$(jq -er '.source_commit' "$local_root/raw/exact-package-capability.json")"
	reference_commit="$(jq -er '.reference_commit' "$local_root/raw/exact-package-capability.json")"
	[[ "$(git -C "$workspace_dir" rev-parse HEAD)" == "$source_commit" ]] || {
		failure_category="current_head_recheck_failed"
		return 1
	}
	run_reference_guard || {
		failure_category="reference_recheck_failed"
		return 1
	}
	[[ "$(git -C "$workspace_dir/reference/esp-miner" rev-parse HEAD)" == "$reference_commit" ]] || {
		failure_category="reference_recheck_failed"
		return 1
	}
	rg -q "phase_lifecycle_id: ${PHASE35_LIFECYCLE_ID}" \
		"$workspace_dir/.planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-02-PLAN.md" || {
		failure_category="lifecycle_recheck_failed"
		return 1
	}
	[[ "$(sha256_file "$local_root/artifacts/runtime-identity.json")" == "$(jq -er '.runtime_identity_digest' "$local_root/raw/exact-package-capability.json")" ]] || {
		failure_category="runtime_identity_recheck_failed"
		return 1
	}
}

run_validator() {
	local projection="$local_root/result.json"
	if [[ -n "$fixture_command" ]]; then
		fixture validator "$local_root" >"$projection"
	else
		"${workspace_dir}/bazel-bin/tools/parity/report" \
			validate-phase35-evidence \
			--root "$local_root" >"$projection"
	fi
	chmod 600 "$projection"
	if rg -qi '"[^"]*(credential|device|endpoint|hostname|ip|mac|network|origin|password|path|pid|pool|port|raw|secret|ssid|target)[^"]*"[[:space:]]*:' "$projection"; then
		failure_category="redaction_failed"
		return 1
	fi
	local canary
	for canary in "$target_token" "$original_setting" "$mutated_setting" "$port"; do
		[[ -z "$canary" ]] && continue
		if rg -Fq "$canary" "$projection"; then
			failure_category="redaction_failed"
			return 1
		fi
	done
}
