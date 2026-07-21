#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir

if [[ -n "${TEST_SRCDIR:-}" && -f "${TEST_SRCDIR}/_main/docs/hardware/hardware-attempt-policy.md" ]]; then
	workspace_root="${TEST_SRCDIR}/_main"
else
	workspace_root="$(cd "${script_dir}/.." && pwd)"
fi
readonly workspace_root
readonly policy_file="${workspace_root}/docs/hardware/hardware-attempt-policy.md"
readonly agents_file="${workspace_root}/AGENTS.md"

fail() {
	printf 'hardware_attempt_policy_contract_test_error: category=%s\n' "$1" >&2
	exit 1
}

require_file() {
	[[ -f "$1" ]] || fail artifact-inventory
}

require_literal() {
	local source_file="$1"
	local expected="$2"
	local category="$3"

	rg -q -F -- "$expected" "$source_file" || fail "$category"
}

require_policy_literal() {
	require_literal "$policy_file" "$1" "$2"
}

forbid_policy_regex() {
	local pattern="$1"
	local category="$2"
	local scan_result

	set +e
	rg -q -i -e "$pattern" -- "$policy_file"
	scan_result=$?
	set -e

	case "$scan_result" in
	0) fail "$category" ;;
	1) ;;
	*) fail scanner ;;
	esac
}

count_literal() {
	local source_file="$1"
	local literal="$2"

	awk -v needle="$literal" '
        {
            remaining = $0
            while ((position = index(remaining, needle)) > 0) {
                count += 1
                remaining = substr(remaining, position + length(needle))
            }
        }
        END { print count + 0 }
    ' "$source_file"
}

require_agents_guidance_literal() {
	local literal="$1"
	local found

	found="$(awk -v needle="\`${literal}\`" '
        /^### Progress-Gated Hardware Attempts$/ {
            in_section = 1
            next
        }
        in_section && /^### / {
            exit
        }
        in_section && index($0, needle) {
            found = 1
        }
        END { print found + 0 }
    ' "$agents_file")"
	[[ "$found" -eq 1 ]] || fail agents-outcome-summary
}

require_file "$policy_file"
require_file "$agents_file"

while IFS= read -r outcome; do
	[[ "$(count_literal "$policy_file" "$outcome")" -eq 1 ]] || fail outcome-vocabulary
	require_agents_guidance_literal "$outcome"
done <<'OUTCOMES'
continue_after_verified_fix
continue_after_manual_remediation
complete
stop_repeated_boundary
stop_hardware_blocker
stop_authority_boundary
stop_impossible_contract
OUTCOMES

require_policy_literal 'There is no fixed numeric attempt cap.' attempt-cap
require_policy_literal 'There is also no unchanged blind retry:' blind-retry
forbid_policy_regex '(maximum|max|at most|no more than)[[:space:]]+[0-9]+[[:space:]]+(hardware[[:space:]]+)?attempts?' numeric-attempt-cap
require_policy_literal 'The same authoritative boundary signature recurred once after its targeted fix was verified at the real boundary.' repeated-boundary
require_policy_literal 'category plus the minimum shareable discriminator fields needed to distinguish' boundary-signature
require_policy_literal 'A repeated coarse category with a newly' boundary-signature
require_policy_literal 'renamed category or discriminator that describes unchanged conditions is not a' boundary-signature
require_policy_literal 'raw identifiers, secrets, paths,' boundary-signature
require_policy_literal "The active phase's genuine hardware success and evidence criteria are all satisfied." genuine-completion

while IFS= read -r invariant; do
	require_policy_literal "$invariant" fresh-attempt-invariant
done <<'INVARIANTS'
fresh ordinal
command exactly once
exact current `HEAD`
exact package identity
mode-`0700` protected parent
child nonexistent
distinct mode-`0600` sibling files
attempt root immutable
earliest typed failure
exactly one closed outcome
INVARIANTS

while IFS= read -r progress_requirement; do
	require_policy_literal "$progress_requirement" progress-decision
done <<'PROGRESS'
diagnosis of the authoritative boundary signature
one targeted fix
regression that crosses the real production
all required software verification gates
new exact-current-HEAD
authorized by
non-invasive
objective evidence that the failed boundary changed
PROGRESS

while IFS= read -r command_requirement; do
	require_policy_literal "$command_requirement" phase-command-ownership
done <<'COMMAND_REQUIREMENTS'
detector admission and target identity
exact allowed effects and prohibited effects
device and operator safety limits
bounded recovery, restoration, and cleanup
private capture, redaction, evidence admission, and non-promotion rules
deterministic software regressions plus required hardware verification
COMMAND_REQUIREMENTS

while IFS= read -r closed_boundary; do
	require_policy_literal "$closed_boundary" unchanged-boundary
done <<'CLOSED_BOUNDARIES'
Direct UART, pins, pads,
Archived Phase 28.1.1 and its descendants remain terminal unresolved history
Phase 30 remains a conservative no-promotion boundary
docs/parity/evidence-policy.md
completion alone is never parity evidence
CLOSED_BOUNDARIES

while IFS= read -r fault_requirement; do
	require_policy_literal "$fault_requirement" fault-testing
done <<'FAULT_REQUIREMENTS'
both the active plan and the
repo-owned command encode repo- and vendor-safe limits
automatic abort
recovery, and required evidence
Electrical overstress is prohibited.
FAULT_REQUIREMENTS

managed_end_line="$(awk '/<!-- bright-builds-rules-managed:end -->/ { print NR; exit }' "$agents_file")"
guidance_line="$(awk '/^### Progress-Gated Hardware Attempts$/ { print NR; exit }' "$agents_file")"
[[ -n "$managed_end_line" && -n "$guidance_line" ]] || fail agents-placement
[[ "$guidance_line" -gt "$managed_end_line" ]] || fail agents-placement
[[ "$(count_literal "$agents_file" 'docs/hardware/hardware-attempt-policy.md')" -eq 1 ]] || fail agents-pointer

guidance_nonempty_lines="$(awk '
    /^### Progress-Gated Hardware Attempts$/ {
        in_section = 1
    }
    in_section && /^### / && $0 != "### Progress-Gated Hardware Attempts" {
        exit
    }
    in_section && NF {
        count += 1
    }
    END { print count + 0 }
' "$agents_file")"
guidance_bullets="$(awk '
    /^### Progress-Gated Hardware Attempts$/ {
        in_section = 1
        next
    }
    in_section && /^### / {
        exit
    }
    in_section && /^- / {
        count += 1
    }
    END { print count + 0 }
' "$agents_file")"
[[ "$guidance_nonempty_lines" -le 5 ]] || fail agents-conciseness
[[ "$guidance_bullets" -eq 4 ]] || fail agents-conciseness

printf 'hardware_attempt_policy_contract_test: passed\n'
