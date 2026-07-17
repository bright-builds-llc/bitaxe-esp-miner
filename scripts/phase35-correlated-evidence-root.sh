#!/usr/bin/env bash
# Package admission and protected-root helpers for the Phase 35 supervisor.
# shellcheck disable=SC2034,SC2154

sha256_file() {
	shasum -a 256 "$1" | awk '{print $1}'
}

sha256_text() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

hash_fields() {
	local domain="$1"
	shift
	{
		printf '%s' "$domain"
		local field
		for field in "$@"; do
			printf '\0%s' "$field"
		done
	} | shasum -a 256 | awk '{print $1}'
}

monotonic_millis() {
	if [[ -n "${PHASE35_MONOTONIC_COMMAND:-}" ]]; then
		"$PHASE35_MONOTONIC_COMMAND"
		return
	fi
	perl -MTime::HiRes=clock_gettime,CLOCK_MONOTONIC -e 'printf "%.0f\n", clock_gettime(CLOCK_MONOTONIC) * 1000'
}

absolute_path() {
	local path="$1"
	if [[ "$path" == /* ]]; then
		printf '%s\n' "$path"
		return
	fi
	printf '%s/%s\n' "$workspace_dir" "$path"
}

prepare_root() {
	umask 077
	if [[ -z "$local_root" ]]; then
		local seed
		seed="$(date -u '+%Y%m%dT%H%M%SZ')-$$-${RANDOM}"
		local_root="${workspace_dir}/scratch/phase35-correlated-evidence/$(sha256_text "$seed")"
	else
		local_root="$(absolute_path "$local_root")"
	fi

	if [[ -e "$local_root" ]]; then
		printf 'failure_category=evidence_root_already_exists\n' >&2
		exit 2
	fi
	if [[ "$local_root" == "${workspace_dir}/"* ]]; then
		local relative="${local_root#"${workspace_dir}/"}"
		git -C "$workspace_dir" check-ignore -q -- "$relative" || {
			printf 'failure_category=evidence_root_not_ignored\n' >&2
			exit 2
		}
	fi

	mkdir -p "$local_root/raw" "$local_root/artifacts"
	chmod 700 "$local_root" "$local_root/raw" "$local_root/artifacts"
	: >"$local_root/raw/chronology.tsv"
	chmod 600 "$local_root/raw/chronology.tsv"
	run_id_digest="$(hash_fields phase35-run-v1 "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$$" "$RANDOM")"
}

write_private() {
	local path="$1"
	shift
	printf '%s\n' "$@" >"$path"
	chmod 600 "$path"
}

record_checkpoint() {
	local category="$1"
	local payload_digest="$2"
	local now
	now="$(monotonic_millis)"
	if ((now <= last_event_millis)); then
		now=$((last_event_millis + 1))
	fi
	last_event_millis="$now"
	((event_sequence += 1))
	printf '%s\t%s\t%s\n' "$event_sequence" "$category" "$now" >>"$local_root/raw/chronology.tsv"
	printf '%s\t%s\n' "$category" "$payload_digest" >>"$local_root/raw/checkpoints.tsv"
	chmod 600 "$local_root/raw/checkpoints.tsv"
}

fixture() {
	[[ -n "$fixture_command" ]] || return 127
	"$fixture_command" "$@"
}

require_unique_artifact() {
	local kind="$1"
	local count
	count="$(jq --arg kind "$kind" '[.artifacts[] | select(.kind == $kind)] | length' "$manifest")"
	[[ "$count" == "1" ]] || return 1
	jq -er --arg kind "$kind" '.artifacts[] | select(.kind == $kind) | .path' "$manifest"
}

run_package_admission() {
	local admission_log="$local_root/raw/package-admission.log"
	if [[ -n "$fixture_command" ]]; then
		fixture package_admission "$manifest" >"$admission_log" 2>&1
	else
		if [[ ! -x "${workspace_dir}/bazel-bin/tools/flash/flash" ]]; then
			bazel build //tools/flash:flash >"$admission_log" 2>&1
		fi
		"${workspace_dir}/bazel-bin/tools/flash/flash" flash \
			--dry-run \
			--board 205 \
			--port phase35-gate1-inert \
			--manifest "$manifest" >>"$admission_log" 2>&1
	fi
	chmod 600 "$admission_log"
}

run_reference_guard() {
	if [[ -n "$fixture_command" ]]; then
		fixture reference_guard
		return
	fi
	"${script_dir}/verify-reference-clean.sh" >/dev/null
}

copy_admitted_artifact() {
	local source="$1"
	local destination="$2"
	cp "$source" "$destination"
	chmod 600 "$destination"
}

run_gate_one() {
	manifest="$(absolute_path "$manifest")"
	[[ -f "$manifest" ]] || {
		failure_category="required_package_missing"
		return 1
	}
	run_package_admission || {
		failure_category="package_admission_failed"
		return 1
	}
	run_reference_guard || {
		failure_category="reference_dirty"
		return 1
	}

	local schema source_commit reference_commit current_head
	schema="$(jq -er '.schema_version' "$manifest")" || return 1
	[[ "$schema" == "3" ]] || {
		failure_category="manifest_schema_not_v3"
		return 1
	}
	source_commit="$(jq -er '.source_commit' "$manifest")"
	reference_commit="$(jq -er '.reference_commit' "$manifest")"
	if [[ -n "$fixture_command" ]]; then
		current_head="$(fixture current_head)"
	else
		current_head="$(git -C "$workspace_dir" rev-parse HEAD)"
	fi
	[[ "$source_commit" == "$current_head" ]] || {
		failure_category="source_head_drift"
		return 1
	}

	local manifest_dir executable_relative factory_relative executable_path factory_path
	manifest_dir="$(dirname "$manifest")"
	executable_relative="$(require_unique_artifact firmware_elf)" || {
		failure_category="executable_artifact_invalid"
		return 1
	}
	factory_relative="$(require_unique_artifact factory_merged_image)" || {
		failure_category="factory_artifact_invalid"
		return 1
	}
	executable_path="${manifest_dir}/${executable_relative}"
	factory_path="${manifest_dir}/${factory_relative}"
	[[ -f "$executable_path" && -f "$factory_path" ]] || {
		failure_category="package_artifact_missing"
		return 1
	}

	copy_admitted_artifact "$manifest" "$local_root/artifacts/package-manifest.json"
	copy_admitted_artifact "$executable_path" "$local_root/artifacts/executable-image.bin"
	copy_admitted_artifact "$factory_path" "$local_root/artifacts/factory-image.bin"

	local manifest_digest executable_digest factory_digest runtime_identity_digest package_digest
	manifest_digest="$(sha256_file "$local_root/artifacts/package-manifest.json")"
	executable_digest="$(sha256_file "$local_root/artifacts/executable-image.bin")"
	factory_digest="$(sha256_file "$local_root/artifacts/factory-image.bin")"
	jq -cn \
		--arg source_commit "$source_commit" \
		--arg app_elf_sha256 "$(jq -er '.app_elf_sha256' "$manifest")" \
		--arg build_label "$(jq -er '.build_identity.label' "$manifest")" \
		'{source_commit:$source_commit,app_elf_sha256:$app_elf_sha256,build_label:$build_label}' \
		>"$local_root/artifacts/runtime-identity.json"
	chmod 600 "$local_root/artifacts/runtime-identity.json"
	runtime_identity_digest="$(sha256_file "$local_root/artifacts/runtime-identity.json")"
	jq -cn \
		--arg manifest "$manifest_digest" \
		--arg executable "$executable_digest" \
		--arg factory "$factory_digest" \
		--arg runtime "$runtime_identity_digest" \
		'{manifest_digest:$manifest,executable_image_digest:$executable,factory_image_digest:$factory,runtime_identity_digest:$runtime}' \
		>"$local_root/artifacts/package.json"
	chmod 600 "$local_root/artifacts/package.json"
	package_digest="$(sha256_file "$local_root/artifacts/package.json")"
	package_capability_digest="$(hash_fields phase35-exact-package-v1 \
		"$source_commit" "$reference_commit" true manifest-v3 "$manifest_digest" \
		"$executable_digest" "$factory_digest" "$package_digest" "$runtime_identity_digest" true)"

	jq -cn \
		--arg source_commit "$source_commit" \
		--arg reference_commit "$reference_commit" \
		--arg manifest_digest "$manifest_digest" \
		--arg executable_image_digest "$executable_digest" \
		--arg factory_image_digest "$factory_digest" \
		--arg package_digest "$package_digest" \
		--arg runtime_identity_digest "$runtime_identity_digest" \
		--arg capability_digest "$package_capability_digest" \
		'{source_commit:$source_commit,reference_commit:$reference_commit,reference_clean:true,manifest_schema:"manifest-v3",manifest_digest:$manifest_digest,executable_image_digest:$executable_image_digest,factory_image_digest:$factory_image_digest,package_digest:$package_digest,runtime_identity_digest:$runtime_identity_digest,current_head_verified:true,capability_digest:$capability_digest}' \
		>"$local_root/raw/exact-package-capability.json"
	chmod 600 "$local_root/raw/exact-package-capability.json"
}

# shellcheck source=scripts/serial-session-trace.sh
# shellcheck disable=SC1091
source "${script_dir}/serial-session-trace.sh"
